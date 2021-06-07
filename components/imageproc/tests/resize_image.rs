use std::env;
use std::path::PathBuf;

use lazy_static::lazy_static;

use config::Config;
use imageproc::{EnqueueResponse, ImageMetaResponse, Processor};
use utils::fs as ufs;

static CONFIG: &str = r#"
title = "imageproc integration tests"
base_url = "https://example.com"
compile_sass = false
build_search_index = false

[markdown]
highlight_code = false
"#;

lazy_static! {
    static ref TEST_IMGS: PathBuf =
        [env!("CARGO_MANIFEST_DIR"), "tests", "test_imgs"].iter().collect();
    static ref TMPDIR: PathBuf = {
        let tmpdir = option_env!("CARGO_TARGET_TMPDIR").map(PathBuf::from).unwrap_or_else(|| {
            env::current_exe().unwrap().parent().unwrap().parent().unwrap().join("tmpdir")
        });
        ufs::ensure_directory_exists(&tmpdir).unwrap();
        tmpdir
    };
    static ref PROCESSED_DIR: PathBuf = TMPDIR.join("static").join("processed_images");
}

fn image_op_test(
    source_img: &str,
    hash: &str,
    op: &str,
    width: Option<u32>,
    height: Option<u32>,
    format: &str,
    expect_ext: &str,
    expect_width: u32,
    expect_height: u32,
    orig_width: u32,
    orig_height: u32,
) {
    let source_path = TEST_IMGS.join(source_img);
    let hash_fn = format!("{}.{}", hash, expect_ext);

    let config = Config::parse(&CONFIG).unwrap();
    let mut proc = Processor::new(TMPDIR.clone(), &config);

    assert_eq!(
        proc.enqueue(source_img.into(), source_path, op, width, height, format, None).unwrap(),
        EnqueueResponse {
            url: format!("https://example.com/processed_images/{}", hash_fn),
            static_path: format!("static/processed_images/{}", hash_fn),
            width: expect_width,
            height: expect_height,
            orig_width,
            orig_height,
        }
    );

    proc.do_process().unwrap();

    let processed_size = imageproc::read_image_metadata(&PROCESSED_DIR.join(hash_fn))
        .map(|meta| (meta.width, meta.height))
        .unwrap();
    assert_eq!(processed_size, (expect_width, expect_height));
}

fn image_meta_test(source_img: &str) -> ImageMetaResponse {
    let source_path = TEST_IMGS.join(source_img);
    imageproc::read_image_metadata(&source_path).unwrap()
}

#[test]
fn resize_image_scale() {
    image_op_test(
        "jpg.jpg",
        "c8f95cf7298a3ac400",
        "scale",
        Some(150),
        Some(150),
        "auto",
        "jpg",
        150,
        150,
        300,
        380,
    );
}

#[test]
fn resize_image_fit_width() {
    image_op_test(
        "jpg.jpg",
        "ea70e772cb2c927500",
        "fit_width",
        Some(150),
        None,
        "auto",
        "jpg",
        150,
        190,
        300,
        380,
    );
}

#[test]
fn resize_image_fit_height() {
    image_op_test(
        "webp.webp",
        "89286296c7c4070d00",
        "fit_height",
        None,
        Some(190),
        "auto",
        "jpg",
        150,
        190,
        300,
        380,
    );
}

#[test]
fn resize_image_fit1() {
    image_op_test(
        "jpg.jpg",
        "ea70e772cb2c927500",
        "fit",
        Some(150),
        Some(200),
        "auto",
        "jpg",
        150,
        190,
        300,
        380,
    );
}

#[test]
fn resize_image_fit2() {
    image_op_test(
        "jpg.jpg",
        "83d2ef621b50ce6f00",
        "fit",
        Some(160),
        Some(180),
        "auto",
        "jpg",
        142,
        180,
        300,
        380,
    );
}

#[test]
fn resize_image_fill1() {
    image_op_test(
        "jpg.jpg",
        "375bbe63f375f8d100",
        "fill",
        Some(100),
        Some(200),
        "auto",
        "jpg",
        100,
        200,
        300,
        380,
    );
}

#[test]
fn resize_image_fill2() {
    image_op_test(
        "jpg.jpg",
        "17d0d5cc1f47663500",
        "fill",
        Some(200),
        Some(100),
        "auto",
        "jpg",
        200,
        100,
        300,
        380,
    );
}

#[test]
fn resize_image_png_png() {
    image_op_test(
        "png.png",
        "1dd3dac068d36b3900",
        "scale",
        Some(150),
        Some(150),
        "auto",
        "png",
        150,
        150,
        300,
        380,
    );
}

#[test]
fn resize_image_png_jpg() {
    image_op_test(
        "png.png",
        "818d7df655ab2e0c00",
        "scale",
        Some(150),
        Some(150),
        "jpg",
        "jpg",
        150,
        150,
        300,
        380,
    );
}

#[test]
fn resize_image_png_webp() {
    image_op_test(
        "png.png",
        "ec478d2823b16d5d00",
        "scale",
        Some(150),
        Some(150),
        "webp",
        "webp",
        150,
        150,
        300,
        380,
    );
}

#[test]
fn resize_image_webp_jpg() {
    image_op_test(
        "webp.webp",
        "a0591a71e1cd9ffd00",
        "scale",
        Some(150),
        Some(150),
        "auto",
        "jpg",
        150,
        150,
        300,
        380,
    );
}

#[test]
fn read_image_metadata_jpg() {
    assert_eq!(
        image_meta_test("jpg.jpg"),
        ImageMetaResponse { width: 300, height: 380, format: Some("jpg") }
    );
}

#[test]
fn read_image_metadata_png() {
    assert_eq!(
        image_meta_test("png.png"),
        ImageMetaResponse { width: 300, height: 380, format: Some("png") }
    );
}

#[test]
fn read_image_metadata_svg() {
    assert_eq!(
        image_meta_test("svg.svg"),
        ImageMetaResponse { width: 300, height: 300, format: Some("svg") }
    );
}

#[test]
fn read_image_metadata_webp() {
    assert_eq!(
        image_meta_test("webp.webp"),
        ImageMetaResponse { width: 300, height: 380, format: Some("webp") }
    );
}

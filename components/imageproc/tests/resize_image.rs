use std::env;
use std::path::{PathBuf, MAIN_SEPARATOR as SLASH};

use config::Config;
use imageproc::{assert_processed_path_matches, fix_orientation, ImageMetaResponse, Processor};
use libs::image::{self, DynamicImage, GenericImageView, Pixel};
use libs::once_cell::sync::Lazy;

static CONFIG: &str = r#"
title = "imageproc integration tests"
base_url = "https://example.com"
compile_sass = false
build_search_index = false

[markdown]
highlight_code = false
"#;

static TEST_IMGS: Lazy<PathBuf> =
    Lazy::new(|| [env!("CARGO_MANIFEST_DIR"), "tests", "test_imgs"].iter().collect());
static PROCESSED_PREFIX: Lazy<String> =
    Lazy::new(|| format!("static{0}processed_images{0}", SLASH));

#[allow(clippy::too_many_arguments)]
fn image_op_test(
    source_img: &str,
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
    let tmpdir = tempfile::tempdir().unwrap().into_path();
    let config = Config::parse(CONFIG).unwrap();
    let mut proc = Processor::new(tmpdir.clone(), &config);

    let resp =
        proc.enqueue(source_img.into(), source_path, op, width, height, format, None).unwrap();
    assert_processed_path_matches(&resp.url, "https://example.com/processed_images/", expect_ext);
    assert_processed_path_matches(&resp.static_path, PROCESSED_PREFIX.as_str(), expect_ext);
    assert_eq!(resp.width, expect_width);
    assert_eq!(resp.height, expect_height);
    assert_eq!(resp.orig_width, orig_width);
    assert_eq!(resp.orig_height, orig_height);

    proc.do_process().unwrap();

    let processed_path = PathBuf::from(&resp.static_path);
    let processed_size = imageproc::read_image_metadata(&tmpdir.join(processed_path))
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
    image_op_test("jpg.jpg", "scale", Some(150), Some(150), "auto", "jpg", 150, 150, 300, 380);
}

#[test]
fn resize_image_fit_width() {
    image_op_test("jpg.jpg", "fit_width", Some(150), None, "auto", "jpg", 150, 190, 300, 380);
}

#[test]
fn resize_image_fit_height() {
    image_op_test("webp.webp", "fit_height", None, Some(190), "auto", "jpg", 150, 190, 300, 380);
}

#[test]
fn resize_image_fit1() {
    image_op_test("jpg.jpg", "fit", Some(150), Some(200), "auto", "jpg", 150, 190, 300, 380);
}

#[test]
fn resize_image_fit2() {
    image_op_test("jpg.jpg", "fit", Some(160), Some(180), "auto", "jpg", 142, 180, 300, 380);
}

#[test]
fn resize_image_fit3() {
    image_op_test("jpg.jpg", "fit", Some(400), Some(400), "auto", "jpg", 300, 380, 300, 380);
}

#[test]
fn resize_image_fill1() {
    image_op_test("jpg.jpg", "fill", Some(100), Some(200), "auto", "jpg", 100, 200, 300, 380);
}

#[test]
fn resize_image_fill2() {
    image_op_test("jpg.jpg", "fill", Some(200), Some(100), "auto", "jpg", 200, 100, 300, 380);
}

#[test]
fn resize_image_png_png() {
    image_op_test("png.png", "scale", Some(150), Some(150), "auto", "png", 150, 150, 300, 380);
}

#[test]
fn resize_image_png_jpg() {
    image_op_test("png.png", "scale", Some(150), Some(150), "jpg", "jpg", 150, 150, 300, 380);
}

#[test]
fn resize_image_png_webp() {
    image_op_test("png.png", "scale", Some(150), Some(150), "webp", "webp", 150, 150, 300, 380);
}

#[test]
fn resize_image_webp_jpg() {
    image_op_test("webp.webp", "scale", Some(150), Some(150), "auto", "jpg", 150, 150, 300, 380);
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

#[test]
fn fix_orientation_test() {
    fn load_img_and_fix_orientation(img_name: &str) -> DynamicImage {
        let path = TEST_IMGS.join(img_name);
        let img = image::open(&path).unwrap();
        fix_orientation(&img, &path).unwrap_or(img)
    }

    let img = image::open(TEST_IMGS.join("exif_1.jpg")).unwrap();
    assert!(check_img(img));
    assert!(check_img(load_img_and_fix_orientation("exif_0.jpg")));
    assert!(check_img(load_img_and_fix_orientation("exif_1.jpg")));
    assert!(check_img(load_img_and_fix_orientation("exif_2.jpg")));
    assert!(check_img(load_img_and_fix_orientation("exif_3.jpg")));
    assert!(check_img(load_img_and_fix_orientation("exif_4.jpg")));
    assert!(check_img(load_img_and_fix_orientation("exif_5.jpg")));
    assert!(check_img(load_img_and_fix_orientation("exif_6.jpg")));
    assert!(check_img(load_img_and_fix_orientation("exif_7.jpg")));
    assert!(check_img(load_img_and_fix_orientation("exif_8.jpg")));
}

#[test]
fn resize_image_applies_exif_rotation() {
    // No exif metadata
    assert!(resize_and_check("exif_0.jpg"));
    // 1: Horizontal (normal)
    assert!(resize_and_check("exif_1.jpg"));
    // 2: Mirror horizontal
    assert!(resize_and_check("exif_2.jpg"));
    // 3: Rotate 180
    assert!(resize_and_check("exif_3.jpg"));
    // 4: Mirror vertical
    assert!(resize_and_check("exif_4.jpg"));
    // 5: Mirror horizontal and rotate 270 CW
    assert!(resize_and_check("exif_5.jpg"));
    // 6: Rotate 90 CW
    assert!(resize_and_check("exif_6.jpg"));
    // 7: Mirror horizontal and rotate 90 CW
    assert!(resize_and_check("exif_7.jpg"));
    // 8: Rotate 270 CW
    assert!(resize_and_check("exif_8.jpg"));
}

fn resize_and_check(source_img: &str) -> bool {
    let source_path = TEST_IMGS.join(source_img);
    let tmpdir = tempfile::tempdir().unwrap().into_path();
    let config = Config::parse(CONFIG).unwrap();
    let mut proc = Processor::new(tmpdir.clone(), &config);

    let resp = proc
        .enqueue(source_img.into(), source_path, "scale", Some(16), Some(16), "jpg", None)
        .unwrap();

    proc.do_process().unwrap();
    let processed_path = PathBuf::from(&resp.static_path);
    let img = image::open(&tmpdir.join(processed_path)).unwrap();
    check_img(img)
}

// Checks that an image has the correct orientation
fn check_img(img: DynamicImage) -> bool {
    // top left is red
    img.get_pixel(0, 0)[0] > 250 // because of the jpeg compression some colors are a bit less than 255
    // top right is green
        && img.get_pixel(15, 0)[1] > 250
    // bottom left is blue
        && img.get_pixel(0, 15)[2] > 250
    // bottom right is white
        && img.get_pixel(15, 15).channels() == [255, 255, 255, 255]
}

// TODO: Test that hash remains the same if physical path is changed

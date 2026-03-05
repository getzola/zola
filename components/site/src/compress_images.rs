use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use aho_corasick::{AhoCorasick, MatchKind};
use config::ImageCompression;
use errors::Result;
use perceptual_image::{
    PerceptualCompressor,
    encoders::{PerceptualAVIFEncoder, PerceptualJpegEncoder, PerceptualWebPEncoder},
};
use walkdir::WalkDir;

pub fn compress_images(
    static_path: &Path,
    output_path: &Path,
    globs: &Vec<ImageCompression>,
) -> Result<()> {
    // Setup paths
    let compressed_path = {
        let mut compressed_path = PathBuf::from(static_path);
        compressed_path.push("compressed_images");
        compressed_path
    };

    // Setup record of compressed files
    let mut old_files = Vec::new();
    let mut new_files = Vec::new();

    for item in globs {
        let glob = globset::GlobBuilder::new(&item.glob).build()?.compile_matcher();
        for entry in WalkDir::new(&static_path)
            .into_iter()
            .filter_entry(|e| !e.path().starts_with(&compressed_path))
        {
            let entry = entry?;
            let input_path = entry.path();

            // Search for matches to glob and compress
            if glob.is_match(entry.path()) {
                let mut output_path =
                    compressed_path.join(entry.path().strip_prefix(&static_path)?);
                output_path.set_extension(item.format.file_extension());

                fs::create_dir_all(&output_path.parent().unwrap())?;
                old_files.push(input_path.strip_prefix(&static_path)?.display().to_string());
                new_files.push(output_path.strip_prefix(&static_path)?.display().to_string());

                // Only run compression if output file does not already exist
                if !Path::exists(&output_path) {
                    let file = File::create(output_path.clone())?;
                    let source = image::open(entry.path())?;
                    let compressor = PerceptualCompressor::new(&source)
                        .max_iterations(item.max_iterations as usize)
                        .target_score(item.target_ssim);

                    match item.format {
                        config::ImageFormat::Avif => {
                            let encoder = PerceptualAVIFEncoder::new();
                            compressor.encode(file, encoder)?;
                        }
                        config::ImageFormat::Jpeg => {
                            let encoder = PerceptualJpegEncoder::new();
                            compressor.encode(file, encoder)?;
                        }
                        config::ImageFormat::Webp => {
                            let encoder = PerceptualWebPEncoder::new();
                            compressor.encode(file, encoder)?;
                        }
                    }
                }
            }
        }
    }

    // Clean any orphaned files from compressed_images.
    for entry in WalkDir::new(compressed_path.clone()) {
        let entry = entry?;
        let comp_path = entry.path().strip_prefix(&static_path)?.display().to_string();

        if !entry.path().is_dir() && new_files.iter().find(|x| *x == &comp_path).is_none() {
            fs::remove_file(entry.path())?;
        }
    }

    // Finally, rewrite references to file in output html.
    let glob = globset::GlobBuilder::new("**/*.html").build()?.compile_matcher();
    for entry in WalkDir::new(output_path) {
        let entry = entry?;
        if glob.is_match(entry.path()) {
            let contents = fs::read_to_string(entry.path())?;

            // Find any references to replaced files.
            let new_contents = AhoCorasick::builder()
                .match_kind(MatchKind::LeftmostFirst)
                .build(&old_files)?
                .replace_all(&contents, &new_files);

            fs::write(entry.path(), new_contents)?;
        }
    }

    Ok(())
}

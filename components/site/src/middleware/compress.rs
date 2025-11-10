use config::Compression;
use errors::Result;

use super::{ContentType, Middleware, Output, OutputData, OutputPackage, OutputTags};

const MIN_COMPRESSION_SIZE: usize = 512;

#[cfg(feature = "gzip")]
const GZIP_COMPRESSION_LEVEL: u32 = 9;

#[cfg(feature = "brotli")]
const BROTLI_COMPRESSION_LEVEL: u32 = 11;

/// Middleware that compresses content using gzip and/or brotli
pub struct CompressionMiddleware {
    algorithms: Vec<Compression>,
}

impl CompressionMiddleware {
    pub fn new(algorithms: Vec<Compression>) -> Self {
        Self { algorithms }
    }

    /// Check if content type should be compressed
    fn should_compress(&self, content_type: &ContentType) -> bool {
        matches!(
            content_type,
            ContentType::Html | ContentType::Xml | ContentType::Json | ContentType::Text
        )
    }

    /// Compress using gzip
    #[cfg(feature = "gzip")]
    fn compress_gzip(&self, content: &[u8]) -> Result<Vec<u8>> {
        use flate2::Compression as GzipCompression;
        use flate2::write::GzEncoder;
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), GzipCompression::new(GZIP_COMPRESSION_LEVEL));
        encoder.write_all(content)?;
        Ok(encoder.finish()?)
    }

    /// Compress using brotli
    #[cfg(feature = "brotli")]
    fn compress_brotli(&self, content: &[u8]) -> Result<Vec<u8>> {
        use std::io::Write;

        let mut output = Vec::new();
        let mut writer = brotli::CompressorWriter::new(
            &mut output,
            4096, // buffer size
            BROTLI_COMPRESSION_LEVEL,
            22, // window size (default)
        );
        writer.write_all(content)?;
        drop(writer); // Ensure writer is flushed
        Ok(output)
    }
}

impl Middleware for CompressionMiddleware {
    #[allow(unreachable_code, unused_variables)]
    fn process(&self, package: &mut OutputPackage) -> Result<()> {
        // Collect outputs to compress (can't modify map while iterating)
        let to_compress: Vec<_> = package
            .outputs
            .iter()
            .filter(|entry| {
                let output = entry.value();
                !output.tags.is_compressed
                    && self.should_compress(&output.content_type)
                    && output.data.as_bytes().len() >= MIN_COMPRESSION_SIZE
            })
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();

        // Add compressed versions for each compressible output
        for (key, output) in to_compress {
            let bytes = output.data.as_bytes();

            // Iterate over all configured compression algorithms
            for algorithm in &self.algorithms {
                match algorithm {
                    Compression::Gzip => {
                        #[cfg(feature = "gzip")]
                        {
                            let compressed = self.compress_gzip(&bytes)?;
                            let gz_filename = format!("{}.gz", key.filename);

                            package.add_at(
                                key.components.clone(),
                                gz_filename,
                                Output {
                                    data: OutputData::Binary(compressed),
                                    content_type: output.content_type.clone(),
                                    tags: OutputTags {
                                        is_compressed: true,
                                        is_derived: true,
                                        ..Default::default()
                                    },
                                },
                            );
                        }
                        #[cfg(not(feature = "gzip"))]
                        {
                            return Err(errors::anyhow!(
                                "Gzip compression is enabled in config but Zola was not compiled with the 'gzip' feature. \
                                Please rebuild with: cargo build --features gzip"
                            ));
                        }
                    }
                    Compression::Brotli => {
                        #[cfg(feature = "brotli")]
                        {
                            let compressed = self.compress_brotli(&bytes)?;
                            let br_filename = format!("{}.br", key.filename);

                            package.add_at(
                                key.components.clone(),
                                br_filename,
                                Output {
                                    data: OutputData::Binary(compressed),
                                    content_type: output.content_type.clone(),
                                    tags: OutputTags {
                                        is_compressed: true,
                                        is_derived: true,
                                        ..Default::default()
                                    },
                                },
                            );
                        }
                        #[cfg(not(feature = "brotli"))]
                        {
                            return Err(errors::anyhow!(
                                "Brotli compression is enabled in config but Zola was not compiled with the 'brotli' feature. \
                                Please rebuild with: cargo build --features brotli"
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "compression"
    }
}

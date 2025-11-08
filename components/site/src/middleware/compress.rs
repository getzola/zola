use config::Compression;
use errors::Result;

use super::{ContentType, Middleware, MiddlewareContext};

const MIN_COMPRESSION_SIZE: usize = 512;

#[cfg(feature = "gzip")]
const GZIP_COMPRESSION_LEVEL: u32 = 9;

#[cfg(feature = "brotli")]
const BROTLI_COMPRESSION_LEVEL: u32 = 11;

/// Middleware that compresses content using gzip or brotli
pub struct CompressionMiddleware {
    algorithm: Compression,
}

impl CompressionMiddleware {
    pub fn new(algorithm: Compression) -> Self {
        Self { algorithm }
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
    fn compress_gzip(&self, content: &str) -> Result<Vec<u8>> {
        use flate2::Compression as GzipCompression;
        use flate2::write::GzEncoder;
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), GzipCompression::new(GZIP_COMPRESSION_LEVEL));
        encoder.write_all(content.as_bytes())?;
        Ok(encoder.finish()?)
    }

    /// Compress using brotli
    #[cfg(feature = "brotli")]
    fn compress_brotli(&self, content: &str) -> Result<Vec<u8>> {
        use std::io::Write;

        let mut output = Vec::new();
        let mut writer = brotli::CompressorWriter::new(
            &mut output,
            4096, // buffer size
            BROTLI_COMPRESSION_LEVEL,
            22, // window size (default)
        );
        writer.write_all(content.as_bytes())?;
        drop(writer); // Ensure writer is flushed
        Ok(output)
    }
}

impl Middleware for CompressionMiddleware {
    #[allow(unreachable_code, unused_variables)]
    fn process(&self, ctx: &mut MiddlewareContext) -> Result<()> {
        // Check if content type should be compressed
        if !self.should_compress(&ctx.metadata.content_type) {
            return Ok(());
        }

        // Check minimum size threshold
        if ctx.content.len() < MIN_COMPRESSION_SIZE {
            return Ok(());
        }

        // Compress based on algorithm
        let (compressed_data, extension) = match self.algorithm {
            #[cfg(feature = "gzip")]
            Compression::Gzip => {
                let compressed = self.compress_gzip(&ctx.content)?;
                (compressed, ".gz".to_string())
            }
            #[cfg(not(feature = "gzip"))]
            Compression::Gzip => {
                return Err(errors::anyhow!(
                    "Gzip compression is enabled in config but Zola was not compiled with the 'gzip' feature. \
                    Please rebuild with: cargo build --features gzip"
                ));
            }

            #[cfg(feature = "brotli")]
            Compression::Brotli => {
                let compressed = self.compress_brotli(&ctx.content)?;
                (compressed, ".br".to_string())
            }
            #[cfg(not(feature = "brotli"))]
            Compression::Brotli => {
                return Err(errors::anyhow!(
                    "Brotli compression is enabled in config but Zola was not compiled with the 'brotli' feature. \
                    Please rebuild with: cargo build --features brotli"
                ));
            }
        };

        // Store compressed data in context
        ctx.binary_content = Some(compressed_data);
        ctx.compressed_extension = Some(extension);

        Ok(())
    }

    fn name(&self) -> &str {
        match self.algorithm {
            Compression::Gzip => "gzip",
            Compression::Brotli => "brotli",
        }
    }
}

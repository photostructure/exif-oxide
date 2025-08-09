
// Re-export tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::path::Path;

    #[test]
    fn test_extension_normalization() {
        let detector = FileTypeDetector::new();

        assert_eq!(normalize_extension("tif"), "TIFF");
        assert_eq!(normalize_extension("jpg"), "JPEG");
        assert_eq!(normalize_extension("png"), "PNG");
    }

    #[test]
    fn test_jpeg_detection() {
        let detector = FileTypeDetector::new();
        let path = Path::new("test.jpg");

        // JPEG magic number: \xff\xd8\xff
        let jpeg_data = vec![0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10];
        let mut cursor = Cursor::new(jpeg_data);

        let result = detector.detect_file_type(path, &mut cursor).unwrap();
        assert_eq!(result.file_type, "JPEG");
        assert_eq!(result.format, "JPEG");
        assert_eq!(result.mime_type, "image/jpeg");
    }

    #[test]
    fn test_png_detection() {
        let detector = FileTypeDetector::new();
        let path = Path::new("test.png");

        // PNG magic number: \x89PNG\r\n\x1a\n
        let png_data = vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
        let mut cursor = Cursor::new(png_data);

        let result = detector.detect_file_type(path, &mut cursor).unwrap();
        assert_eq!(result.file_type, "PNG");
        assert_eq!(result.format, "PNG");
        assert_eq!(result.mime_type, "image/png");
    }

    #[test]
    fn test_tiff_extension_alias() {
        let detector = FileTypeDetector::new();
        let path = Path::new("test.tif");

        // TIFF magic number: II*\0 (little endian)
        let tiff_data = vec![0x49, 0x49, 0x2a, 0x00];
        let mut cursor = Cursor::new(tiff_data);

        let result = detector.detect_file_type(path, &mut cursor).unwrap();
        assert_eq!(result.file_type, "TIFF");
        assert_eq!(result.format, "TIFF");
        assert_eq!(result.mime_type, "image/tiff");
    }

    #[test]
    fn test_unknown_extension_fallback_to_magic() {
        // Test that unknown extensions correctly fall back to magic number detection
        // This ensures we follow ExifTool's behavior: unknown extensions trigger
        // comprehensive magic number scanning instead of being treated as file types

        let detector = FileTypeDetector::new();
        let path = Path::new("unknown.xyz"); // Completely unknown extension

        // Data with JPEG magic signature
        let mut data = vec![0x00, 0x01, 0x02, 0x03]; // Unknown header
        data.extend_from_slice(&[0xff, 0xd8, 0xff]); // JPEG signature
        let mut cursor = Cursor::new(data);

        let result = detector.detect_file_type(path, &mut cursor).unwrap();

        // Should detect JPEG via magic number, not treat "XYZ" as a file type
        assert_eq!(result.file_type, "JPEG");
        assert_eq!(result.mime_type, "image/jpeg");
    }

    #[test]
    fn test_embedded_jpeg_recovery() {
        let detector = FileTypeDetector::new();
        // Use a filename with unknown extension to trigger embedded signature scan
        let path = Path::new("unknown.xyz");

        // Unknown header followed by JPEG signature
        let mut data = vec![0x00, 0x01, 0x02, 0x03]; // Unknown header
        data.extend_from_slice(&[0xff, 0xd8, 0xff]); // JPEG signature
        let mut cursor = Cursor::new(data);

        let result = detector.detect_file_type(path, &mut cursor).unwrap();
        assert_eq!(result.file_type, "JPEG");
    }

    #[test]
    fn test_weak_magic_mp3() {
        let detector = FileTypeDetector::new();
        let path = Path::new("test.mp3");

        // MP3 has weak magic, should rely on extension
        let mp3_data = vec![0x49, 0x44, 0x33]; // ID3 tag (valid MP3 start)
        let mut cursor = Cursor::new(mp3_data);

        let result = detector.detect_file_type(path, &mut cursor).unwrap();
        assert_eq!(result.file_type, "MP3");
        assert_eq!(result.mime_type, "audio/mpeg");
    }

    #[test]
    fn test_unknown_file_type() {
        let detector = FileTypeDetector::new();
        let path = Path::new("test.unknown");

        let unknown_data = vec![0x00, 0x01, 0x02, 0x03];
        let mut cursor = Cursor::new(unknown_data);

        let result = detector.detect_file_type(path, &mut cursor);
        assert!(matches!(result, Err(FileDetectionError::UnknownFileType)));
    }

    #[test]
    fn test_heic_detection() {
        let detector = FileTypeDetector::new();
        let path = Path::new("test.heic");

        // HEIC file with ftyp box and heic brand
        let mut heic_data = Vec::new();
        heic_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x28]); // Box size (40 bytes)
        heic_data.extend_from_slice(b"ftyp"); // Box type (bytes 4-7)
        heic_data.extend_from_slice(b"heic"); // Major brand (bytes 8-11)
        heic_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // Minor version
        heic_data.extend_from_slice(b"mif1"); // Compatible brand
        heic_data.extend_from_slice(b"MiHE"); // Compatible brand
        heic_data.extend_from_slice(b"MiPr"); // Compatible brand
        heic_data.extend_from_slice(b"miaf"); // Compatible brand
        heic_data.extend_from_slice(b"MiHB"); // Compatible brand
        heic_data.extend_from_slice(b"heic"); // Compatible brand

        let mut cursor = Cursor::new(heic_data);

        let result = detector.detect_file_type(path, &mut cursor).unwrap();
        assert_eq!(result.file_type, "HEIC");
        assert_eq!(result.format, "MOV");
        assert_eq!(result.mime_type, "image/heic");
        assert_eq!(
            result.description,
            "High Efficiency Image Format still image"
        );
    }

    #[test]
    fn test_avi_riff_detection() {
        let detector = FileTypeDetector::new();
        let path = Path::new("test.avi");

        // AVI RIFF header: RIFF + size + "AVI " format identifier
        let mut avi_data = Vec::new();
        avi_data.extend_from_slice(b"RIFF"); // RIFF magic (bytes 0-3)
        avi_data.extend_from_slice(&[0x24, 0x00, 0x00, 0x00]); // File size - 8 (bytes 4-7)
        avi_data.extend_from_slice(b"AVI "); // AVI format identifier (bytes 8-11)
        avi_data.extend_from_slice(b"LIST"); // Chunk header start (bytes 12+)
        let mut cursor = Cursor::new(avi_data);

        let result = detector.detect_file_type(path, &mut cursor).unwrap();
        assert_eq!(result.file_type, "AVI");
        assert_eq!(result.format, "RIFF");
        assert_eq!(result.mime_type, "video/x-msvideo");
    }

    #[test]
    fn test_wav_riff_detection() {
        let detector = FileTypeDetector::new();
        let path = Path::new("test.wav");

        // WAV RIFF header: RIFF + size + "WAVE" format identifier
        let mut wav_data = Vec::new();
        wav_data.extend_from_slice(b"RIFF"); // RIFF magic (bytes 0-3)
        wav_data.extend_from_slice(&[0x24, 0x00, 0x00, 0x00]); // File size - 8 (bytes 4-7)
        wav_data.extend_from_slice(b"WAVE"); // WAVE format identifier (bytes 8-11)
        wav_data.extend_from_slice(b"fmt "); // Format chunk start (bytes 12+)
        let mut cursor = Cursor::new(wav_data);

        let result = detector.detect_file_type(path, &mut cursor).unwrap();
        assert_eq!(result.file_type, "WAV");
        assert_eq!(result.format, "RIFF");
        assert_eq!(result.mime_type, "audio/x-wav");
    }

    #[test]
    fn test_webp_riff_detection() {
        let detector = FileTypeDetector::new();
        let path = Path::new("test.webp");

        // WebP RIFF header: RIFF + size + "WEBP" format identifier
        let mut webp_data = Vec::new();
        webp_data.extend_from_slice(b"RIFF"); // RIFF magic (bytes 0-3)
        webp_data.extend_from_slice(&[0x20, 0x00, 0x00, 0x00]); // File size - 8 (bytes 4-7)
        webp_data.extend_from_slice(b"WEBP"); // WEBP format identifier (bytes 8-11)
        webp_data.extend_from_slice(b"VP8 "); // VP8 chunk header (bytes 12+)
        let mut cursor = Cursor::new(webp_data);

        let result = detector.detect_file_type(path, &mut cursor).unwrap();
        assert_eq!(result.file_type, "WEBP");
        assert_eq!(result.format, "RIFF");
        assert_eq!(result.mime_type, "image/webp");
    }

    #[test]
    fn test_heic_extension_detection() {
        let detector = FileTypeDetector::new();
        let path = Path::new("test.heic");

        // MOV file with HEIC ftyp brand
        let mut heic_data = Vec::new();
        heic_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x20]); // Size
        heic_data.extend_from_slice(b"ftyp"); // Box type
        heic_data.extend_from_slice(b"mif1"); // Major brand (HEIF)
        heic_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // Minor version
        heic_data.extend_from_slice(b"mif1heic"); // Compatible brands
        let mut cursor = Cursor::new(heic_data);

        match detector.detect_file_type(path, &mut cursor) {
            Ok(result) => {
                println!(
                    "HEIC detection result: file_type={}, format={}, mime_type={}",
                    result.file_type, result.format, result.mime_type
                );
                // Should detect as HEIF due to mif1 brand
                assert_eq!(result.file_type, "HEIF");
                assert_eq!(result.format, "MOV");
                assert_eq!(result.mime_type, "image/heif");
            }
            Err(e) => {
                panic!("Failed to detect HEIC file: {e:?}");
            }
        }
    }

    #[test]
    fn test_riff_format_content_detection() {
        let detector = FileTypeDetector::new();
        let path = Path::new("test.avi"); // AVI extension

        // But contains WAV data - should detect as WAV based on content
        // Following ExifTool's behavior: content takes precedence over extension
        let mut wav_data = Vec::new();
        wav_data.extend_from_slice(b"RIFF"); // RIFF magic
        wav_data.extend_from_slice(&[0x24, 0x00, 0x00, 0x00]); // File size
        wav_data.extend_from_slice(b"WAVE"); // WAVE format (not AVI)
        wav_data.extend_from_slice(b"fmt "); // Format chunk
        let mut cursor = Cursor::new(wav_data);

        // Should detect as WAV based on content, following ExifTool's behavior
        let result = detector.detect_file_type(path, &mut cursor);
        match result {
            Ok(detection) => {
                assert_eq!(detection.file_type, "WAV");
                assert_eq!(detection.format, "RIFF");
                assert_eq!(detection.mime_type, "audio/x-wav");
            }
            Err(e) => {
                panic!("Expected WAV detection but got error: {e:?}");
            }
        }
    }
}

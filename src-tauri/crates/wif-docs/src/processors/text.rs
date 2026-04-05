use std::path::Path;

use crate::{DocsError, DocumentMetadata, DocumentProcessor};

/// Processes plain `.txt` files using only the standard library.
pub struct TextProcessor;

impl DocumentProcessor for TextProcessor {
    fn extract_text(&self, path: &Path) -> Result<String, DocsError> {
        std::fs::read_to_string(path).map_err(DocsError::Io)
    }

    fn extract_metadata(&self, path: &Path) -> Result<DocumentMetadata, DocsError> {
        let fs_meta = std::fs::metadata(path)?;
        let content = std::fs::read_to_string(path)?;
        let line_count = content.lines().count();

        Ok(DocumentMetadata {
            file_name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            file_size: fs_meta.len(),
            content_type: "text/plain".to_string(),
            page_count: Some(line_count as u32),
            author: None,
            created_date: None,
            modified_date: None,
            title: None,
        })
    }

    fn supports(&self, extension: &str) -> bool {
        extension.eq_ignore_ascii_case("txt")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_txt_case_insensitive() {
        let p = TextProcessor;
        assert!(p.supports("txt"));
        assert!(p.supports("TXT"));
        assert!(!p.supports("csv"));
        assert!(!p.supports("xlsx"));
    }

    #[test]
    fn extract_text_reads_file_contents() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(tmp, "Hello, world!").unwrap();

        let p = TextProcessor;
        let text = p.extract_text(tmp.path()).unwrap();
        assert!(text.contains("Hello, world!"));
    }

    #[test]
    fn extract_metadata_has_correct_content_type() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(tmp, "line1\nline2").unwrap();

        let p = TextProcessor;
        let meta = p.extract_metadata(tmp.path()).unwrap();
        assert_eq!(meta.content_type, "text/plain");
        assert!(meta.page_count.unwrap() >= 2);
    }
}

use std::path::Path;

use crate::{DocsError, DocumentMetadata, DocumentProcessor};

/// Processes plain `.csv` files using only the standard library.
pub struct CsvProcessor;

impl DocumentProcessor for CsvProcessor {
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
            content_type: "text/csv".to_string(),
            page_count: Some(line_count as u32),
            author: None,
            created_date: None,
            modified_date: None,
            title: None,
        })
    }

    fn supports(&self, extension: &str) -> bool {
        extension.eq_ignore_ascii_case("csv")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_csv_case_insensitive() {
        let p = CsvProcessor;
        assert!(p.supports("csv"));
        assert!(p.supports("CSV"));
        assert!(!p.supports("xlsx"));
        assert!(!p.supports("txt"));
    }

    #[test]
    fn extract_text_reads_file_contents() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(tmp, "a,b,c\n1,2,3").unwrap();

        let p = CsvProcessor;
        let text = p.extract_text(tmp.path()).unwrap();
        assert!(text.contains("a,b,c"));
    }

    #[test]
    fn extract_metadata_counts_lines() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(tmp, "header\nrow1\nrow2").unwrap();

        let p = CsvProcessor;
        let meta = p.extract_metadata(tmp.path()).unwrap();
        assert_eq!(meta.content_type, "text/csv");
        assert!(meta.page_count.unwrap() >= 3);
    }
}

use std::path::Path;

use calamine::{open_workbook, Data, Reader, Xlsx};

use crate::{DocsError, DocumentMetadata, DocumentProcessor};

/// Processes `.xlsx` and `.xls` spreadsheet files via the calamine crate.
pub struct XlsxProcessor;

impl DocumentProcessor for XlsxProcessor {
    fn extract_text(&self, path: &Path) -> Result<String, DocsError> {
        let mut workbook: Xlsx<_> = open_workbook(path)
            .map_err(|e| DocsError::ParseError(format!("{e}")))?;

        let sheet_names = workbook.sheet_names().to_vec();
        let mut text = String::new();

        for sheet_name in &sheet_names {
            if let Ok(range) = workbook.worksheet_range(sheet_name) {
                text.push_str(&format!("=== {} ===\n", sheet_name));
                for row in range.rows() {
                    let cells: Vec<String> = row
                        .iter()
                        .map(|c| match c {
                            Data::String(s) => s.clone(),
                            Data::Float(f) => f.to_string(),
                            Data::Int(i) => i.to_string(),
                            Data::Bool(b) => b.to_string(),
                            _ => String::new(),
                        })
                        .collect();
                    text.push_str(&cells.join("\t"));
                    text.push('\n');
                }
            }
        }

        Ok(text)
    }

    fn extract_metadata(&self, path: &Path) -> Result<DocumentMetadata, DocsError> {
        let fs_meta = std::fs::metadata(path)?;
        let workbook: Xlsx<_> = open_workbook(path)
            .map_err(|e| DocsError::ParseError(format!("{e}")))?;

        Ok(DocumentMetadata {
            file_name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            file_size: fs_meta.len(),
            content_type:
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(),
            page_count: Some(workbook.sheet_names().len() as u32),
            author: None,
            created_date: None,
            modified_date: None,
            title: None,
        })
    }

    fn supports(&self, extension: &str) -> bool {
        matches!(extension.to_lowercase().as_str(), "xlsx" | "xls")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_xlsx_and_xls() {
        let p = XlsxProcessor;
        assert!(p.supports("xlsx"));
        assert!(p.supports("xls"));
        assert!(p.supports("XLSX"));
        assert!(!p.supports("csv"));
        assert!(!p.supports("docx"));
    }
}

use std::{io::Write, path::Path};

use crate::{DocsError, DocumentExporter};

/// Exports tabular data to a plain `.csv` file.
///
/// Values that contain commas, quotes, or newlines are quoted per RFC 4180.
pub struct CsvExporter;

impl DocumentExporter for CsvExporter {
    fn export(
        &self,
        path: &Path,
        headers: &[String],
        rows: &[Vec<String>],
    ) -> Result<(), DocsError> {
        let mut file = std::fs::File::create(path)
            .map_err(|e| DocsError::ExportError(format!("cannot create file: {e}")))?;

        writeln!(file, "{}", encode_row(headers))
            .map_err(|e| DocsError::ExportError(format!("write error: {e}")))?;

        for row in rows {
            writeln!(file, "{}", encode_row(row))
                .map_err(|e| DocsError::ExportError(format!("write error: {e}")))?;
        }

        Ok(())
    }

    fn format(&self) -> &str {
        "csv"
    }
}

/// Encode a single row as a comma-separated line.
///
/// Fields containing `,`, `"`, or newline characters are wrapped in double
/// quotes, and any embedded double-quote characters are escaped as `""`.
fn encode_row(fields: &[String]) -> String {
    fields
        .iter()
        .map(|f| {
            if f.contains(',') || f.contains('"') || f.contains('\n') || f.contains('\r') {
                format!("\"{}\"", f.replace('"', "\"\""))
            } else {
                f.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_row_simple() {
        let row = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert_eq!(encode_row(&row), "a,b,c");
    }

    #[test]
    fn encode_row_quotes_fields_with_commas() {
        let row = vec!["a,b".to_string(), "c".to_string()];
        assert_eq!(encode_row(&row), "\"a,b\",c");
    }

    #[test]
    fn encode_row_escapes_embedded_quotes() {
        let row = vec!["say \"hi\"".to_string()];
        assert_eq!(encode_row(&row), "\"say \"\"hi\"\"\"");
    }

    #[test]
    fn export_creates_valid_csv() {
        let exporter = CsvExporter;
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let headers = vec!["name".to_string(), "age".to_string()];
        let rows = vec![
            vec!["Alice".to_string(), "30".to_string()],
            vec!["Bob".to_string(), "25".to_string()],
        ];

        exporter.export(tmp.path(), &headers, &rows).unwrap();

        let content = std::fs::read_to_string(tmp.path()).unwrap();
        assert!(content.contains("name,age"));
        assert!(content.contains("Alice,30"));
        assert!(content.contains("Bob,25"));
    }

    #[test]
    fn format_returns_csv() {
        assert_eq!(CsvExporter.format(), "csv");
    }
}

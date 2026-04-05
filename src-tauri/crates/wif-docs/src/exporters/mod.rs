pub mod csv;

use crate::DocumentExporter;

pub use csv::CsvExporter;

/// Return the exporter for the requested format name (case-insensitive),
/// or `None` if no exporter is registered for that format.
pub fn get_exporter(format: &str) -> Option<Box<dyn DocumentExporter>> {
    match format.to_lowercase().as_str() {
        "csv" => Some(Box::new(CsvExporter)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_exporter_for_csv() {
        assert!(get_exporter("csv").is_some());
        assert!(get_exporter("CSV").is_some());
    }

    #[test]
    fn returns_none_for_unknown_format() {
        assert!(get_exporter("xlsx").is_none());
        assert!(get_exporter("pdf").is_none());
        assert!(get_exporter("").is_none());
    }
}

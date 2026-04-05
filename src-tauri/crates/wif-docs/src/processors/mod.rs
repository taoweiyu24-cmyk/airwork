pub mod csv;
pub mod text;
pub mod xlsx;

use crate::DocumentProcessor;

pub use csv::CsvProcessor;
pub use text::TextProcessor;
pub use xlsx::XlsxProcessor;

/// Return the first processor that supports the given file extension,
/// or `None` if no processor is registered for that extension.
///
/// The extension should be provided without a leading dot, e.g. `"xlsx"`.
pub fn get_processor(extension: &str) -> Option<Box<dyn DocumentProcessor>> {
    let processors: Vec<Box<dyn DocumentProcessor>> = vec![
        Box::new(XlsxProcessor),
        Box::new(CsvProcessor),
        Box::new(TextProcessor),
    ];
    processors.into_iter().find(|p| p.supports(extension))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_processor_for_known_extensions() {
        assert!(get_processor("xlsx").is_some());
        assert!(get_processor("xls").is_some());
        assert!(get_processor("csv").is_some());
        assert!(get_processor("txt").is_some());
    }

    #[test]
    fn returns_none_for_unknown_extension() {
        assert!(get_processor("pdf").is_none());
        assert!(get_processor("docx").is_none());
        assert!(get_processor("pptx").is_none());
        assert!(get_processor("").is_none());
    }
}

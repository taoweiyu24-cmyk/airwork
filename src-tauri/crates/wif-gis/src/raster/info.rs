use serde::{Deserialize, Serialize};

/// Raster dataset metadata.
///
/// This is a stub — full raster support requires the GDAL crate which in turn
/// requires a system-level GDAL installation. Use this type to represent
/// metadata that has already been extracted by another means.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RasterInfo {
    pub file_path: String,
    pub width: u32,
    pub height: u32,
    pub band_count: u32,
    pub crs: Option<String>,
    /// Bounding box as `((min_lon, min_lat), (max_lon, max_lat))`.
    pub bounds: Option<((f64, f64), (f64, f64))>,
}

/// Attempt to read raster metadata from a file.
///
/// This always returns [`GisError::UnsupportedFormat`] until a GDAL integration
/// is added. The function signature is stable so callers can depend on it now.
pub fn read_raster_info(_path: &std::path::Path) -> Result<RasterInfo, crate::error::GisError> {
    Err(crate::error::GisError::UnsupportedFormat(
        "Raster support requires GDAL (TBD)".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn read_raster_info_returns_unsupported() {
        let result = read_raster_info(Path::new("/some/file.tif"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("GDAL"));
    }

    #[test]
    fn raster_info_serialises() {
        let info = RasterInfo {
            file_path: "/data/dem.tif".into(),
            width: 1024,
            height: 768,
            band_count: 1,
            crs: Some("EPSG:4326".into()),
            bounds: Some(((110.0, 20.0), (120.0, 30.0))),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("1024"));
        assert!(json.contains("EPSG:4326"));
    }
}

//! Vector data processing — GeoJSON I/O and spatial analysis.

pub mod analysis;
pub mod geojson_io;

pub use analysis::{
    bounding_rect, buffer_point, centroid, geometries_intersect, geometry_contains, polygon_area,
};
pub use geojson_io::{export_geojson, geometry_to_wkt, import_geojson, wkt_to_geometry};

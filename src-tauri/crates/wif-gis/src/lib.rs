//! `wif-gis` — GIS module for WorkItemFlow.
//!
//! Provides map tile sources, vector data I/O (GeoJSON), spatial analysis,
//! coordinate transforms, and measurement utilities.
//! Raster support is stubbed pending a GDAL integration.

pub mod coordinate;
pub mod error;
pub mod map;
pub mod measure;
pub mod raster;
pub mod vector;

// Flat re-exports for the most commonly used types and functions.

pub use error::GisError;

pub use map::{MapService, TileSource};

pub use vector::{
    bounding_rect, buffer_point, centroid, export_geojson, geometries_intersect,
    geometry_contains, geometry_to_wkt, import_geojson, polygon_area, wkt_to_geometry,
};

pub use coordinate::{
    deg_to_rad, haversine_distance, mercator_to_wgs84, rad_to_deg, wgs84_to_mercator,
};

pub use measure::{polygon_area_m2, polyline_length};

pub use raster::{read_raster_info, RasterInfo};

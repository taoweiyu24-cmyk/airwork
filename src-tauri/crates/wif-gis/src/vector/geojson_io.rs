use geojson::{Feature, FeatureCollection, GeoJson, Geometry, Value};
use std::path::Path;
use ulid::Ulid;
use wif_domain::GisFeature;

use crate::error::GisError;

/// Import a GeoJSON file and return `GisFeature` entities assigned to `layer_id`.
pub fn import_geojson(path: &Path, layer_id: Ulid) -> Result<Vec<GisFeature>, GisError> {
    let content = std::fs::read_to_string(path)
        .map_err(|_| GisError::FileNotFound(path.display().to_string()))?;

    let geojson: GeoJson = content
        .parse()
        .map_err(|e| GisError::InvalidGeometry(format!("{e}")))?;

    let features = match geojson {
        GeoJson::FeatureCollection(fc) => fc.features,
        GeoJson::Feature(f) => vec![f],
        GeoJson::Geometry(g) => vec![Feature {
            geometry: Some(g),
            ..Default::default()
        }],
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let result = features
        .iter()
        .filter_map(|f| {
            let geometry = f.geometry.as_ref()?;
            let wkt = geometry_to_wkt(geometry);
            let props = f
                .properties
                .as_ref()
                .map(|p| serde_json::to_string(p).unwrap_or_default());

            Some(GisFeature {
                id: Ulid::new(),
                layer_id,
                geometry_wkt: wkt,
                properties_json: props,
                work_item_id: None,
                created_at: now,
            })
        })
        .collect();

    Ok(result)
}

/// Export `GisFeature` entities to a pretty-printed GeoJSON string.
pub fn export_geojson(features: &[GisFeature]) -> Result<String, GisError> {
    let geojson_features: Vec<Feature> = features
        .iter()
        .filter_map(|f| {
            let geometry = wkt_to_geometry(&f.geometry_wkt)?;
            let properties = f
                .properties_json
                .as_ref()
                .and_then(|p| serde_json::from_str(p).ok());
            Some(Feature {
                geometry: Some(geometry),
                properties,
                ..Default::default()
            })
        })
        .collect();

    let fc = FeatureCollection {
        features: geojson_features,
        bbox: None,
        foreign_members: None,
    };

    serde_json::to_string_pretty(&fc)
        .map_err(|e| GisError::AnalysisError(format!("{e}")))
}

/// Convert a `geojson::Geometry` to a simple WKT string.
pub fn geometry_to_wkt(geom: &Geometry) -> String {
    match &geom.value {
        Value::Point(coords) => format!("POINT({} {})", coords[0], coords[1]),
        Value::LineString(coords) => {
            let points: Vec<String> = coords
                .iter()
                .map(|c| format!("{} {}", c[0], c[1]))
                .collect();
            format!("LINESTRING({})", points.join(", "))
        }
        Value::Polygon(rings) => {
            let ring_strs: Vec<String> = rings
                .iter()
                .map(|ring| {
                    let points: Vec<String> =
                        ring.iter().map(|c| format!("{} {}", c[0], c[1])).collect();
                    format!("({})", points.join(", "))
                })
                .collect();
            format!("POLYGON({})", ring_strs.join(", "))
        }
        Value::MultiPoint(pts) => {
            let points: Vec<String> = pts
                .iter()
                .map(|c| format!("({} {})", c[0], c[1]))
                .collect();
            format!("MULTIPOINT({})", points.join(", "))
        }
        Value::MultiLineString(lines) => {
            let line_strs: Vec<String> = lines
                .iter()
                .map(|line| {
                    let points: Vec<String> =
                        line.iter().map(|c| format!("{} {}", c[0], c[1])).collect();
                    format!("({})", points.join(", "))
                })
                .collect();
            format!("MULTILINESTRING({})", line_strs.join(", "))
        }
        Value::MultiPolygon(polys) => {
            let poly_strs: Vec<String> = polys
                .iter()
                .map(|rings| {
                    let ring_strs: Vec<String> = rings
                        .iter()
                        .map(|ring| {
                            let points: Vec<String> =
                                ring.iter().map(|c| format!("{} {}", c[0], c[1])).collect();
                            format!("({})", points.join(", "))
                        })
                        .collect();
                    format!("({})", ring_strs.join(", "))
                })
                .collect();
            format!("MULTIPOLYGON({})", poly_strs.join(", "))
        }
        _ => "GEOMETRYCOLLECTION EMPTY".into(),
    }
}

/// Convert a simple WKT string back to a `geojson::Geometry`.
///
/// Supports POINT, LINESTRING, and POLYGON. Returns `None` for unrecognised
/// or malformed WKT.
pub fn wkt_to_geometry(wkt: &str) -> Option<Geometry> {
    let wkt = wkt.trim();

    if let Some(rest) = wkt.strip_prefix("POINT(").and_then(|s| s.strip_suffix(')')) {
        let parts: Vec<f64> = rest
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();
        if parts.len() >= 2 {
            return Some(Geometry::new(Value::Point(vec![parts[0], parts[1]])));
        }
    }

    if let Some(rest) = wkt
        .strip_prefix("LINESTRING(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let coords = parse_coord_list(rest);
        if !coords.is_empty() {
            return Some(Geometry::new(Value::LineString(coords)));
        }
    }

    if let Some(rest) = wkt
        .strip_prefix("POLYGON(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let rings = parse_rings(rest);
        if !rings.is_empty() {
            return Some(Geometry::new(Value::Polygon(rings)));
        }
    }

    None
}

/// Parse a space-separated coordinate pair list like "1 2, 3 4, 5 6".
fn parse_coord_list(s: &str) -> Vec<Vec<f64>> {
    s.split(',')
        .filter_map(|pair| {
            let nums: Vec<f64> = pair
                .split_whitespace()
                .filter_map(|n| n.parse().ok())
                .collect();
            if nums.len() >= 2 {
                Some(vec![nums[0], nums[1]])
            } else {
                None
            }
        })
        .collect()
}

/// Parse ring groups from a Polygon WKT interior like "(1 2, 3 4), (5 6, 7 8)".
fn parse_rings(s: &str) -> Vec<Vec<Vec<f64>>> {
    let mut rings = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;

    for (i, ch) in s.char_indices() {
        match ch {
            '(' => {
                if depth == 0 {
                    start = i + 1;
                }
                depth += 1;
            }
            ')' => {
                depth -= 1;
                if depth == 0 {
                    let ring = parse_coord_list(&s[start..i]);
                    if !ring.is_empty() {
                        rings.push(ring);
                    }
                }
            }
            _ => {}
        }
    }
    rings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_wkt_roundtrip() {
        let geom = Geometry::new(Value::Point(vec![10.5, 20.3]));
        let wkt = geometry_to_wkt(&geom);
        assert_eq!(wkt, "POINT(10.5 20.3)");
        let back = wkt_to_geometry(&wkt).unwrap();
        if let Value::Point(c) = back.value {
            assert!((c[0] - 10.5).abs() < 1e-9);
            assert!((c[1] - 20.3).abs() < 1e-9);
        } else {
            panic!("expected Point");
        }
    }

    #[test]
    fn linestring_wkt_roundtrip() {
        let geom = Geometry::new(Value::LineString(vec![
            vec![0.0, 0.0],
            vec![1.0, 1.0],
            vec![2.0, 0.0],
        ]));
        let wkt = geometry_to_wkt(&geom);
        assert!(wkt.starts_with("LINESTRING("));
        let back = wkt_to_geometry(&wkt).unwrap();
        assert!(matches!(back.value, Value::LineString(_)));
    }

    #[test]
    fn polygon_wkt_roundtrip() {
        let geom = Geometry::new(Value::Polygon(vec![vec![
            vec![0.0, 0.0],
            vec![1.0, 0.0],
            vec![1.0, 1.0],
            vec![0.0, 1.0],
            vec![0.0, 0.0],
        ]]));
        let wkt = geometry_to_wkt(&geom);
        assert!(wkt.starts_with("POLYGON("));
        let back = wkt_to_geometry(&wkt).unwrap();
        assert!(matches!(back.value, Value::Polygon(_)));
    }

    #[test]
    fn unknown_wkt_returns_geometrycollection_empty() {
        let geom = Geometry::new(Value::GeometryCollection(vec![]));
        let wkt = geometry_to_wkt(&geom);
        assert_eq!(wkt, "GEOMETRYCOLLECTION EMPTY");
    }

    #[test]
    fn export_geojson_produces_valid_json() {
        let feature = GisFeature {
            id: Ulid::new(),
            layer_id: Ulid::new(),
            geometry_wkt: "POINT(116.4 39.9)".into(),
            properties_json: Some(r#"{"name":"Beijing"}"#.into()),
            work_item_id: None,
            created_at: 0,
        };
        let json = export_geojson(&[feature]).unwrap();
        assert!(json.contains("FeatureCollection"));
        assert!(json.contains("Point"));
    }

    #[test]
    fn export_geojson_skips_unparseable_wkt() {
        let feature = GisFeature {
            id: Ulid::new(),
            layer_id: Ulid::new(),
            geometry_wkt: "INVALID WKT".into(),
            properties_json: None,
            work_item_id: None,
            created_at: 0,
        };
        let json = export_geojson(&[feature]).unwrap();
        // Feature with bad WKT is silently skipped → empty features array
        assert!(json.contains("\"features\""));
    }
}

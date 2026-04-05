use geo::algorithm::area::Area;
use geo::algorithm::bounding_rect::BoundingRect;
use geo::algorithm::centroid::Centroid;
use geo::algorithm::contains::Contains;
use geo::algorithm::intersects::Intersects;
use geo::{Point, Polygon};

/// Build an approximate circular buffer around `(lon, lat)` with the given
/// radius in metres. Returns a closed ring of `(lon, lat)` coordinate pairs.
pub fn buffer_point(lon: f64, lat: f64, radius_meters: f64) -> Vec<(f64, f64)> {
    let deg_per_meter_lat = 1.0 / 111_320.0;
    let deg_per_meter_lon = 1.0 / (111_320.0 * lat.to_radians().cos());
    let segments = 64usize;
    let mut coords = Vec::with_capacity(segments + 1);
    for i in 0..=segments {
        let angle = 2.0 * std::f64::consts::PI * (i as f64) / (segments as f64);
        let dx = radius_meters * angle.cos() * deg_per_meter_lon;
        let dy = radius_meters * angle.sin() * deg_per_meter_lat;
        coords.push((lon + dx, lat + dy));
    }
    coords
}

/// Return `true` when polygon `a` and polygon `b` share any area or boundary.
pub fn geometries_intersect(a: &Polygon<f64>, b: &Polygon<f64>) -> bool {
    a.intersects(b)
}

/// Return `true` when polygon `a` fully contains point `b`.
pub fn geometry_contains(a: &Polygon<f64>, b: &Point<f64>) -> bool {
    a.contains(b)
}

/// Return the centroid of `poly` as `(lon, lat)`, or `None` for degenerate geometries.
pub fn centroid(poly: &Polygon<f64>) -> Option<(f64, f64)> {
    poly.centroid().map(|c| (c.x(), c.y()))
}

/// Return the unsigned area of `poly` in square degrees.
pub fn polygon_area(poly: &Polygon<f64>) -> f64 {
    poly.unsigned_area()
}

/// Return the axis-aligned bounding rectangle of `poly` as
/// `((min_lon, min_lat), (max_lon, max_lat))`.
pub fn bounding_rect(poly: &Polygon<f64>) -> Option<((f64, f64), (f64, f64))> {
    poly.bounding_rect()
        .map(|r| ((r.min().x, r.min().y), (r.max().x, r.max().y)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo::polygon;

    fn unit_square() -> Polygon<f64> {
        polygon![
            (x: 0.0, y: 0.0),
            (x: 1.0, y: 0.0),
            (x: 1.0, y: 1.0),
            (x: 0.0, y: 1.0),
            (x: 0.0, y: 0.0),
        ]
    }

    #[test]
    fn buffer_point_returns_closed_ring() {
        let ring = buffer_point(116.4, 39.9, 1000.0);
        // closed ring: first and last point should be equal
        assert_eq!(ring.len(), 65);
        assert!((ring[0].0 - ring[64].0).abs() < 1e-10);
        assert!((ring[0].1 - ring[64].1).abs() < 1e-10);
    }

    #[test]
    fn intersecting_polygons_detected() {
        let a = unit_square();
        let b = polygon![
            (x: 0.5, y: 0.5),
            (x: 1.5, y: 0.5),
            (x: 1.5, y: 1.5),
            (x: 0.5, y: 1.5),
            (x: 0.5, y: 0.5),
        ];
        assert!(geometries_intersect(&a, &b));
    }

    #[test]
    fn disjoint_polygons_not_intersecting() {
        let a = unit_square();
        let b = polygon![
            (x: 5.0, y: 5.0),
            (x: 6.0, y: 5.0),
            (x: 6.0, y: 6.0),
            (x: 5.0, y: 6.0),
            (x: 5.0, y: 5.0),
        ];
        assert!(!geometries_intersect(&a, &b));
    }

    #[test]
    fn polygon_contains_interior_point() {
        let poly = unit_square();
        let interior = Point::new(0.5, 0.5);
        assert!(geometry_contains(&poly, &interior));
    }

    #[test]
    fn polygon_does_not_contain_exterior_point() {
        let poly = unit_square();
        let exterior = Point::new(2.0, 2.0);
        assert!(!geometry_contains(&poly, &exterior));
    }

    #[test]
    fn centroid_of_unit_square_is_centre() {
        let poly = unit_square();
        let (cx, cy) = centroid(&poly).unwrap();
        assert!((cx - 0.5).abs() < 1e-9);
        assert!((cy - 0.5).abs() < 1e-9);
    }

    #[test]
    fn polygon_area_positive() {
        let poly = unit_square();
        let area = polygon_area(&poly);
        assert!((area - 1.0).abs() < 1e-9);
    }

    #[test]
    fn bounding_rect_correct() {
        let poly = unit_square();
        let ((min_x, min_y), (max_x, max_y)) = bounding_rect(&poly).unwrap();
        assert!((min_x - 0.0).abs() < 1e-9);
        assert!((min_y - 0.0).abs() < 1e-9);
        assert!((max_x - 1.0).abs() < 1e-9);
        assert!((max_y - 1.0).abs() < 1e-9);
    }
}

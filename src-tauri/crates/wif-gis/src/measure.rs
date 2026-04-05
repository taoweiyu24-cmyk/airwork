//! Distance and area measurement utilities.

use crate::coordinate::transform::haversine_distance;

/// Calculate the total length of a polyline in **metres**.
///
/// `coords` is a slice of `(lon, lat)` pairs in WGS84 degrees.
/// Returns 0.0 for slices with fewer than two points.
pub fn polyline_length(coords: &[(f64, f64)]) -> f64 {
    coords
        .windows(2)
        .map(|w| haversine_distance(w[0].1, w[0].0, w[1].1, w[1].0))
        .sum()
}

/// Calculate the approximate area of a polygon in **square metres**.
///
/// `coords` is a slice of `(lon, lat)` pairs in WGS84 degrees forming a
/// closed ring. Uses the Shoelace formula after scaling coordinates to metres
/// using the mid-latitude approximation.
///
/// Returns 0.0 for slices with fewer than three points.
pub fn polygon_area_m2(coords: &[(f64, f64)]) -> f64 {
    if coords.len() < 3 {
        return 0.0;
    }
    let mid_lat = coords.iter().map(|c| c.1).sum::<f64>() / coords.len() as f64;
    let m_per_deg_lat = 111_320.0_f64;
    let m_per_deg_lon = 111_320.0 * mid_lat.to_radians().cos();

    let scaled: Vec<(f64, f64)> = coords
        .iter()
        .map(|(lon, lat)| (lon * m_per_deg_lon, lat * m_per_deg_lat))
        .collect();

    let n = scaled.len();
    let area: f64 = (0..n)
        .map(|i| {
            let j = (i + 1) % n;
            scaled[i].0 * scaled[j].1 - scaled[j].0 * scaled[i].1
        })
        .sum();

    (area / 2.0).abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_polyline_has_zero_length() {
        assert_eq!(polyline_length(&[]), 0.0);
    }

    #[test]
    fn single_point_polyline_has_zero_length() {
        assert_eq!(polyline_length(&[(0.0, 0.0)]), 0.0);
    }

    #[test]
    fn polyline_length_equatorial_segment() {
        // 1° longitude at the equator ≈ 111,195 m (haversine, WGS84 mean radius).
        // Allow ±500 m tolerance for the approximation.
        let len = polyline_length(&[(0.0, 0.0), (1.0, 0.0)]);
        assert!((len - 111_195.0).abs() < 500.0, "len={len}");
    }

    #[test]
    fn polygon_area_below_three_points_is_zero() {
        assert_eq!(polygon_area_m2(&[(0.0, 0.0), (1.0, 0.0)]), 0.0);
    }

    #[test]
    fn polygon_area_unit_degree_square_near_equator() {
        // 1° × 1° square at equator ≈ 111,320 × 111,320 m²
        let coords = vec![
            (0.0_f64, 0.0_f64),
            (1.0, 0.0),
            (1.0, 1.0),
            (0.0, 1.0),
            (0.0, 0.0),
        ];
        let area = polygon_area_m2(&coords);
        let expected = 111_320.0_f64 * 111_320.0;
        // Allow 1% tolerance for approximation
        assert!(
            (area - expected).abs() / expected < 0.01,
            "area={area}, expected≈{expected}"
        );
    }
}

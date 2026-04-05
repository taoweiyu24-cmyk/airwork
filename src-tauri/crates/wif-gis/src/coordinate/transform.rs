/// Convert degrees to radians.
#[inline]
pub fn deg_to_rad(deg: f64) -> f64 {
    deg * std::f64::consts::PI / 180.0
}

/// Convert radians to degrees.
#[inline]
pub fn rad_to_deg(rad: f64) -> f64 {
    rad * 180.0 / std::f64::consts::PI
}

/// Haversine great-circle distance between two WGS84 points in **metres**.
pub fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const EARTH_RADIUS_M: f64 = 6_371_000.0;
    let d_lat = deg_to_rad(lat2 - lat1);
    let d_lon = deg_to_rad(lon2 - lon1);
    let a = (d_lat / 2.0).sin().powi(2)
        + deg_to_rad(lat1).cos() * deg_to_rad(lat2).cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    EARTH_RADIUS_M * c
}

/// Convert WGS84 longitude/latitude to Web Mercator (EPSG:3857) metres.
pub fn wgs84_to_mercator(lon: f64, lat: f64) -> (f64, f64) {
    const HALF_CIRCUMFERENCE: f64 = 20_037_508.34;
    let x = lon * HALF_CIRCUMFERENCE / 180.0;
    let y_rad = ((90.0 + lat) * std::f64::consts::PI / 360.0).tan().ln()
        / (std::f64::consts::PI / 180.0);
    let y = y_rad * HALF_CIRCUMFERENCE / 180.0;
    (x, y)
}

/// Convert Web Mercator (EPSG:3857) metres back to WGS84 longitude/latitude.
pub fn mercator_to_wgs84(x: f64, y: f64) -> (f64, f64) {
    const HALF_CIRCUMFERENCE: f64 = 20_037_508.34;
    let lon = x * 180.0 / HALF_CIRCUMFERENCE;
    let lat_rad = (y * 180.0 / HALF_CIRCUMFERENCE) * (std::f64::consts::PI / 180.0);
    let lat = rad_to_deg(2.0 * lat_rad.exp().atan() - std::f64::consts::PI / 2.0);
    (lon, lat)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deg_rad_roundtrip() {
        let deg = 45.0_f64;
        assert!((rad_to_deg(deg_to_rad(deg)) - deg).abs() < 1e-10);
    }

    #[test]
    fn haversine_equator_quarter_circle() {
        // 90° longitude difference at equator ≈ 10,007,543 m
        let d = haversine_distance(0.0, 0.0, 0.0, 90.0);
        assert!((d - 10_007_543.0).abs() < 100.0);
    }

    #[test]
    fn haversine_zero_distance() {
        let d = haversine_distance(39.9, 116.4, 39.9, 116.4);
        assert!(d.abs() < 1e-6);
    }

    #[test]
    fn mercator_roundtrip_beijing() {
        let (lon, lat) = (116.4, 39.9);
        let (mx, my) = wgs84_to_mercator(lon, lat);
        let (lon2, lat2) = mercator_to_wgs84(mx, my);
        assert!((lon2 - lon).abs() < 1e-6);
        assert!((lat2 - lat).abs() < 1e-6);
    }

    #[test]
    fn mercator_origin_is_zero() {
        let (x, y) = wgs84_to_mercator(0.0, 0.0);
        assert!(x.abs() < 1e-6);
        assert!(y.abs() < 1e-6);
    }
}

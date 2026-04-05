use serde::{Deserialize, Serialize};

/// A map tile source with URL template and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileSource {
    pub name: String,
    pub url_template: String,
    pub attribution: String,
    pub min_zoom: u8,
    pub max_zoom: u8,
}

impl TileSource {
    /// OpenStreetMap standard tile layer.
    pub fn osm() -> Self {
        Self {
            name: "OpenStreetMap".into(),
            url_template: "https://tile.openstreetmap.org/{z}/{x}/{y}.png".into(),
            attribution: "© OpenStreetMap contributors".into(),
            min_zoom: 0,
            max_zoom: 19,
        }
    }

    /// 天地图 vector layer (requires a valid API token).
    pub fn tianditu_vec(token: &str) -> Self {
        Self {
            name: "天地图矢量".into(),
            url_template: format!(
                "https://t0.tianditu.gov.cn/vec_w/wmts?SERVICE=WMTS&REQUEST=GetTile\
                &VERSION=1.0.0&LAYER=vec&STYLE=default&TILEMATRIXSET=w&FORMAT=tiles\
                &TILECOL={{x}}&TILEROW={{y}}&TILEMATRIX={{z}}&tk={token}"
            ),
            attribution: "© 天地图".into(),
            min_zoom: 0,
            max_zoom: 18,
        }
    }

    /// 天地图 satellite imagery layer (requires a valid API token).
    pub fn tianditu_img(token: &str) -> Self {
        Self {
            name: "天地图影像".into(),
            url_template: format!(
                "https://t0.tianditu.gov.cn/img_w/wmts?SERVICE=WMTS&REQUEST=GetTile\
                &VERSION=1.0.0&LAYER=img&STYLE=default&TILEMATRIXSET=w&FORMAT=tiles\
                &TILECOL={{x}}&TILEROW={{y}}&TILEMATRIX={{z}}&tk={token}"
            ),
            attribution: "© 天地图".into(),
            min_zoom: 0,
            max_zoom: 18,
        }
    }

    /// Arbitrary XYZ tile source.
    pub fn xyz(name: &str, url_template: &str) -> Self {
        Self {
            name: name.into(),
            url_template: url_template.into(),
            attribution: String::new(),
            min_zoom: 0,
            max_zoom: 18,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn osm_has_expected_template() {
        let src = TileSource::osm();
        assert_eq!(src.name, "OpenStreetMap");
        assert!(src.url_template.contains("{z}"));
        assert_eq!(src.min_zoom, 0);
        assert_eq!(src.max_zoom, 19);
    }

    #[test]
    fn tianditu_vec_embeds_token() {
        let src = TileSource::tianditu_vec("mytoken");
        assert!(src.url_template.contains("mytoken"));
    }

    #[test]
    fn tianditu_img_embeds_token() {
        let src = TileSource::tianditu_img("imgtoken");
        assert!(src.url_template.contains("imgtoken"));
        assert!(src.url_template.contains("img_w"));
    }

    #[test]
    fn xyz_roundtrips_name_and_template() {
        let src = TileSource::xyz("Custom", "https://example.com/{z}/{x}/{y}.png");
        assert_eq!(src.name, "Custom");
        assert!(src.url_template.contains("example.com"));
        assert!(src.attribution.is_empty());
    }
}

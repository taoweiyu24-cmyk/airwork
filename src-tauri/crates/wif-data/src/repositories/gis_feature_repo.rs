use ulid::Ulid;
use wif_domain::entities::GisFeature;
use wif_domain::repository::{DomainError, Result, GisFeatureRepository};

use crate::connection::Database;
use crate::repositories::util::{anyhow_to_domain, parse_ulid_col};

pub struct SqliteGisFeatureRepo<'a> {
    db: &'a Database,
}

impl<'a> SqliteGisFeatureRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    fn row_to_feature(row: &rusqlite::Row) -> rusqlite::Result<GisFeature> {
        let id_str: String = row.get(0)?;
        let layer_str: String = row.get(1)?;
        let wi_str: Option<String> = row.get(4)?;
        let work_item_id = wi_str.as_deref().map(|s| parse_ulid_col(s, 4)).transpose()?;
        Ok(GisFeature {
            id: parse_ulid_col(&id_str, 0)?,
            layer_id: parse_ulid_col(&layer_str, 1)?,
            geometry_wkt: row.get(2)?,
            properties_json: row.get(3)?,
            work_item_id,
            created_at: row.get(5)?,
        })
    }
}

const SELECT_COLS: &str =
    "SELECT id,layer_id,geometry_wkt,properties_json,work_item_id,created_at FROM gis_features";

impl<'a> GisFeatureRepository for SqliteGisFeatureRepo<'a> {
    fn find_by_layer(&self, layer_id: Ulid) -> Result<Vec<GisFeature>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(&format!("{SELECT_COLS} WHERE layer_id=?1 ORDER BY created_at DESC"))?;
            let rows = stmt.query_map([layer_id.to_string()], Self::row_to_feature)?;
            rows.map(|r| r.map_err(|e| anyhow::anyhow!(e))).collect()
        }).map_err(anyhow_to_domain)
    }

    fn find_by_id(&self, id: Ulid) -> Result<Option<GisFeature>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(&format!("{SELECT_COLS} WHERE id=?1"))?;
            let mut rows = stmt.query_map([id.to_string()], Self::row_to_feature)?;
            match rows.next() {
                None => Ok(None),
                Some(r) => r.map(Some).map_err(|e| anyhow::anyhow!(e)),
            }
        }).map_err(anyhow_to_domain)
    }

    fn find_by_work_item(&self, work_item_id: Ulid) -> Result<Vec<GisFeature>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(&format!("{SELECT_COLS} WHERE work_item_id=?1 ORDER BY created_at DESC"))?;
            let rows = stmt.query_map([work_item_id.to_string()], Self::row_to_feature)?;
            rows.map(|r| r.map_err(|e| anyhow::anyhow!(e))).collect()
        }).map_err(anyhow_to_domain)
    }

    fn create(&self, feature: &GisFeature) -> Result<GisFeature> {
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO gis_features (id,layer_id,geometry_wkt,properties_json,work_item_id,created_at) VALUES (?1,?2,?3,?4,?5,?6)",
                rusqlite::params![
                    feature.id.to_string(),
                    feature.layer_id.to_string(),
                    feature.geometry_wkt,
                    feature.properties_json,
                    feature.work_item_id.map(|u| u.to_string()),
                    feature.created_at,
                ],
            )?;
            Ok(feature.clone())
        }).map_err(anyhow_to_domain)
    }

    fn update(&self, feature: &GisFeature) -> Result<GisFeature> {
        self.db.with_conn(|conn| {
            let n = conn.execute(
                "UPDATE gis_features SET layer_id=?1,geometry_wkt=?2,properties_json=?3,work_item_id=?4 WHERE id=?5",
                rusqlite::params![
                    feature.layer_id.to_string(),
                    feature.geometry_wkt,
                    feature.properties_json,
                    feature.work_item_id.map(|u| u.to_string()),
                    feature.id.to_string(),
                ],
            )?;
            if n == 0 {
                return Err(anyhow::anyhow!(DomainError::NotFound(feature.id.to_string())));
            }
            Ok(feature.clone())
        }).map_err(anyhow_to_domain)
    }

    fn delete(&self, id: Ulid) -> Result<()> {
        self.db.with_conn(|conn| {
            let n = conn.execute("DELETE FROM gis_features WHERE id=?1", [id.to_string()])?;
            if n == 0 {
                return Err(anyhow::anyhow!(DomainError::NotFound(id.to_string())));
            }
            Ok(())
        }).map_err(anyhow_to_domain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::Database;
    use crate::repositories::gis_layer_repo::SqliteGisLayerRepo;
    use wif_domain::entities::GisLayer;
    use wif_domain::enums::LayerType;
    use wif_domain::repository::GisLayerRepository;

    fn setup() -> Database {
        let db = Database::open_in_memory().unwrap();
        db.run_migrations().unwrap();
        db
    }

    fn insert_layer(db: &Database) -> Ulid {
        let repo = SqliteGisLayerRepo::new(db);
        let layer = GisLayer {
            id: Ulid::new(), name: "L".into(), layer_type: LayerType::Vector,
            source_path: None, visible: true, opacity: 1.0,
            style_json: None, created_at: 1_000_000,
        };
        repo.create(&layer).unwrap();
        layer.id
    }

    #[test]
    fn create_and_find_feature() {
        let db = setup();
        let layer_id = insert_layer(&db);
        let repo = SqliteGisFeatureRepo::new(&db);
        let feat = GisFeature {
            id: Ulid::new(), layer_id,
            geometry_wkt: "POINT(116.39 39.91)".into(),
            properties_json: None, work_item_id: None, created_at: 1_000_000,
        };
        repo.create(&feat).unwrap();
        let found = repo.find_by_id(feat.id).unwrap().unwrap();
        assert_eq!(found.geometry_wkt, "POINT(116.39 39.91)");
    }

    #[test]
    fn find_by_layer() {
        let db = setup();
        let layer_id = insert_layer(&db);
        let repo = SqliteGisFeatureRepo::new(&db);
        let feat = GisFeature {
            id: Ulid::new(), layer_id,
            geometry_wkt: "POINT(0 0)".into(),
            properties_json: None, work_item_id: None, created_at: 1_000_000,
        };
        repo.create(&feat).unwrap();
        assert_eq!(repo.find_by_layer(layer_id).unwrap().len(), 1);
    }
}

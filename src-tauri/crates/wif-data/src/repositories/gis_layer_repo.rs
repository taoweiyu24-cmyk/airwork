use ulid::Ulid;
use wif_domain::entities::GisLayer;
use wif_domain::enums::LayerType;
use wif_domain::repository::{DomainError, Result, GisLayerRepository};

use crate::connection::Database;
use crate::repositories::util::{anyhow_to_domain, parse_ulid_col};

pub struct SqliteGisLayerRepo<'a> {
    db: &'a Database,
}

impl<'a> SqliteGisLayerRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    fn row_to_layer(row: &rusqlite::Row) -> rusqlite::Result<GisLayer> {
        let id_str: String = row.get(0)?;
        let lt_str: String = row.get(2)?;
        let visible: i64 = row.get(4)?;
        Ok(GisLayer {
            id: parse_ulid_col(&id_str, 0)?,
            name: row.get(1)?,
            layer_type: layer_type_from_str(&lt_str),
            source_path: row.get(3)?,
            visible: visible != 0,
            opacity: row.get::<_, f64>(5)? as f32,
            style_json: row.get(6)?,
            created_at: row.get(7)?,
        })
    }
}

const SELECT_COLS: &str =
    "SELECT id,name,layer_type,source_path,visible,opacity,style_json,created_at FROM gis_layers";

impl<'a> GisLayerRepository for SqliteGisLayerRepo<'a> {
    fn find_all(&self) -> Result<Vec<GisLayer>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(&format!("{SELECT_COLS} ORDER BY name"))?;
            let rows = stmt.query_map([], Self::row_to_layer)?;
            rows.map(|r| r.map_err(|e| anyhow::anyhow!(e))).collect()
        }).map_err(anyhow_to_domain)
    }

    fn find_by_id(&self, id: Ulid) -> Result<Option<GisLayer>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(&format!("{SELECT_COLS} WHERE id=?1"))?;
            let mut rows = stmt.query_map([id.to_string()], Self::row_to_layer)?;
            match rows.next() {
                None => Ok(None),
                Some(r) => r.map(Some).map_err(|e| anyhow::anyhow!(e)),
            }
        }).map_err(anyhow_to_domain)
    }

    fn create(&self, layer: &GisLayer) -> Result<GisLayer> {
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO gis_layers (id,name,layer_type,source_path,visible,opacity,style_json,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                rusqlite::params![
                    layer.id.to_string(), layer.name,
                    layer_type_to_str(layer.layer_type),
                    layer.source_path,
                    layer.visible as i64,
                    layer.opacity as f64,
                    layer.style_json, layer.created_at,
                ],
            )?;
            Ok(layer.clone())
        }).map_err(anyhow_to_domain)
    }

    fn update(&self, layer: &GisLayer) -> Result<GisLayer> {
        self.db.with_conn(|conn| {
            let n = conn.execute(
                "UPDATE gis_layers SET name=?1,layer_type=?2,source_path=?3,visible=?4,opacity=?5,style_json=?6 WHERE id=?7",
                rusqlite::params![
                    layer.name,
                    layer_type_to_str(layer.layer_type),
                    layer.source_path,
                    layer.visible as i64,
                    layer.opacity as f64,
                    layer.style_json,
                    layer.id.to_string(),
                ],
            )?;
            if n == 0 {
                return Err(anyhow::anyhow!(DomainError::NotFound(layer.id.to_string())));
            }
            Ok(layer.clone())
        }).map_err(anyhow_to_domain)
    }

    fn delete(&self, id: Ulid) -> Result<()> {
        self.db.with_conn(|conn| {
            let n = conn.execute("DELETE FROM gis_layers WHERE id=?1", [id.to_string()])?;
            if n == 0 {
                return Err(anyhow::anyhow!(DomainError::NotFound(id.to_string())));
            }
            Ok(())
        }).map_err(anyhow_to_domain)
    }
}

fn layer_type_to_str(lt: LayerType) -> &'static str {
    match lt {
        LayerType::Vector => "vector",
        LayerType::Raster => "raster",
        LayerType::Tile => "tile",
    }
}

fn layer_type_from_str(s: &str) -> LayerType {
    match s {
        "raster" => LayerType::Raster,
        "tile" => LayerType::Tile,
        _ => LayerType::Vector,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::Database;

    fn setup() -> Database {
        let db = Database::open_in_memory().unwrap();
        db.run_migrations().unwrap();
        db
    }

    fn sample() -> GisLayer {
        GisLayer {
            id: Ulid::new(), name: "Base".into(), layer_type: LayerType::Vector,
            source_path: Some("/data/base.geojson".into()),
            visible: true, opacity: 1.0, style_json: None, created_at: 1_000_000,
        }
    }

    #[test]
    fn create_and_find_layer() {
        let db = setup();
        let repo = SqliteGisLayerRepo::new(&db);
        let layer = sample();
        repo.create(&layer).unwrap();
        let found = repo.find_by_id(layer.id).unwrap().unwrap();
        assert_eq!(found.name, "Base");
        assert_eq!(found.layer_type, LayerType::Vector);
    }

    #[test]
    fn update_layer() {
        let db = setup();
        let repo = SqliteGisLayerRepo::new(&db);
        let mut layer = sample();
        repo.create(&layer).unwrap();
        layer.visible = false;
        repo.update(&layer).unwrap();
        let found = repo.find_by_id(layer.id).unwrap().unwrap();
        assert!(!found.visible);
    }
}

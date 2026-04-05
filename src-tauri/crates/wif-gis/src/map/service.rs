use super::tile_source::TileSource;

/// Map service that manages a collection of tile sources.
pub struct MapService {
    tile_sources: Vec<TileSource>,
    active_source: usize,
}

impl MapService {
    /// Create a new `MapService` with OSM as the default source.
    pub fn new() -> Self {
        Self {
            tile_sources: vec![TileSource::osm()],
            active_source: 0,
        }
    }

    /// Append a tile source to the list.
    pub fn add_source(&mut self, source: TileSource) {
        self.tile_sources.push(source);
    }

    /// Return all registered tile sources.
    pub fn list_sources(&self) -> &[TileSource] {
        &self.tile_sources
    }

    /// Return the currently active tile source.
    pub fn active_source(&self) -> &TileSource {
        &self.tile_sources[self.active_source]
    }

    /// Switch the active source by index. Silently ignores out-of-range indices.
    pub fn set_active(&mut self, index: usize) {
        if index < self.tile_sources.len() {
            self.active_source = index;
        }
    }
}

impl Default for MapService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_service_has_osm_as_default() {
        let svc = MapService::new();
        assert_eq!(svc.list_sources().len(), 1);
        assert_eq!(svc.active_source().name, "OpenStreetMap");
    }

    #[test]
    fn add_source_increases_count() {
        let mut svc = MapService::new();
        svc.add_source(TileSource::xyz("Test", "https://test/{z}/{x}/{y}.png"));
        assert_eq!(svc.list_sources().len(), 2);
    }

    #[test]
    fn set_active_switches_source() {
        let mut svc = MapService::new();
        svc.add_source(TileSource::xyz("Second", "https://second/{z}/{x}/{y}.png"));
        svc.set_active(1);
        assert_eq!(svc.active_source().name, "Second");
    }

    #[test]
    fn set_active_out_of_bounds_is_ignored() {
        let mut svc = MapService::new();
        svc.set_active(99);
        // active_source should still be 0 (OSM)
        assert_eq!(svc.active_source().name, "OpenStreetMap");
    }
}

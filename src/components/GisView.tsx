import { useState, useEffect, useRef, useCallback } from 'react';
import maplibregl from 'maplibre-gl';
import 'maplibre-gl/dist/maplibre-gl.css';
import { Eye, EyeOff, Layers, Globe } from 'lucide-react';
import * as api from '../services/api';
import type { GisLayer, TileSource } from '../types/domain';
import Card from './ui/Card';
import Badge from './ui/Badge';

const LAYER_TYPE_VARIANT: Record<string, 'info' | 'success' | 'warning'> = {
  vector: 'info',
  raster: 'success',
  tile: 'warning',
};

const LAYER_TYPE_LABEL: Record<string, string> = {
  vector: '矢量',
  raster: '栅格',
  tile: '瓦片',
};

const DEFAULT_CENTER: [number, number] = [116.4, 39.9];
const DEFAULT_ZOOM = 4;
const RASTER_SOURCE_ID = 'basemap-source';
const RASTER_LAYER_ID = 'basemap-layer';

function buildMapStyle(source: TileSource): maplibregl.StyleSpecification {
  return {
    version: 8,
    sources: {
      [RASTER_SOURCE_ID]: {
        type: 'raster',
        tiles: [source.url_template],
        tileSize: 256,
        minzoom: source.min_zoom,
        maxzoom: source.max_zoom,
        attribution: source.attribution,
      },
    },
    layers: [
      {
        id: RASTER_LAYER_ID,
        type: 'raster',
        source: RASTER_SOURCE_ID,
      },
    ],
  };
}

export default function GisView() {
  const [tileSources, setTileSources] = useState<TileSource[]>([]);
  const [layers, setLayers] = useState<GisLayer[]>([]);
  const [selectedSourceIdx, setSelectedSourceIdx] = useState(0);
  const [loading, setLoading] = useState(true);

  const mapContainerRef = useRef<HTMLDivElement>(null);
  const mapRef = useRef<maplibregl.Map | null>(null);

  // Load data from backend
  useEffect(() => {
    let cancelled = false;

    async function load() {
      try {
        const [ts, gl] = await Promise.all([
          api.listTileSources(),
          api.listGisLayers(),
        ]);
        if (!cancelled) {
          setTileSources(ts);
          setLayers(gl);
        }
      } finally {
        if (!cancelled) setLoading(false);
      }
    }

    load();
    return () => { cancelled = true; };
  }, []);

  // Initialize map once data is loaded and container is ready
  useEffect(() => {
    if (loading || tileSources.length === 0 || !mapContainerRef.current) return;
    if (mapRef.current) return; // already initialized

    const source = tileSources[selectedSourceIdx] ?? tileSources[0];
    const map = new maplibregl.Map({
      container: mapContainerRef.current,
      style: buildMapStyle(source),
      center: DEFAULT_CENTER,
      zoom: DEFAULT_ZOOM,
    });

    map.addControl(new maplibregl.NavigationControl(), 'top-left');
    map.addControl(new maplibregl.AttributionControl({ compact: true }), 'bottom-right');

    mapRef.current = map;

    return () => {
      map.remove();
      mapRef.current = null;
    };
    // Only run on mount after loading; tile source changes handled separately
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [loading]);

  // Update tile source when selector changes
  useEffect(() => {
    const map = mapRef.current;
    if (!map || tileSources.length === 0) return;

    const source = tileSources[selectedSourceIdx];
    if (!source) return;

    // Wait for map style to be loaded before updating
    if (!map.isStyleLoaded()) {
      const onStyleLoad = () => {
        map.setStyle(buildMapStyle(source));
        map.off('style.load', onStyleLoad);
      };
      map.on('style.load', onStyleLoad);
      return;
    }

    map.setStyle(buildMapStyle(source));
  }, [selectedSourceIdx, tileSources]);

  const handleVisibilityToggle = useCallback((id: string) => {
    setLayers((prev) =>
      prev.map((l) => (l.id === id ? { ...l, visible: !l.visible } : l)),
    );
  }, []);

  const handleOpacityChange = useCallback((id: string, opacity: number) => {
    setLayers((prev) =>
      prev.map((l) => (l.id === id ? { ...l, opacity } : l)),
    );
  }, []);

  const activeSource: TileSource | undefined = tileSources[selectedSourceIdx];

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center text-gray-400 dark:text-gray-500">
        <Globe className="mr-2 h-5 w-5 animate-spin" />
        <span>加载 GIS 数据...</span>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col gap-4">
      {/* Top bar: tile source selector */}
      <div className="flex items-center gap-3 rounded-xl border border-gray-200 bg-white px-4 py-3 dark:border-gray-700 dark:bg-gray-800">
        <Globe className="h-5 w-5 shrink-0 text-blue-500" />
        <label
          htmlFor="tile-source-select"
          className="text-sm font-medium text-gray-700 dark:text-gray-300"
        >
          底图源
        </label>
        {tileSources.length > 0 ? (
          <select
            id="tile-source-select"
            value={selectedSourceIdx}
            onChange={(e) => setSelectedSourceIdx(Number(e.target.value))}
            className="rounded-lg border border-gray-300 bg-white px-3 py-1.5 text-sm text-gray-900 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100"
          >
            {tileSources.map((ts, idx) => (
              <option key={ts.name} value={idx}>
                {ts.name}
              </option>
            ))}
          </select>
        ) : (
          <span className="text-sm text-gray-400">暂无底图源</span>
        )}
        {activeSource && (
          <span className="ml-auto text-xs text-gray-400 dark:text-gray-500">
            缩放: {activeSource.min_zoom}–{activeSource.max_zoom} · {activeSource.attribution}
          </span>
        )}
      </div>

      {/* Main area: map + layers sidebar */}
      <div className="flex min-h-0 flex-1 gap-4">
        {/* Map container */}
        <div className="relative flex-1 overflow-hidden rounded-xl border border-gray-200 dark:border-gray-700">
          <div
            ref={mapContainerRef}
            className="absolute inset-0"
          />
          {tileSources.length === 0 && (
            <div className="absolute inset-0 flex flex-col items-center justify-center bg-gray-50 text-gray-400 dark:bg-gray-900 dark:text-gray-500">
              <Globe className="mb-3 h-16 w-16" />
              <p className="text-sm font-medium">暂无底图源可用</p>
            </div>
          )}
        </div>

        {/* Layers sidebar */}
        <Card title="图层" className="w-70 shrink-0 overflow-hidden">
          {layers.length === 0 ? (
            <div className="flex flex-col items-center gap-2 py-10 text-gray-400 dark:text-gray-500">
              <Layers className="h-10 w-10" />
              <span className="text-sm">暂无图层</span>
            </div>
          ) : (
            <ul className="flex flex-col gap-3">
              {layers.map((layer) => (
                <li
                  key={layer.id}
                  className="rounded-lg border border-gray-100 px-3 py-2.5 dark:border-gray-700"
                >
                  <div className="flex items-center gap-2">
                    <button
                      type="button"
                      onClick={() => handleVisibilityToggle(layer.id)}
                      className="shrink-0 text-gray-400 transition-colors hover:text-gray-700 dark:hover:text-gray-200"
                      aria-label={layer.visible ? '隐藏图层' : '显示图层'}
                    >
                      {layer.visible ? (
                        <Eye className="h-4 w-4 text-blue-500" />
                      ) : (
                        <EyeOff className="h-4 w-4" />
                      )}
                    </button>

                    <span className="flex-1 truncate text-sm font-medium text-gray-900 dark:text-white">
                      {layer.name}
                    </span>

                    <Badge variant={LAYER_TYPE_VARIANT[layer.layerType] ?? 'default'}>
                      {LAYER_TYPE_LABEL[layer.layerType] ?? layer.layerType}
                    </Badge>
                  </div>

                  <div className="mt-2 flex items-center gap-2">
                    <span className="text-xs text-gray-400">透明度</span>
                    <input
                      type="range"
                      min={0}
                      max={1}
                      step={0.05}
                      value={layer.opacity}
                      onChange={(e) =>
                        handleOpacityChange(layer.id, Number(e.target.value))
                      }
                      className="h-1 flex-1 cursor-pointer appearance-none rounded-full bg-gray-200 accent-blue-500 dark:bg-gray-600"
                    />
                    <span className="w-8 text-right text-xs tabular-nums text-gray-500">
                      {Math.round(layer.opacity * 100)}%
                    </span>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </Card>
      </div>
    </div>
  );
}

import { useState, useEffect } from 'react';
import { Map, Eye, EyeOff, Layers, Globe } from 'lucide-react';
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

export default function GisView() {
  const [tileSources, setTileSources] = useState<TileSource[]>([]);
  const [layers, setLayers] = useState<GisLayer[]>([]);
  const [selectedSourceIdx, setSelectedSourceIdx] = useState(0);
  const [loading, setLoading] = useState(true);

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

  const activeSource: TileSource | undefined = tileSources[selectedSourceIdx];

  function handleVisibilityToggle(id: string) {
    setLayers((prev) =>
      prev.map((l) => (l.id === id ? { ...l, visible: !l.visible } : l)),
    );
  }

  function handleOpacityChange(id: string, opacity: number) {
    setLayers((prev) =>
      prev.map((l) => (l.id === id ? { ...l, opacity } : l)),
    );
  }

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
            {activeSource.attribution}
          </span>
        )}
      </div>

      {/* Main area: map placeholder + layers sidebar */}
      <div className="flex min-h-0 flex-1 gap-4">
        {/* Map placeholder */}
        <div className="relative flex flex-1 flex-col items-center justify-center overflow-hidden rounded-xl border border-dashed border-gray-300 bg-gradient-to-br from-blue-50 to-sky-100 dark:border-gray-600 dark:from-gray-800 dark:to-gray-900">
          <Map className="mb-3 h-16 w-16 text-blue-300 dark:text-blue-700" />
          <p className="text-sm font-medium text-gray-500 dark:text-gray-400">
            地图视图（待集成 MapLibre GL JS）
          </p>

          {activeSource && (
            <div className="mt-6 w-80 rounded-lg border border-gray-200 bg-white/80 px-4 py-3 text-left shadow-sm backdrop-blur dark:border-gray-700 dark:bg-gray-800/80">
              <p className="mb-1 text-xs font-semibold uppercase tracking-wide text-gray-400 dark:text-gray-500">
                当前底图
              </p>
              <p className="text-sm font-medium text-gray-900 dark:text-white">
                {activeSource.name}
              </p>
              <p className="mt-1 truncate text-xs text-gray-500 dark:text-gray-400">
                {activeSource.url_template}
              </p>
              <div className="mt-2 flex gap-3 text-xs text-gray-400 dark:text-gray-500">
                <span>缩放: {activeSource.min_zoom}–{activeSource.max_zoom}</span>
                <span>· {activeSource.attribution}</span>
              </div>
            </div>
          )}
        </div>

        {/* Layers sidebar */}
        <Card title="图层" className="w-72 shrink-0 overflow-hidden">
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

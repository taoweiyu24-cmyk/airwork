# gis-map Implementation Plan

> **For agentic workers:** Use this plan to implement the gis-map module task-by-task. Each task is self-contained with complete code. Execute in order — later tasks depend on earlier ones compiling.

**Goal:** Provide Mapsui map rendering infrastructure — Map instance management, basemap tile sources (OSM/TianDiTu/XYZ), and GisLayer-to-Mapsui vector layer conversion with style mapping.

**Architecture:** MapService owns the Mapsui `Map`, TileSourceFactory creates tile layers, FeatureStyleProvider converts domain styles to Mapsui styles. The assembler agent binds `MapService.Map` to the Avalonia `MapControl` — this agent does NOT touch GisView.axaml, GisViewModel.cs, or DependencyInjection.cs.

**Tech Stack:** Mapsui.Avalonia 5.0.2, Mapsui.Nts 5.0.2, NetTopologySuite 2.6.0

**Internal CRS:** EPSG:4326 (WGS84). Mapsui renders in SphericalMercator (EPSG:3857). MapService handles the projection via `SphericalMercator.FromLonLat()`.

**Rendering Pipeline:**
```
GisFeature.GeometryWkt
  → GeometryService.ParseWkt()        // NTS Geometry in WGS84
  → ProjectToMercator()               // coordinate transformation
  → new GeometryFeature(projected)    // Mapsui feature
  → FeatureStyleProvider.CreateStyle() // VectorStyle from GisLayer colors
  → MemoryLayer { Features, Style }   // one MemoryLayer per GisLayer
  → Map.Layers.Add(layer)             // displayed by MapControl
```

---

### Task 1: Add Mapsui NuGet packages

**Files:** Modify `src/Wif.Gis/Wif.Gis.csproj`

- [ ] Step 1: Add `Mapsui.Avalonia` and `Mapsui.Nts` package references to the csproj. The `Mapsui.Avalonia` package provides the MapControl and tiling infrastructure. `Mapsui.Nts` provides `GeometryFeature` for NTS geometry integration.

Replace the entire `<ItemGroup>` containing PackageReferences with:

```xml
<ItemGroup>
  <PackageReference Include="Mapsui.Avalonia" Version="5.0.2" />
  <PackageReference Include="Mapsui.Nts" Version="5.0.2" />
  <PackageReference Include="NetTopologySuite" Version="2.6.0" />
  <PackageReference Include="NetTopologySuite.IO.GeoJSON" Version="4.0.0" />
  <PackageReference Include="NetTopologySuite.IO.ShapeFile" Version="2.1.0" />
  <PackageReference Include="NetTopologySuite.Features" Version="2.2.0" />
</ItemGroup>
```

The full csproj should be:

```xml
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net9.0</TargetFramework>
    <ImplicitUsings>enable</ImplicitUsings>
    <Nullable>enable</Nullable>
  </PropertyGroup>

  <ItemGroup>
    <ProjectReference Include="..\Wif.Domain\Wif.Domain.csproj" />
  </ItemGroup>

  <ItemGroup>
    <PackageReference Include="Mapsui.Avalonia" Version="5.0.2" />
    <PackageReference Include="Mapsui.Nts" Version="5.0.2" />
    <PackageReference Include="NetTopologySuite" Version="2.6.0" />
    <PackageReference Include="NetTopologySuite.IO.GeoJSON" Version="4.0.0" />
    <PackageReference Include="NetTopologySuite.IO.ShapeFile" Version="2.1.0" />
    <PackageReference Include="NetTopologySuite.Features" Version="2.2.0" />
  </ItemGroup>
</Project>
```

- [ ] Step 2: Restore packages and verify no errors:

```bash
dotnet restore src/Wif.Gis/Wif.Gis.csproj
```

- [ ] Step 3: Verify the project builds:

```bash
dotnet build src/Wif.Gis/Wif.Gis.csproj
```

---

### Task 2: Add GisLayerSourceType enum and SourceType property to GisLayer

**Files:**
- Create `src/Wif.Domain/Enums/GisLayerSourceType.cs`
- Modify `src/Wif.Domain/Entities/GisLayer.cs`

> **Note:** The existing `SourceType` enum in `Wif.Domain/Enums/SourceType.cs` is for WorkItem sources (Manual, Email, File, Api). The GIS layer source type is a separate concept, so we use `GisLayerSourceType` to avoid collision.

- [ ] Step 1: Create the enum file `src/Wif.Domain/Enums/GisLayerSourceType.cs`:

```csharp
namespace Wif.Domain.Enums;

/// <summary>
/// GIS 图层数据源类型
/// </summary>
public enum GisLayerSourceType
{
    /// <summary>矢量图层（本地要素数据）</summary>
    Vector,

    /// <summary>瓦片图层（在线底图服务）</summary>
    Tile
}
```

- [ ] Step 2: Add the `SourceType` property to `GisLayer.cs`. Add it after the `Description` property:

In `src/Wif.Domain/Entities/GisLayer.cs`, add the using and property so the file becomes:

```csharp
using Wif.Domain.Enums;

namespace Wif.Domain.Entities;

public class GisLayer
{
    public Ulid Id { get; init; }
    public required string Name { get; set; }
    public string? Description { get; set; }
    public GisLayerSourceType SourceType { get; set; } = GisLayerSourceType.Vector;
    public bool IsVisible { get; set; } = true;
    public int DisplayOrder { get; set; }
    public string FillColor { get; set; } = "#3388FF40";
    public string StrokeColor { get; set; } = "#3388FF";
    public double StrokeWidth { get; set; } = 2.0;
    public double Opacity { get; set; } = 1.0;
    public DateTimeOffset CreatedAt { get; init; }
    public DateTimeOffset UpdatedAt { get; set; }

    // Navigation
    public List<GisFeature> Features { get; init; } = [];
}
```

- [ ] Step 3: Verify the solution builds:

```bash
dotnet build Wif.sln
```

---

### Task 3: Create FeatureStyleProvider

**Files:** Create `src/Wif.Gis/Rendering/FeatureStyleProvider.cs`

This class maps GisLayer style properties (FillColor, StrokeColor, StrokeWidth, Opacity) to Mapsui `VectorStyle` objects. It parses hex color strings (with optional alpha) into Mapsui `Color` values.

- [ ] Step 1: Create the directory if it does not exist:

```bash
mkdir -p src/Wif.Gis/Rendering
```

- [ ] Step 2: Create `src/Wif.Gis/Rendering/FeatureStyleProvider.cs`:

```csharp
using Mapsui.Styles;
using Wif.Domain.Entities;

namespace Wif.Gis.Rendering;

/// <summary>
/// 将 GisLayer 的样式属性映射为 Mapsui VectorStyle。
/// </summary>
public sealed class FeatureStyleProvider
{
    /// <summary>
    /// 根据图层样式属性创建 Mapsui VectorStyle。
    /// </summary>
    public VectorStyle CreateStyle(GisLayer layer)
    {
        var fillColor = ParseColor(layer.FillColor);
        var strokeColor = ParseColor(layer.StrokeColor);

        var style = new VectorStyle
        {
            Fill = new Brush(fillColor),
            Outline = new Pen(strokeColor, layer.StrokeWidth),
            Line = new Pen(strokeColor, layer.StrokeWidth),
            Opacity = (float)layer.Opacity
        };

        return style;
    }

    /// <summary>
    /// 解析十六进制颜色字符串为 Mapsui Color。
    /// 支持格式: #RGB, #RGBA, #RRGGBB, #RRGGBBAA
    /// </summary>
    internal static Color ParseColor(string hex)
    {
        if (string.IsNullOrWhiteSpace(hex))
            return Color.Gray;

        var value = hex.TrimStart('#');

        return value.Length switch
        {
            // #RGB → expand to #RRGGBB
            3 => new Color(
                ParseHexPair(new string(value[0], 2)),
                ParseHexPair(new string(value[1], 2)),
                ParseHexPair(new string(value[2], 2))),

            // #RGBA → expand to #RRGGBBAA
            4 => new Color(
                ParseHexPair(new string(value[0], 2)),
                ParseHexPair(new string(value[1], 2)),
                ParseHexPair(new string(value[2], 2)),
                ParseHexPair(new string(value[3], 2))),

            // #RRGGBB
            6 => new Color(
                ParseHexPair(value[..2]),
                ParseHexPair(value[2..4]),
                ParseHexPair(value[4..6])),

            // #RRGGBBAA
            8 => new Color(
                ParseHexPair(value[..2]),
                ParseHexPair(value[2..4]),
                ParseHexPair(value[4..6]),
                ParseHexPair(value[6..8])),

            _ => Color.Gray
        };
    }

    private static int ParseHexPair(string pair) =>
        int.TryParse(pair, System.Globalization.NumberStyles.HexNumber, null, out var result)
            ? result
            : 128;
}
```

- [ ] Step 3: Verify it compiles:

```bash
dotnet build src/Wif.Gis/Wif.Gis.csproj
```

---

### Task 4: Create TileSourceFactory

**Files:** Create `src/Wif.Gis/Services/TileSourceFactory.cs`

This factory creates tile layers for three scenarios:
1. **OSM** — uses Mapsui's built-in `OpenStreetMap.CreateTileLayer()`
2. **TianDiTu (天地图)** — uses `HttpTileSource` with the WMTS URL template
3. **Custom XYZ** — uses `HttpTileSource` with a user-supplied URL template

- [ ] Step 1: Create `src/Wif.Gis/Services/TileSourceFactory.cs`:

```csharp
using BruTile;
using BruTile.Predefined;
using BruTile.Web;
using Mapsui.Layers;
using Mapsui.Tiling;
using Mapsui.Tiling.Layers;

namespace Wif.Gis.Services;

/// <summary>
/// 创建瓦片图层：支持 OSM、天地图、自定义 XYZ 瓦片源。
/// </summary>
public sealed class TileSourceFactory
{
    private const string TianDiTuVecUrl =
        "https://t{s}.tianditu.gov.cn/vec_w/wmts?SERVICE=WMTS&REQUEST=GetTile" +
        "&VERSION=1.0.0&LAYER=vec&STYLE=default&TILEMATRIXSET=w" +
        "&FORMAT=tiles&TILECOL={x}&TILEROW={y}&TILEMATRIX={z}&tk={key}";

    private const string TianDiTuCvaUrl =
        "https://t{s}.tianditu.gov.cn/cva_w/wmts?SERVICE=WMTS&REQUEST=GetTile" +
        "&VERSION=1.0.0&LAYER=cva&STYLE=default&TILEMATRIXSET=w" +
        "&FORMAT=tiles&TILECOL={x}&TILEROW={y}&TILEMATRIX={z}&tk={key}";

    private static readonly string[] TianDiTuServers = ["0", "1", "2", "3", "4", "5", "6", "7"];

    /// <summary>
    /// 创建 OpenStreetMap 底图图层。
    /// </summary>
    public TileLayer CreateOsmLayer(string name = "OpenStreetMap")
    {
        var layer = OpenStreetMap.CreateTileLayer(name);
        return layer;
    }

    /// <summary>
    /// 创建天地图矢量底图图层（含中文标注）。
    /// 需要天地图 API Key（从 https://console.tianditu.gov.cn 申请）。
    /// </summary>
    public ILayer CreateTianDiTuLayer(string apiKey, string name = "天地图")
    {
        var vecUrl = TianDiTuVecUrl.Replace("{key}", apiKey);
        var vecSource = CreateXyzTileSource(vecUrl, TianDiTuServers, name + "_vec");
        var vecLayer = new TileLayer(vecSource) { Name = name };
        return vecLayer;
    }

    /// <summary>
    /// 创建天地图中文标注图层（叠加在底图上）。
    /// </summary>
    public ILayer CreateTianDiTuAnnotationLayer(string apiKey, string name = "天地图标注")
    {
        var cvaUrl = TianDiTuCvaUrl.Replace("{key}", apiKey);
        var cvaSource = CreateXyzTileSource(cvaUrl, TianDiTuServers, name + "_cva");
        var cvaLayer = new TileLayer(cvaSource) { Name = name };
        return cvaLayer;
    }

    /// <summary>
    /// 创建自定义 XYZ 瓦片图层。
    /// URL 模板使用 {x}, {y}, {z} 占位符，可选 {s} 用于子域名轮询。
    /// 示例: "https://tile.example.com/{z}/{x}/{y}.png"
    /// </summary>
    public ILayer CreateXyzLayer(string urlTemplate, string name = "自定义瓦片",
        string[]? subdomains = null)
    {
        var source = CreateXyzTileSource(urlTemplate, subdomains, name);
        return new TileLayer(source) { Name = name };
    }

    private static HttpTileSource CreateXyzTileSource(string urlTemplate,
        string[]? serverNodes, string name)
    {
        // BruTile uses {s} for server nodes, {x}/{y}/{z} for tile coordinates.
        // Replace common placeholders to match BruTile's expected format.
        var bruTileUrl = urlTemplate
            .Replace("{x}", "{1}")
            .Replace("{y}", "{2}")
            .Replace("{z}", "{0}")
            .Replace("{s}", "{s}");

        return new HttpTileSource(
            new GlobalSphericalMercator(),
            bruTileUrl,
            serverNodes,
            name: name);
    }
}
```

- [ ] Step 2: Verify it compiles:

```bash
dotnet build src/Wif.Gis/Wif.Gis.csproj
```

> **Compile note:** If `BruTile` namespaces are not resolved, they are transitive dependencies of `Mapsui.Tiling` (included via `Mapsui.Avalonia`). If the compiler cannot find them, check that `Mapsui.Avalonia 5.0.2` was restored correctly. The `HttpTileSource` constructor and `GlobalSphericalMercator` come from the `BruTile` and `BruTile.Predefined` namespaces respectively.

---

### Task 5: Create MapService

**Files:** Create `src/Wif.Gis/Services/MapService.cs`

MapService is the central orchestrator that:
- Owns and exposes a `Mapsui.Map` instance for the assembler to bind to `MapControl.Map`
- Converts `GisLayer` entities (with their `GisFeature` children) into Mapsui `MemoryLayer` objects
- Projects WGS84 coordinates to SphericalMercator via `Mapsui.Projections.SphericalMercator`
- Manages basemap switching (OSM, TianDiTu, custom XYZ)
- Supports layer add/remove/refresh operations

- [ ] Step 1: Create `src/Wif.Gis/Services/MapService.cs`:

```csharp
using Mapsui;
using Mapsui.Layers;
using Mapsui.Nts;
using Mapsui.Projections;
using NetTopologySuite.Geometries;
using Wif.Domain.Entities;
using Wif.Domain.Enums;
using Wif.Gis.Rendering;

namespace Wif.Gis.Services;

/// <summary>
/// 管理 Mapsui 地图实例，负责底图切换和矢量图层渲染。
/// 内部坐标系为 WGS84 (EPSG:4326)，显示坐标系为 SphericalMercator (EPSG:3857)。
/// </summary>
public sealed class MapService
{
    private readonly GeometryService _geometryService;
    private readonly FeatureStyleProvider _styleProvider;
    private readonly TileSourceFactory _tileSourceFactory;

    /// <summary>
    /// Mapsui 地图实例，由 assembler 绑定到 MapControl.Map。
    /// </summary>
    public Map Map { get; }

    // 跟踪当前底图图层，用于切换时移除旧底图
    private readonly List<ILayer> _baseMapLayers = [];

    // 跟踪矢量图层，key = GisLayer.Id.ToString()
    private readonly Dictionary<string, ILayer> _vectorLayers = new();

    public MapService(
        GeometryService geometryService,
        FeatureStyleProvider styleProvider,
        TileSourceFactory tileSourceFactory)
    {
        _geometryService = geometryService;
        _styleProvider = styleProvider;
        _tileSourceFactory = tileSourceFactory;

        Map = new Map
        {
            CRS = "EPSG:3857"
        };

        // 默认加载 OSM 底图
        SetOsmBasemap();

        // 默认视图：中国中心 (经度 104, 纬度 35)，缩放级别约 5
        Map.Home = navigator =>
        {
            var center = SphericalMercator.FromLonLat(104.0, 35.0);
            navigator.CenterOnAndZoomTo(center, 4000);
        };
    }

    // ─── 底图管理 ───────────────────────────────────────

    /// <summary>
    /// 切换为 OpenStreetMap 底图。
    /// </summary>
    public void SetOsmBasemap()
    {
        ClearBaseMapLayers();
        var osmLayer = _tileSourceFactory.CreateOsmLayer();
        _baseMapLayers.Add(osmLayer);
        Map.Layers.Insert(0, osmLayer);
    }

    /// <summary>
    /// 切换为天地图底图（含中文标注）。
    /// </summary>
    public void SetTianDiTuBasemap(string apiKey)
    {
        ClearBaseMapLayers();

        var vecLayer = _tileSourceFactory.CreateTianDiTuLayer(apiKey);
        var cvaLayer = _tileSourceFactory.CreateTianDiTuAnnotationLayer(apiKey);

        _baseMapLayers.Add(vecLayer);
        _baseMapLayers.Add(cvaLayer);

        Map.Layers.Insert(0, vecLayer);
        Map.Layers.Insert(1, cvaLayer);
    }

    /// <summary>
    /// 切换为自定义 XYZ 瓦片底图。
    /// </summary>
    public void SetCustomBasemap(string urlTemplate, string name = "自定义底图",
        string[]? subdomains = null)
    {
        ClearBaseMapLayers();
        var layer = _tileSourceFactory.CreateXyzLayer(urlTemplate, name, subdomains);
        _baseMapLayers.Add(layer);
        Map.Layers.Insert(0, layer);
    }

    private void ClearBaseMapLayers()
    {
        foreach (var layer in _baseMapLayers)
        {
            Map.Layers.Remove(layer);
        }
        _baseMapLayers.Clear();
    }

    // ─── 矢量图层管理 ──────────────────────────────────

    /// <summary>
    /// 将 GisLayer（含其 Features）添加到地图。
    /// 如果该图层已存在则先移除再重新添加（刷新）。
    /// </summary>
    public void AddOrUpdateLayer(GisLayer gisLayer)
    {
        // 跳过瓦片类型图层——它们由底图管理
        if (gisLayer.SourceType == GisLayerSourceType.Tile)
            return;

        var layerKey = gisLayer.Id.ToString();

        // 移除已存在的同 ID 图层
        if (_vectorLayers.TryGetValue(layerKey, out var existing))
        {
            Map.Layers.Remove(existing);
            _vectorLayers.Remove(layerKey);
        }

        var mapsuiFeatures = ConvertFeatures(gisLayer);
        var style = _styleProvider.CreateStyle(gisLayer);

        var memoryLayer = new MemoryLayer(gisLayer.Name)
        {
            Features = mapsuiFeatures,
            Style = style,
            Opacity = gisLayer.Opacity,
            Enabled = gisLayer.IsVisible
        };

        _vectorLayers[layerKey] = memoryLayer;
        Map.Layers.Add(memoryLayer);
    }

    /// <summary>
    /// 从地图移除指定 GisLayer。
    /// </summary>
    public void RemoveLayer(Ulid layerId)
    {
        var layerKey = layerId.ToString();
        if (_vectorLayers.TryGetValue(layerKey, out var layer))
        {
            Map.Layers.Remove(layer);
            _vectorLayers.Remove(layerKey);
        }
    }

    /// <summary>
    /// 批量加载多个 GisLayer 到地图（清除现有矢量图层后重新添加）。
    /// </summary>
    public void LoadLayers(IEnumerable<GisLayer> gisLayers)
    {
        // 清除现有矢量图层
        foreach (var layer in _vectorLayers.Values)
        {
            Map.Layers.Remove(layer);
        }
        _vectorLayers.Clear();

        // 按 DisplayOrder 排序后添加
        foreach (var gisLayer in gisLayers.OrderBy(l => l.DisplayOrder))
        {
            AddOrUpdateLayer(gisLayer);
        }
    }

    /// <summary>
    /// 设置图层可见性。
    /// </summary>
    public void SetLayerVisibility(Ulid layerId, bool isVisible)
    {
        var layerKey = layerId.ToString();
        if (_vectorLayers.TryGetValue(layerKey, out var layer))
        {
            layer.Enabled = isVisible;
        }
    }

    /// <summary>
    /// 缩放到指定图层的范围。
    /// </summary>
    public void ZoomToLayer(Ulid layerId)
    {
        var layerKey = layerId.ToString();
        if (_vectorLayers.TryGetValue(layerKey, out var layer) && layer.Extent is not null)
        {
            Map.Navigator.ZoomToBox(layer.Extent);
        }
    }

    /// <summary>
    /// 缩放到所有矢量图层的合并范围。
    /// </summary>
    public void ZoomToAll()
    {
        MRect? combinedExtent = null;
        foreach (var layer in _vectorLayers.Values)
        {
            if (layer.Extent is null) continue;
            combinedExtent = combinedExtent is null
                ? layer.Extent
                : combinedExtent.Join(layer.Extent);
        }

        if (combinedExtent is not null)
        {
            Map.Navigator.ZoomToBox(combinedExtent);
        }
    }

    // ─── 坐标转换 ──────────────────────────────────────

    /// <summary>
    /// 将 NTS Geometry 从 WGS84 投影到 SphericalMercator。
    /// 递归处理所有坐标点。
    /// </summary>
    internal Geometry ProjectToMercator(Geometry wgs84Geometry)
    {
        var projected = wgs84Geometry.Copy();
        ProjectCoordinates(projected.Coordinates);
        // 需要重新构建 geometry 以更新内部缓存
        projected.GeometryChanged();
        return projected;
    }

    private static void ProjectCoordinates(Coordinate[] coordinates)
    {
        foreach (var coord in coordinates)
        {
            var (x, y) = SphericalMercator.FromLonLat(coord.X, coord.Y);
            coord.X = x;
            coord.Y = y;
        }
    }

    // ─── 要素转换 ──────────────────────────────────────

    private List<GeometryFeature> ConvertFeatures(GisLayer gisLayer)
    {
        var features = new List<GeometryFeature>();

        foreach (var gisFeature in gisLayer.Features)
        {
            try
            {
                var ntsGeometry = _geometryService.ParseWkt(gisFeature.GeometryWkt);
                var projected = ProjectToMercator(ntsGeometry);
                var mapsuiFeature = new GeometryFeature(projected);
                features.Add(mapsuiFeature);
            }
            catch
            {
                // 跳过无法解析的要素，避免单个坏数据影响整个图层
            }
        }

        return features;
    }
}
```

- [ ] Step 2: Verify the project compiles:

```bash
dotnet build src/Wif.Gis/Wif.Gis.csproj
```

- [ ] Step 3: Verify the full solution builds:

```bash
dotnet build Wif.sln
```

---

## File Summary

| File | Action | Description |
|------|--------|-------------|
| `src/Wif.Gis/Wif.Gis.csproj` | Modify | Add Mapsui.Avalonia + Mapsui.Nts packages |
| `src/Wif.Domain/Enums/GisLayerSourceType.cs` | Create | Vector/Tile enum for GIS layers |
| `src/Wif.Domain/Entities/GisLayer.cs` | Modify | Add SourceType property |
| `src/Wif.Gis/Rendering/FeatureStyleProvider.cs` | Create | Hex color parsing + VectorStyle mapping |
| `src/Wif.Gis/Services/TileSourceFactory.cs` | Create | OSM, TianDiTu, custom XYZ tile layers |
| `src/Wif.Gis/Services/MapService.cs` | Create | Map instance, layer management, projection |

## Do NOT Modify

These files are managed by the assembler agent:
- `src/Wif.App/Views/GisView.axaml`
- `src/Wif.App/ViewModels/GisViewModel.cs`
- `src/Wif.Core/Services/DependencyInjection.cs`

## Build Verification

After all tasks complete, the following must succeed with no errors:

```bash
dotnet build Wif.sln
dotnet test Wif.sln
```

## API Surface for Assembler

The assembler agent needs these public members to wire everything up:

```csharp
// MapService — register as singleton
MapService.Map                          // Mapsui.Map — bind to MapControl.Map
MapService.AddOrUpdateLayer(GisLayer)   // add vector layer
MapService.RemoveLayer(Ulid)            // remove vector layer
MapService.LoadLayers(IEnumerable<GisLayer>) // batch load
MapService.SetLayerVisibility(Ulid, bool) // toggle visibility
MapService.ZoomToLayer(Ulid)            // zoom to layer extent
MapService.ZoomToAll()                  // zoom to all layers
MapService.SetOsmBasemap()              // switch to OSM
MapService.SetTianDiTuBasemap(string)   // switch to TianDiTu (needs API key)
MapService.SetCustomBasemap(string, string?, string[]?) // switch to custom XYZ

// DI registration (assembler writes this in DependencyInjection.cs):
services.AddSingleton<GeometryService>();
services.AddSingleton<FeatureStyleProvider>();
services.AddSingleton<TileSourceFactory>();
services.AddSingleton<MapService>();
```

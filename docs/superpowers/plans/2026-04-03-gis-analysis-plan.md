# gis-analysis Implementation Plan

> **For agentic workers:** Use this plan to implement the gis-analysis module task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Provide spatial analysis (buffer/intersection/union/difference), measurement tools, and coordinate system management.

**Architecture:** SpatialAnalysisService for batch operations on GisLayer entities, MeasurementService for planar/geodesic calculations, CrsManager+CoordinateTransformer for projection support via ProjNET, MeasureTool for interactive map measurement (implements IMapTool), and AnalysisDialog for user-facing analysis parameter input.

**Tech Stack:** NetTopologySuite 2.6.0, ProjNET, Avalonia 11.2.1, CommunityToolkit.Mvvm 8.4.2

---

## File Map

| Action | File | Responsibility |
|--------|------|----------------|
| Create | `src/Wif.Gis/Analysis/AnalysisResult.cs` | Result model: output layer + statistics |
| Create | `src/Wif.Gis/Analysis/SpatialAnalysisService.cs` | Buffer, Intersection, Union, Difference on layers |
| Create | `src/Wif.Gis/Analysis/MeasurementService.cs` | Planar and geodesic distance/area measurement |
| Create | `src/Wif.Gis/Projection/CrsManager.cs` | CRS registry, EPSG lookup, custom WKT support |
| Create | `src/Wif.Gis/Projection/CoordinateTransformer.cs` | Transform geometries between CRS via ProjNET |
| Create | `src/Wif.Gis/Editing/MeasureTool.cs` | Interactive measurement tool (IMapTool) |
| Create | `src/Wif.App/Views/AnalysisDialog.axaml` | Dialog for analysis parameters |
| Create | `src/Wif.App/Views/AnalysisDialog.axaml.cs` | Dialog code-behind |
| Create | `src/Wif.App/ViewModels/AnalysisDialogViewModel.cs` | Validation, calls analysis, writes result |
| Modify | `src/Wif.Gis/Wif.Gis.csproj` | Add ProjNET NuGet package |
| Modify | `src/Wif.Domain/Entities/GisLayer.cs` | Add CrsCode property |

**IMPORTANT:** Do NOT modify `GisView.axaml`, `GisViewModel.cs`, or `DependencyInjection.cs` — the assembler agent handles those.

---

### Task 1: Add ProjNET Package to Wif.Gis

**Files:**
- Modify: `src/Wif.Gis/Wif.Gis.csproj`

- [ ] **Step 1: Add ProjNET NuGet package reference**

Add the ProjNET package to the existing `<ItemGroup>` containing NuGet packages:

```xml
<PackageReference Include="ProjNET" Version="2.0.0" />
```

The full `<ItemGroup>` should look like:

```xml
<ItemGroup>
  <PackageReference Include="NetTopologySuite" Version="2.6.0" />
  <PackageReference Include="NetTopologySuite.IO.GeoJSON" Version="4.0.0" />
  <PackageReference Include="NetTopologySuite.IO.ShapeFile" Version="2.1.0" />
  <PackageReference Include="NetTopologySuite.Features" Version="2.2.0" />
  <PackageReference Include="ProjNET" Version="2.0.0" />
</ItemGroup>
```

- [ ] **Step 2: Verify build**

```bash
dotnet build src/Wif.Gis/Wif.Gis.csproj
```

---

### Task 2: Add CrsCode Property to GisLayer

**Files:**
- Modify: `src/Wif.Domain/Entities/GisLayer.cs`

- [ ] **Step 1: Add CrsCode property**

Add the `CrsCode` property after the `Opacity` property:

```csharp
public string CrsCode { get; set; } = "EPSG:4326";
```

The full file should be:

```csharp
namespace Wif.Domain.Entities;

public class GisLayer
{
    public Ulid Id { get; init; }
    public required string Name { get; set; }
    public string? Description { get; set; }
    public bool IsVisible { get; set; } = true;
    public int DisplayOrder { get; set; }
    public string FillColor { get; set; } = "#3388FF40";
    public string StrokeColor { get; set; } = "#3388FF";
    public double StrokeWidth { get; set; } = 2.0;
    public double Opacity { get; set; } = 1.0;
    public string CrsCode { get; set; } = "EPSG:4326";
    public DateTimeOffset CreatedAt { get; init; }
    public DateTimeOffset UpdatedAt { get; set; }

    // Navigation
    public List<GisFeature> Features { get; init; } = [];
}
```

- [ ] **Step 2: Verify build**

```bash
dotnet build src/Wif.Domain/Wif.Domain.csproj
```

---

### Task 3: Create AnalysisResult Model

**Files:**
- Create: `src/Wif.Gis/Analysis/AnalysisResult.cs`

- [ ] **Step 1: Create the AnalysisResult record**

```csharp
using Wif.Domain.Entities;

namespace Wif.Gis.Analysis;

/// <summary>
/// 空间分析操作的返回结果。
/// </summary>
public sealed record AnalysisResult
{
    /// <summary>包含结果要素的输出图层。</summary>
    public required GisLayer OutputLayer { get; init; }

    /// <summary>结果要素数量。</summary>
    public int FeatureCount { get; init; }

    /// <summary>所有结果要素的总面积（坐标系单位）。</summary>
    public double TotalArea { get; init; }

    /// <summary>所有结果要素的总长度（坐标系单位）。</summary>
    public double TotalLength { get; init; }

    /// <summary>分析执行耗时。</summary>
    public TimeSpan ExecutionTime { get; init; }

    /// <summary>人类可读的统计摘要（中文）。</summary>
    public string Summary { get; init; } = string.Empty;
}
```

- [ ] **Step 2: Verify build**

```bash
dotnet build src/Wif.Gis/Wif.Gis.csproj
```

---

### Task 4: Create SpatialAnalysisService

**Files:**
- Create: `src/Wif.Gis/Analysis/SpatialAnalysisService.cs`

- [ ] **Step 1: Create the service with all four analysis operations**

```csharp
using System.Diagnostics;
using NetTopologySuite.Geometries;
using NetTopologySuite.Operation.Union;
using Wif.Domain.Entities;
using Wif.Gis.Services;

namespace Wif.Gis.Analysis;

public sealed class SpatialAnalysisService
{
    private readonly GeometryService _geometry;

    public SpatialAnalysisService(GeometryService geometry)
    {
        _geometry = geometry;
    }

    /// <summary>
    /// 缓冲区分析：为图层中每个要素创建指定距离的缓冲区。
    /// </summary>
    /// <param name="inputLayer">输入图层。</param>
    /// <param name="distance">缓冲距离（坐标系单位，度或米取决于 CRS）。</param>
    /// <param name="resultLayerName">输出图层名称。</param>
    /// <param name="mergeResults">是否将所有缓冲区合并为一个要素。</param>
    /// <param name="quadrantSegments">圆弧近似的象限分段数，默认 8。</param>
    public AnalysisResult BufferAnalysis(
        GisLayer inputLayer,
        double distance,
        string resultLayerName,
        bool mergeResults = false,
        int quadrantSegments = 8)
    {
        var sw = Stopwatch.StartNew();
        var now = DateTimeOffset.UtcNow;
        var outputLayer = new GisLayer
        {
            Id = Ulid.NewUlid(),
            Name = resultLayerName,
            Description = $"缓冲区分析 - 距离: {distance}",
            FillColor = "#FF990040",
            StrokeColor = "#FF9900",
            CrsCode = inputLayer.CrsCode,
            CreatedAt = now,
            UpdatedAt = now
        };

        var bufferedGeometries = new List<Geometry>();

        foreach (var feature in inputLayer.Features)
        {
            var geom = _geometry.ParseWkt(feature.GeometryWkt);
            var buffered = geom.Buffer(distance, quadrantSegments);
            bufferedGeometries.Add(buffered);
        }

        if (mergeResults && bufferedGeometries.Count > 0)
        {
            var merged = CascadedPolygonUnion.Union(bufferedGeometries);
            outputLayer.Features.Add(new GisFeature
            {
                Id = Ulid.NewUlid(),
                LayerId = outputLayer.Id,
                GeometryWkt = _geometry.ToWkt(merged),
                GeometryType = merged.GeometryType,
                Name = "合并缓冲区",
                CreatedAt = now,
                UpdatedAt = now
            });
        }
        else
        {
            for (int i = 0; i < bufferedGeometries.Count; i++)
            {
                var buffered = bufferedGeometries[i];
                var sourceName = inputLayer.Features[i].Name ?? $"要素{i + 1}";
                outputLayer.Features.Add(new GisFeature
                {
                    Id = Ulid.NewUlid(),
                    LayerId = outputLayer.Id,
                    GeometryWkt = _geometry.ToWkt(buffered),
                    GeometryType = buffered.GeometryType,
                    Name = $"{sourceName}_缓冲",
                    CreatedAt = now,
                    UpdatedAt = now
                });
            }
        }

        sw.Stop();

        var totalArea = outputLayer.Features.Sum(f => _geometry.Area(f.GeometryWkt));
        var totalLength = outputLayer.Features.Sum(f => _geometry.Length(f.GeometryWkt));

        return new AnalysisResult
        {
            OutputLayer = outputLayer,
            FeatureCount = outputLayer.Features.Count,
            TotalArea = totalArea,
            TotalLength = totalLength,
            ExecutionTime = sw.Elapsed,
            Summary = $"缓冲区分析完成: 输入 {inputLayer.Features.Count} 个要素, " +
                      $"输出 {outputLayer.Features.Count} 个要素, " +
                      $"总面积 {totalArea:F4}, 耗时 {sw.ElapsedMilliseconds}ms"
        };
    }

    /// <summary>
    /// 交集分析：计算两个图层中要素的几何交集。
    /// </summary>
    public AnalysisResult IntersectionAnalysis(
        GisLayer layerA,
        GisLayer layerB,
        string resultLayerName)
    {
        var sw = Stopwatch.StartNew();
        var now = DateTimeOffset.UtcNow;
        var outputLayer = new GisLayer
        {
            Id = Ulid.NewUlid(),
            Name = resultLayerName,
            Description = $"交集分析: {layerA.Name} ∩ {layerB.Name}",
            FillColor = "#E91E6340",
            StrokeColor = "#E91E63",
            CrsCode = layerA.CrsCode,
            CreatedAt = now,
            UpdatedAt = now
        };

        foreach (var featureA in layerA.Features)
        {
            var geomA = _geometry.ParseWkt(featureA.GeometryWkt);

            foreach (var featureB in layerB.Features)
            {
                var geomB = _geometry.ParseWkt(featureB.GeometryWkt);

                if (!geomA.Intersects(geomB))
                    continue;

                var intersection = geomA.Intersection(geomB);
                if (intersection.IsEmpty)
                    continue;

                var nameA = featureA.Name ?? featureA.Id.ToString()[..8];
                var nameB = featureB.Name ?? featureB.Id.ToString()[..8];
                outputLayer.Features.Add(new GisFeature
                {
                    Id = Ulid.NewUlid(),
                    LayerId = outputLayer.Id,
                    GeometryWkt = _geometry.ToWkt(intersection),
                    GeometryType = intersection.GeometryType,
                    Name = $"{nameA} ∩ {nameB}",
                    CreatedAt = now,
                    UpdatedAt = now
                });
            }
        }

        sw.Stop();

        var totalArea = outputLayer.Features.Sum(f => _geometry.Area(f.GeometryWkt));
        var totalLength = outputLayer.Features.Sum(f => _geometry.Length(f.GeometryWkt));

        return new AnalysisResult
        {
            OutputLayer = outputLayer,
            FeatureCount = outputLayer.Features.Count,
            TotalArea = totalArea,
            TotalLength = totalLength,
            ExecutionTime = sw.Elapsed,
            Summary = $"交集分析完成: {layerA.Name}({layerA.Features.Count}个) × {layerB.Name}({layerB.Features.Count}个), " +
                      $"输出 {outputLayer.Features.Count} 个交集要素, 耗时 {sw.ElapsedMilliseconds}ms"
        };
    }

    /// <summary>
    /// 合并分析：将图层中所有要素合并为一个几何体。
    /// </summary>
    public AnalysisResult UnionAnalysis(
        GisLayer inputLayer,
        string resultLayerName)
    {
        var sw = Stopwatch.StartNew();
        var now = DateTimeOffset.UtcNow;
        var outputLayer = new GisLayer
        {
            Id = Ulid.NewUlid(),
            Name = resultLayerName,
            Description = $"合并分析: {inputLayer.Name}",
            FillColor = "#4CAF5040",
            StrokeColor = "#4CAF50",
            CrsCode = inputLayer.CrsCode,
            CreatedAt = now,
            UpdatedAt = now
        };

        if (inputLayer.Features.Count > 0)
        {
            var geometries = inputLayer.Features
                .Select(f => _geometry.ParseWkt(f.GeometryWkt))
                .ToList();

            var merged = CascadedPolygonUnion.Union(geometries);

            outputLayer.Features.Add(new GisFeature
            {
                Id = Ulid.NewUlid(),
                LayerId = outputLayer.Id,
                GeometryWkt = _geometry.ToWkt(merged),
                GeometryType = merged.GeometryType,
                Name = $"{inputLayer.Name}_合并",
                CreatedAt = now,
                UpdatedAt = now
            });
        }

        sw.Stop();

        var totalArea = outputLayer.Features.Sum(f => _geometry.Area(f.GeometryWkt));
        var totalLength = outputLayer.Features.Sum(f => _geometry.Length(f.GeometryWkt));

        return new AnalysisResult
        {
            OutputLayer = outputLayer,
            FeatureCount = outputLayer.Features.Count,
            TotalArea = totalArea,
            TotalLength = totalLength,
            ExecutionTime = sw.Elapsed,
            Summary = $"合并分析完成: 输入 {inputLayer.Features.Count} 个要素, " +
                      $"合并为 {outputLayer.Features.Count} 个要素, " +
                      $"总面积 {totalArea:F4}, 耗时 {sw.ElapsedMilliseconds}ms"
        };
    }

    /// <summary>
    /// 差异分析：计算 layerA 中要素去除与 layerB 重叠部分后的结果。
    /// </summary>
    public AnalysisResult DifferenceAnalysis(
        GisLayer layerA,
        GisLayer layerB,
        string resultLayerName)
    {
        var sw = Stopwatch.StartNew();
        var now = DateTimeOffset.UtcNow;
        var outputLayer = new GisLayer
        {
            Id = Ulid.NewUlid(),
            Name = resultLayerName,
            Description = $"差异分析: {layerA.Name} - {layerB.Name}",
            FillColor = "#9C27B040",
            StrokeColor = "#9C27B0",
            CrsCode = layerA.CrsCode,
            CreatedAt = now,
            UpdatedAt = now
        };

        // Build a union of all geometries in layerB for erasing
        Geometry? eraseGeom = null;
        if (layerB.Features.Count > 0)
        {
            var bGeometries = layerB.Features
                .Select(f => _geometry.ParseWkt(f.GeometryWkt))
                .ToList();
            eraseGeom = CascadedPolygonUnion.Union(bGeometries);
        }

        foreach (var featureA in layerA.Features)
        {
            var geomA = _geometry.ParseWkt(featureA.GeometryWkt);

            var result = eraseGeom is not null ? geomA.Difference(eraseGeom) : geomA;

            if (result.IsEmpty)
                continue;

            var sourceName = featureA.Name ?? featureA.Id.ToString()[..8];
            outputLayer.Features.Add(new GisFeature
            {
                Id = Ulid.NewUlid(),
                LayerId = outputLayer.Id,
                GeometryWkt = _geometry.ToWkt(result),
                GeometryType = result.GeometryType,
                Name = $"{sourceName}_差异",
                CreatedAt = now,
                UpdatedAt = now
            });
        }

        sw.Stop();

        var totalArea = outputLayer.Features.Sum(f => _geometry.Area(f.GeometryWkt));
        var totalLength = outputLayer.Features.Sum(f => _geometry.Length(f.GeometryWkt));

        return new AnalysisResult
        {
            OutputLayer = outputLayer,
            FeatureCount = outputLayer.Features.Count,
            TotalArea = totalArea,
            TotalLength = totalLength,
            ExecutionTime = sw.Elapsed,
            Summary = $"差异分析完成: {layerA.Name}({layerA.Features.Count}个) - {layerB.Name}({layerB.Features.Count}个), " +
                      $"输出 {outputLayer.Features.Count} 个要素, 耗时 {sw.ElapsedMilliseconds}ms"
        };
    }
}
```

- [ ] **Step 2: Verify build**

```bash
dotnet build src/Wif.Gis/Wif.Gis.csproj
```

---

### Task 5: Create MeasurementService

**Files:**
- Create: `src/Wif.Gis/Analysis/MeasurementService.cs`

- [ ] **Step 1: Create MeasureMode enum and MeasurementResult record**

```csharp
using NetTopologySuite.Geometries;
using Wif.Gis.Services;

namespace Wif.Gis.Analysis;

/// <summary>测量模式。</summary>
public enum MeasureMode
{
    /// <summary>平面测量（直接使用 NTS 坐标运算）。</summary>
    Planar,
    /// <summary>大地测量（WGS84 椭球体 Vincenty 公式）。</summary>
    Geodesic
}

/// <summary>测量结果。</summary>
public sealed record MeasurementResult
{
    /// <summary>测量值（距离或面积）。</summary>
    public double Value { get; init; }

    /// <summary>单位（米、平方米、度等）。</summary>
    public string Unit { get; init; } = string.Empty;

    /// <summary>人类可读的格式化文本（中文）。</summary>
    public string DisplayText { get; init; } = string.Empty;
}

/// <summary>
/// 提供平面和大地测量服务。
/// 平面模式直接使用 NTS 几何运算；大地模式使用 Vincenty 公式在 WGS84 椭球体上计算。
/// </summary>
public sealed class MeasurementService
{
    // WGS84 椭球体参数
    private const double WGS84_A = 6378137.0;           // 长半轴 (m)
    private const double WGS84_B = 6356752.314245;      // 短半轴 (m)
    private const double WGS84_F = 1.0 / 298.257223563; // 扁率

    private readonly GeometryService _geometry;

    public MeasurementService(GeometryService geometry)
    {
        _geometry = geometry;
    }

    /// <summary>
    /// 计算两点之间的距离。
    /// </summary>
    /// <param name="wktA">点A的WKT。</param>
    /// <param name="wktB">点B的WKT。</param>
    /// <param name="mode">测量模式。</param>
    public MeasurementResult MeasureDistance(string wktA, string wktB, MeasureMode mode)
    {
        if (mode == MeasureMode.Planar)
        {
            var distance = _geometry.Distance(wktA, wktB);
            return new MeasurementResult
            {
                Value = distance,
                Unit = "度",
                DisplayText = $"平面距离: {distance:F6} 度"
            };
        }

        // Geodesic: extract coordinates and use Vincenty
        var geomA = _geometry.ParseWkt(wktA);
        var geomB = _geometry.ParseWkt(wktB);
        var coordA = geomA.Centroid.Coordinate;
        var coordB = geomB.Centroid.Coordinate;

        var distMeters = VincentyDistance(coordA.Y, coordA.X, coordB.Y, coordB.X);

        return FormatDistance(distMeters);
    }

    /// <summary>
    /// 计算折线路径的总长度。
    /// </summary>
    /// <param name="points">按顺序排列的点坐标 (lon, lat)。</param>
    /// <param name="mode">测量模式。</param>
    public MeasurementResult MeasurePathLength(IReadOnlyList<Coordinate> points, MeasureMode mode)
    {
        if (points.Count < 2)
        {
            return new MeasurementResult
            {
                Value = 0,
                Unit = mode == MeasureMode.Planar ? "度" : "米",
                DisplayText = "路径点不足，无法测量"
            };
        }

        if (mode == MeasureMode.Planar)
        {
            double total = 0;
            for (int i = 1; i < points.Count; i++)
            {
                var dx = points[i].X - points[i - 1].X;
                var dy = points[i].Y - points[i - 1].Y;
                total += Math.Sqrt(dx * dx + dy * dy);
            }

            return new MeasurementResult
            {
                Value = total,
                Unit = "度",
                DisplayText = $"平面路径长度: {total:F6} 度"
            };
        }

        // Geodesic
        double totalMeters = 0;
        for (int i = 1; i < points.Count; i++)
        {
            totalMeters += VincentyDistance(
                points[i - 1].Y, points[i - 1].X,
                points[i].Y, points[i].X);
        }

        return FormatDistance(totalMeters);
    }

    /// <summary>
    /// 计算多边形的面积。
    /// </summary>
    /// <param name="wkt">多边形的WKT。</param>
    /// <param name="mode">测量模式。</param>
    public MeasurementResult MeasureArea(string wkt, MeasureMode mode)
    {
        if (mode == MeasureMode.Planar)
        {
            var area = _geometry.Area(wkt);
            return new MeasurementResult
            {
                Value = area,
                Unit = "平方度",
                DisplayText = $"平面面积: {area:F6} 平方度"
            };
        }

        // Geodesic: approximate using spherical excess on WGS84 sphere
        var geom = _geometry.ParseWkt(wkt);
        var areaM2 = SphericalPolygonArea(geom);

        return FormatArea(areaM2);
    }

    /// <summary>
    /// 计算多边形面积（由坐标序列构成的环）。
    /// </summary>
    /// <param name="ring">闭合环的坐标序列 (lon, lat)。</param>
    /// <param name="mode">测量模式。</param>
    public MeasurementResult MeasureRingArea(IReadOnlyList<Coordinate> ring, MeasureMode mode)
    {
        if (ring.Count < 3)
        {
            return new MeasurementResult
            {
                Value = 0,
                Unit = mode == MeasureMode.Planar ? "平方度" : "平方米",
                DisplayText = "点数不足，无法计算面积"
            };
        }

        // Build WKT polygon from ring
        var coords = ring.Select(c => new Coordinate(c.X, c.Y)).ToList();
        // Ensure closed
        if (coords[0].X != coords[^1].X || coords[0].Y != coords[^1].Y)
            coords.Add(new Coordinate(coords[0].X, coords[0].Y));

        var factory = new GeometryFactory();
        var polygon = factory.CreatePolygon(coords.ToArray());
        var wkt = _geometry.ToWkt(polygon);

        return MeasureArea(wkt, mode);
    }

    // ─── Vincenty Distance ──────────────────────────────────

    /// <summary>
    /// Vincenty 公式计算椭球体上两点间的大地线距离（米）。
    /// </summary>
    private static double VincentyDistance(double lat1Deg, double lon1Deg, double lat2Deg, double lon2Deg)
    {
        var lat1 = DegreesToRadians(lat1Deg);
        var lon1 = DegreesToRadians(lon1Deg);
        var lat2 = DegreesToRadians(lat2Deg);
        var lon2 = DegreesToRadians(lon2Deg);

        var U1 = Math.Atan((1 - WGS84_F) * Math.Tan(lat1));
        var U2 = Math.Atan((1 - WGS84_F) * Math.Tan(lat2));
        var sinU1 = Math.Sin(U1); var cosU1 = Math.Cos(U1);
        var sinU2 = Math.Sin(U2); var cosU2 = Math.Cos(U2);

        var lambda = lon2 - lon1;
        var lambdaPrev = double.MaxValue;

        double sinSigma = 0, cosSigma = 0, sigma = 0;
        double sinAlpha = 0, cos2Alpha = 0, cos2SigmaM = 0;
        double C;

        for (int iter = 0; iter < 200; iter++)
        {
            var sinLambda = Math.Sin(lambda);
            var cosLambda = Math.Cos(lambda);

            sinSigma = Math.Sqrt(
                Math.Pow(cosU2 * sinLambda, 2) +
                Math.Pow(cosU1 * sinU2 - sinU1 * cosU2 * cosLambda, 2));

            if (sinSigma == 0) return 0; // co-incident points

            cosSigma = sinU1 * sinU2 + cosU1 * cosU2 * cosLambda;
            sigma = Math.Atan2(sinSigma, cosSigma);

            sinAlpha = cosU1 * cosU2 * sinLambda / sinSigma;
            cos2Alpha = 1 - sinAlpha * sinAlpha;

            cos2SigmaM = cos2Alpha != 0
                ? cosSigma - 2 * sinU1 * sinU2 / cos2Alpha
                : 0;

            C = WGS84_F / 16 * cos2Alpha * (4 + WGS84_F * (4 - 3 * cos2Alpha));

            lambdaPrev = lambda;
            lambda = (lon2 - lon1) + (1 - C) * WGS84_F * sinAlpha *
                (sigma + C * sinSigma * (cos2SigmaM + C * cosSigma * (-1 + 2 * cos2SigmaM * cos2SigmaM)));

            if (Math.Abs(lambda - lambdaPrev) < 1e-12)
                break;
        }

        var uSq = cos2Alpha * (WGS84_A * WGS84_A - WGS84_B * WGS84_B) / (WGS84_B * WGS84_B);
        var A2 = 1 + uSq / 16384 * (4096 + uSq * (-768 + uSq * (320 - 175 * uSq)));
        var B2 = uSq / 1024 * (256 + uSq * (-128 + uSq * (74 - 47 * uSq)));

        var deltaSigma = B2 * sinSigma * (cos2SigmaM + B2 / 4 * (
            cosSigma * (-1 + 2 * cos2SigmaM * cos2SigmaM) -
            B2 / 6 * cos2SigmaM * (-3 + 4 * sinSigma * sinSigma) *
            (-3 + 4 * cos2SigmaM * cos2SigmaM)));

        return WGS84_B * A2 * (sigma - deltaSigma);
    }

    // ─── Spherical Polygon Area ──────────────────────────

    /// <summary>
    /// 使用球面过剩法近似计算多边形在球面上的面积（平方米）。
    /// </summary>
    private static double SphericalPolygonArea(Geometry geometry)
    {
        if (geometry is not Polygon polygon)
            return 0;

        var ring = polygon.ExteriorRing.Coordinates;
        if (ring.Length < 3)
            return 0;

        // Spherical excess method (Girard's theorem on sphere of radius WGS84_A)
        double totalAngle = 0;
        int n = ring.Length - 1; // last point == first point

        for (int i = 0; i < n; i++)
        {
            var p0 = ring[(i - 1 + n) % n];
            var p1 = ring[i];
            var p2 = ring[(i + 1) % n];

            var bearing1 = InitialBearing(p1.Y, p1.X, p0.Y, p0.X);
            var bearing2 = InitialBearing(p1.Y, p1.X, p2.Y, p2.X);

            var angle = bearing2 - bearing1;
            if (angle < 0) angle += 2 * Math.PI;
            if (angle > 2 * Math.PI) angle -= 2 * Math.PI;

            totalAngle += angle;
        }

        var sphericalExcess = Math.Abs(totalAngle - (n - 2) * Math.PI);
        return sphericalExcess * WGS84_A * WGS84_A;
    }

    private static double InitialBearing(double lat1Deg, double lon1Deg, double lat2Deg, double lon2Deg)
    {
        var lat1 = DegreesToRadians(lat1Deg);
        var lat2 = DegreesToRadians(lat2Deg);
        var dLon = DegreesToRadians(lon2Deg - lon1Deg);

        var y = Math.Sin(dLon) * Math.Cos(lat2);
        var x = Math.Cos(lat1) * Math.Sin(lat2) - Math.Sin(lat1) * Math.Cos(lat2) * Math.Cos(dLon);
        return Math.Atan2(y, x);
    }

    // ─── Formatting ─────────────────────────────────────

    private static MeasurementResult FormatDistance(double meters)
    {
        if (meters >= 1000)
        {
            var km = meters / 1000;
            return new MeasurementResult
            {
                Value = meters,
                Unit = "米",
                DisplayText = $"大地距离: {km:F3} 公里 ({meters:F1} 米)"
            };
        }

        return new MeasurementResult
        {
            Value = meters,
            Unit = "米",
            DisplayText = $"大地距离: {meters:F2} 米"
        };
    }

    private static MeasurementResult FormatArea(double squareMeters)
    {
        if (squareMeters >= 1_000_000)
        {
            var km2 = squareMeters / 1_000_000;
            return new MeasurementResult
            {
                Value = squareMeters,
                Unit = "平方米",
                DisplayText = $"大地面积: {km2:F4} 平方公里 ({squareMeters:F0} 平方米)"
            };
        }

        return new MeasurementResult
        {
            Value = squareMeters,
            Unit = "平方米",
            DisplayText = $"大地面积: {squareMeters:F2} 平方米"
        };
    }

    private static double DegreesToRadians(double degrees) => degrees * Math.PI / 180.0;
}
```

- [ ] **Step 2: Verify build**

```bash
dotnet build src/Wif.Gis/Wif.Gis.csproj
```

---

### Task 6: Create CrsManager

**Files:**
- Create: `src/Wif.Gis/Projection/CrsManager.cs`

- [ ] **Step 1: Create the CRS registry with built-in definitions**

```csharp
using ProjNet.CoordinateSystems;

namespace Wif.Gis.Projection;

/// <summary>
/// 坐标参考系统管理器。
/// 提供常用 CRS 的内置注册表和自定义 WKT 定义支持。
/// </summary>
public sealed class CrsManager
{
    private readonly CoordinateSystemFactory _factory = new();
    private readonly Dictionary<string, ICoordinateSystem> _cache = new(StringComparer.OrdinalIgnoreCase);

    /// <summary>所有已注册的 CRS 代码。</summary>
    public IReadOnlyCollection<string> RegisteredCodes => _cache.Keys;

    public CrsManager()
    {
        RegisterBuiltInSystems();
    }

    /// <summary>
    /// 按 EPSG 代码获取坐标参考系统。
    /// </summary>
    /// <param name="code">EPSG 代码，例如 "EPSG:4326"。</param>
    /// <returns>坐标系统，未找到时返回 null。</returns>
    public ICoordinateSystem? GetCrs(string code)
    {
        return _cache.TryGetValue(NormalizeCode(code), out var crs) ? crs : null;
    }

    /// <summary>
    /// 注册自定义 CRS（通过 WKT 定义）。
    /// </summary>
    /// <param name="code">代码标识，例如 "EPSG:32650"。</param>
    /// <param name="wkt">OGC WKT 格式的坐标系统定义。</param>
    public void RegisterFromWkt(string code, string wkt)
    {
        var cs = _factory.CreateFromWkt(wkt);
        _cache[NormalizeCode(code)] = (ICoordinateSystem)cs;
    }

    /// <summary>
    /// 检查指定代码是否已注册。
    /// </summary>
    public bool IsRegistered(string code) => _cache.ContainsKey(NormalizeCode(code));

    private static string NormalizeCode(string code) => code.Trim().ToUpperInvariant();

    private void RegisterBuiltInSystems()
    {
        // EPSG:4326 - WGS 84 地理坐标系
        Register("EPSG:4326", """
            GEOGCS["WGS 84",
                DATUM["WGS_1984",
                    SPHEROID["WGS 84",6378137,298.257223563,AUTHORITY["EPSG","7030"]],
                    AUTHORITY["EPSG","6326"]],
                PRIMEM["Greenwich",0,AUTHORITY["EPSG","8901"]],
                UNIT["degree",0.0174532925199433,AUTHORITY["EPSG","9122"]],
                AUTHORITY["EPSG","4326"]]
            """);

        // EPSG:3857 - Web Mercator (伪墨卡托)
        Register("EPSG:3857", """
            PROJCS["WGS 84 / Pseudo-Mercator",
                GEOGCS["WGS 84",
                    DATUM["WGS_1984",
                        SPHEROID["WGS 84",6378137,298.257223563,AUTHORITY["EPSG","7030"]],
                        AUTHORITY["EPSG","6326"]],
                    PRIMEM["Greenwich",0,AUTHORITY["EPSG","8901"]],
                    UNIT["degree",0.0174532925199433,AUTHORITY["EPSG","9122"]],
                    AUTHORITY["EPSG","4326"]],
                PROJECTION["Mercator_1SP"],
                PARAMETER["central_meridian",0],
                PARAMETER["scale_factor",1],
                PARAMETER["false_easting",0],
                PARAMETER["false_northing",0],
                UNIT["metre",1,AUTHORITY["EPSG","9001"]],
                AUTHORITY["EPSG","3857"]]
            """);

        // EPSG:4490 - CGCS2000 (中国国家大地坐标系)
        Register("EPSG:4490", """
            GEOGCS["China Geodetic Coordinate System 2000",
                DATUM["China_2000",
                    SPHEROID["CGCS2000",6378137,298.257222101,AUTHORITY["EPSG","1024"]],
                    AUTHORITY["EPSG","1043"]],
                PRIMEM["Greenwich",0,AUTHORITY["EPSG","8901"]],
                UNIT["degree",0.0174532925199433,AUTHORITY["EPSG","9122"]],
                AUTHORITY["EPSG","4490"]]
            """);

        // Common UTM zones for China (UTM 49N-51N)
        RegisterUtmZone(49);
        RegisterUtmZone(50);
        RegisterUtmZone(51);
    }

    private void RegisterUtmZone(int zone)
    {
        var centralMeridian = zone * 6 - 183;
        var code = $"EPSG:{32600 + zone}";
        var wkt = $"""
            PROJCS["WGS 84 / UTM zone {zone}N",
                GEOGCS["WGS 84",
                    DATUM["WGS_1984",
                        SPHEROID["WGS 84",6378137,298.257223563,AUTHORITY["EPSG","7030"]],
                        AUTHORITY["EPSG","6326"]],
                    PRIMEM["Greenwich",0,AUTHORITY["EPSG","8901"]],
                    UNIT["degree",0.0174532925199433,AUTHORITY["EPSG","9122"]],
                    AUTHORITY["EPSG","4326"]],
                PROJECTION["Transverse_Mercator"],
                PARAMETER["latitude_of_origin",0],
                PARAMETER["central_meridian",{centralMeridian}],
                PARAMETER["scale_factor",0.9996],
                PARAMETER["false_easting",500000],
                PARAMETER["false_northing",0],
                UNIT["metre",1,AUTHORITY["EPSG","9001"]],
                AUTHORITY["EPSG","{32600 + zone}"]]
            """;
        Register(code, wkt);
    }

    private void Register(string code, string wkt)
    {
        var cs = _factory.CreateFromWkt(wkt);
        _cache[NormalizeCode(code)] = (ICoordinateSystem)cs;
    }
}
```

- [ ] **Step 2: Verify build**

```bash
dotnet build src/Wif.Gis/Wif.Gis.csproj
```

---

### Task 7: Create CoordinateTransformer

**Files:**
- Create: `src/Wif.Gis/Projection/CoordinateTransformer.cs`

- [ ] **Step 1: Create the transformer that converts NTS geometries between CRS**

```csharp
using NetTopologySuite.Geometries;
using ProjNet.CoordinateSystems;
using ProjNet.CoordinateSystems.Transformations;

namespace Wif.Gis.Projection;

/// <summary>
/// 坐标转换器。
/// 使用 ProjNET 在不同坐标参考系统之间转换 NTS Geometry 对象。
/// </summary>
public sealed class CoordinateTransformer
{
    private readonly CrsManager _crsManager;
    private readonly CoordinateTransformationFactory _transformFactory = new();

    public CoordinateTransformer(CrsManager crsManager)
    {
        _crsManager = crsManager;
    }

    /// <summary>
    /// 将几何对象从源坐标系转换到目标坐标系。
    /// </summary>
    /// <param name="geometry">输入几何对象。</param>
    /// <param name="sourceCrsCode">源坐标系代码，例如 "EPSG:4326"。</param>
    /// <param name="targetCrsCode">目标坐标系代码，例如 "EPSG:3857"。</param>
    /// <returns>转换后的几何对象。</returns>
    public Geometry Transform(Geometry geometry, string sourceCrsCode, string targetCrsCode)
    {
        if (string.Equals(sourceCrsCode, targetCrsCode, StringComparison.OrdinalIgnoreCase))
            return geometry.Copy();

        var sourceCrs = _crsManager.GetCrs(sourceCrsCode)
            ?? throw new ArgumentException($"未知的源坐标系: {sourceCrsCode}", nameof(sourceCrsCode));
        var targetCrs = _crsManager.GetCrs(targetCrsCode)
            ?? throw new ArgumentException($"未知的目标坐标系: {targetCrsCode}", nameof(targetCrsCode));

        var transformation = _transformFactory.CreateFromCoordinateSystems(sourceCrs, targetCrs);
        var mathTransform = transformation.MathTransform;

        return TransformGeometry(geometry, mathTransform);
    }

    /// <summary>
    /// 将 WKT 几何字符串从源坐标系转换到目标坐标系，返回新的 WKT。
    /// </summary>
    public string TransformWkt(string wkt, string sourceCrsCode, string targetCrsCode,
        Services.GeometryService geometryService)
    {
        var geom = geometryService.ParseWkt(wkt);
        var transformed = Transform(geom, sourceCrsCode, targetCrsCode);
        return geometryService.ToWkt(transformed);
    }

    private static Geometry TransformGeometry(Geometry geometry, IMathTransform transform)
    {
        var factory = geometry.Factory;

        return geometry switch
        {
            Point p => factory.CreatePoint(TransformCoordinate(p.Coordinate, transform)),
            LineString ls => factory.CreateLineString(TransformCoordinates(ls.Coordinates, transform)),
            Polygon pg => TransformPolygon(pg, transform, factory),
            MultiPoint mp => factory.CreateMultiPoint(
                Enumerable.Range(0, mp.NumGeometries)
                    .Select(i => (Point)TransformGeometry(mp.GetGeometryN(i), transform))
                    .ToArray()),
            MultiLineString mls => factory.CreateMultiLineString(
                Enumerable.Range(0, mls.NumGeometries)
                    .Select(i => (LineString)TransformGeometry(mls.GetGeometryN(i), transform))
                    .ToArray()),
            MultiPolygon mpg => factory.CreateMultiPolygon(
                Enumerable.Range(0, mpg.NumGeometries)
                    .Select(i => (Polygon)TransformGeometry(mpg.GetGeometryN(i), transform))
                    .ToArray()),
            GeometryCollection gc => factory.CreateGeometryCollection(
                Enumerable.Range(0, gc.NumGeometries)
                    .Select(i => TransformGeometry(gc.GetGeometryN(i), transform))
                    .ToArray()),
            _ => throw new NotSupportedException($"不支持的几何类型: {geometry.GeometryType}")
        };
    }

    private static Polygon TransformPolygon(Polygon polygon, IMathTransform transform, GeometryFactory factory)
    {
        var shell = factory.CreateLinearRing(TransformCoordinates(polygon.ExteriorRing.Coordinates, transform));
        var holes = new LinearRing[polygon.NumInteriorRings];
        for (int i = 0; i < polygon.NumInteriorRings; i++)
        {
            holes[i] = factory.CreateLinearRing(
                TransformCoordinates(polygon.GetInteriorRingN(i).Coordinates, transform));
        }
        return factory.CreatePolygon(shell, holes);
    }

    private static Coordinate[] TransformCoordinates(Coordinate[] coords, IMathTransform transform)
    {
        return coords.Select(c => TransformCoordinate(c, transform)).ToArray();
    }

    private static Coordinate TransformCoordinate(Coordinate coord, IMathTransform transform)
    {
        var result = transform.Transform(new double[] { coord.X, coord.Y });
        return new Coordinate(result[0], result[1]);
    }
}
```

- [ ] **Step 2: Verify build**

```bash
dotnet build src/Wif.Gis/Wif.Gis.csproj
```

---

### Task 8: Create MeasureTool

**Files:**
- Create: `src/Wif.Gis/Editing/MeasureTool.cs`

This task depends on the `IMapTool` interface being created by the gis-editing agent. The tool must implement that interface. If `IMapTool` is not yet available, create a stub reference and update once the editing agent completes.

- [ ] **Step 1: Create the MeasureTool**

```csharp
using Avalonia.Input;
using NetTopologySuite.Geometries;
using Wif.Domain.Entities;
using Wif.Gis.Analysis;
using Wif.Gis.Services;

namespace Wif.Gis.Editing;

/// <summary>测量工具模式。</summary>
public enum MeasureToolMode
{
    /// <summary>测量距离（折线）。</summary>
    Distance,
    /// <summary>测量面积（多边形）。</summary>
    Area
}

/// <summary>
/// 交互式测量工具。
/// 点击添加测量点，移动显示实时距离/面积，双击结束测量。
/// 实现 IMapTool 接口（由 gis-editing 模块定义）。
/// </summary>
public sealed class MeasureTool : IMapTool
{
    private readonly MeasurementService _measurementService;
    private readonly GeometryService _geometryService;
    private readonly List<Coordinate> _points = [];
    private Coordinate? _currentMousePosition;
    private bool _isActive;

    public MeasureTool(MeasurementService measurementService, GeometryService geometryService)
    {
        _measurementService = measurementService;
        _geometryService = geometryService;
    }

    // ─── IMapTool ────────────────────────────────────

    public string Name => "Measure";
    public string DisplayName => "测量工具";

    /// <summary>当前测量模式。</summary>
    public MeasureToolMode Mode { get; set; } = MeasureToolMode.Distance;

    /// <summary>测量使用的计算模式（平面/大地）。</summary>
    public MeasureMode MeasureMode { get; set; } = MeasureMode.Geodesic;

    /// <summary>当前收集的测量点。</summary>
    public IReadOnlyList<Coordinate> Points => _points;

    /// <summary>当测量更新时触发（实时数据、临时图层）。</summary>
    public event Action<MeasureToolUpdate>? MeasurementUpdated;

    /// <summary>当测量完成时触发。</summary>
    public event Action<MeasureToolResult>? MeasurementCompleted;

    public ToolResult OnPointerPressed(MapToolEventArgs e)
    {
        if (!_isActive) return ToolResult.Unhandled;

        var coord = new Coordinate(e.MapX, e.MapY);

        // Double-click finishes
        if (e.ClickCount >= 2)
        {
            FinishMeasurement();
            return ToolResult.Handled;
        }

        _points.Add(coord);
        UpdateLiveMeasurement();

        return ToolResult.Handled;
    }

    public ToolResult OnPointerMoved(MapToolEventArgs e)
    {
        if (!_isActive || _points.Count == 0) return ToolResult.Unhandled;

        _currentMousePosition = new Coordinate(e.MapX, e.MapY);
        UpdateLiveMeasurement();

        return ToolResult.Handled;
    }

    public ToolResult OnPointerReleased(MapToolEventArgs e)
    {
        return ToolResult.Unhandled;
    }

    public ToolResult OnKeyDown(KeyEventArgs e)
    {
        if (!_isActive) return ToolResult.Unhandled;

        if (e.Key == Key.Escape)
        {
            CancelMeasurement();
            return ToolResult.Handled;
        }

        if (e.Key == Key.Enter && _points.Count >= 2)
        {
            FinishMeasurement();
            return ToolResult.Handled;
        }

        // Backspace removes last point
        if (e.Key == Key.Back && _points.Count > 0)
        {
            _points.RemoveAt(_points.Count - 1);
            UpdateLiveMeasurement();
            return ToolResult.Handled;
        }

        return ToolResult.Unhandled;
    }

    public void Activate()
    {
        _isActive = true;
        _points.Clear();
        _currentMousePosition = null;
    }

    public void Deactivate()
    {
        _isActive = false;
        _points.Clear();
        _currentMousePosition = null;
    }

    // ─── Internal Logic ──────────────────────────────

    private void UpdateLiveMeasurement()
    {
        var allPoints = new List<Coordinate>(_points);
        if (_currentMousePosition is not null)
            allPoints.Add(_currentMousePosition);

        MeasurementResult? result = null;
        GisLayer? previewLayer = null;

        if (Mode == MeasureToolMode.Distance && allPoints.Count >= 2)
        {
            result = _measurementService.MeasurePathLength(allPoints, MeasureMode);
            previewLayer = CreateLinePreview(allPoints);
        }
        else if (Mode == MeasureToolMode.Area && allPoints.Count >= 3)
        {
            result = _measurementService.MeasureRingArea(allPoints, MeasureMode);
            previewLayer = CreatePolygonPreview(allPoints);
        }

        MeasurementUpdated?.Invoke(new MeasureToolUpdate
        {
            PointCount = _points.Count,
            LiveResult = result,
            PreviewLayer = previewLayer,
            StatusText = result?.DisplayText ?? $"已添加 {_points.Count} 个点，继续点击添加"
        });
    }

    private void FinishMeasurement()
    {
        MeasurementResult? finalResult = null;
        GisLayer? resultLayer = null;

        if (Mode == MeasureToolMode.Distance && _points.Count >= 2)
        {
            finalResult = _measurementService.MeasurePathLength(_points, MeasureMode);
            resultLayer = CreateLinePreview(_points);
            resultLayer.Name = $"距离测量 ({finalResult.DisplayText})";
        }
        else if (Mode == MeasureToolMode.Area && _points.Count >= 3)
        {
            finalResult = _measurementService.MeasureRingArea(_points, MeasureMode);
            resultLayer = CreatePolygonPreview(_points);
            resultLayer.Name = $"面积测量 ({finalResult.DisplayText})";
        }

        MeasurementCompleted?.Invoke(new MeasureToolResult
        {
            Result = finalResult,
            ResultLayer = resultLayer,
            Points = _points.ToList()
        });

        _points.Clear();
        _currentMousePosition = null;
    }

    private void CancelMeasurement()
    {
        _points.Clear();
        _currentMousePosition = null;

        MeasurementUpdated?.Invoke(new MeasureToolUpdate
        {
            PointCount = 0,
            LiveResult = null,
            PreviewLayer = null,
            StatusText = "测量已取消"
        });
    }

    private GisLayer CreateLinePreview(IReadOnlyList<Coordinate> coords)
    {
        var now = DateTimeOffset.UtcNow;
        var factory = new GeometryFactory();
        var line = factory.CreateLineString(coords.ToArray());

        var layer = new GisLayer
        {
            Id = Ulid.NewUlid(),
            Name = "测量预览",
            StrokeColor = "#FF5722",
            StrokeWidth = 2.5,
            FillColor = "#FF572240",
            Opacity = 0.8,
            CreatedAt = now,
            UpdatedAt = now
        };

        layer.Features.Add(new GisFeature
        {
            Id = Ulid.NewUlid(),
            LayerId = layer.Id,
            GeometryWkt = _geometryService.ToWkt(line),
            GeometryType = "LineString",
            Name = "测量路径",
            CreatedAt = now,
            UpdatedAt = now
        });

        return layer;
    }

    private GisLayer CreatePolygonPreview(IReadOnlyList<Coordinate> coords)
    {
        var now = DateTimeOffset.UtcNow;
        var factory = new GeometryFactory();
        var ring = coords.ToList();
        if (ring[0].X != ring[^1].X || ring[0].Y != ring[^1].Y)
            ring.Add(new Coordinate(ring[0].X, ring[0].Y));

        var polygon = factory.CreatePolygon(ring.ToArray());

        var layer = new GisLayer
        {
            Id = Ulid.NewUlid(),
            Name = "测量预览",
            StrokeColor = "#FF5722",
            StrokeWidth = 2.5,
            FillColor = "#FF572240",
            Opacity = 0.6,
            CreatedAt = now,
            UpdatedAt = now
        };

        layer.Features.Add(new GisFeature
        {
            Id = Ulid.NewUlid(),
            LayerId = layer.Id,
            GeometryWkt = _geometryService.ToWkt(polygon),
            GeometryType = "Polygon",
            Name = "测量区域",
            CreatedAt = now,
            UpdatedAt = now
        });

        return layer;
    }
}

// ─── Event Data Types ─────────────────────────────

/// <summary>实时测量更新数据。</summary>
public sealed record MeasureToolUpdate
{
    public int PointCount { get; init; }
    public MeasurementResult? LiveResult { get; init; }
    public GisLayer? PreviewLayer { get; init; }
    public string StatusText { get; init; } = string.Empty;
}

/// <summary>测量完成结果数据。</summary>
public sealed record MeasureToolResult
{
    public MeasurementResult? Result { get; init; }
    public GisLayer? ResultLayer { get; init; }
    public IReadOnlyList<Coordinate> Points { get; init; } = [];
}
```

**Note:** This file references `IMapTool`, `MapToolEventArgs`, and `ToolResult` from the gis-editing agent's work. These types must be defined in `src/Wif.Gis/Editing/` before this file compiles. If the gis-editing agent hasn't created them yet, the build will fail on this file — that's expected and will resolve once gis-editing completes.

- [ ] **Step 2: Verify build (may need gis-editing types first)**

```bash
dotnet build src/Wif.Gis/Wif.Gis.csproj
```

---

### Task 9: Create AnalysisDialog AXAML

**Files:**
- Create: `src/Wif.App/Views/AnalysisDialog.axaml`
- Create: `src/Wif.App/Views/AnalysisDialog.axaml.cs`

- [ ] **Step 1: Create the AXAML dialog**

```xml
<Window xmlns="https://github.com/avaloniaui"
        xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml"
        xmlns:vm="clr-namespace:Wif.App.ViewModels"
        x:Class="Wif.App.Views.AnalysisDialog"
        x:CompileBindings="False"
        Title="空间分析"
        Width="480" Height="520"
        WindowStartupLocation="CenterOwner"
        CanResize="False"
        SystemDecorations="Full"
        ExtendClientAreaToDecorationsHint="False">

  <Window.Styles>
    <Style Selector="Window">
      <Setter Property="Background" Value="{DynamicResource SystemControlBackgroundAltHighBrush}" />
    </Style>
    <Style Selector="TextBlock.section-title">
      <Setter Property="FontSize" Value="13" />
      <Setter Property="FontWeight" Value="SemiBold" />
      <Setter Property="Margin" Value="0,12,0,4" />
    </Style>
    <Style Selector="Button.primary">
      <Setter Property="Background" Value="#1A73E8" />
      <Setter Property="Foreground" Value="White" />
      <Setter Property="CornerRadius" Value="6" />
      <Setter Property="Padding" Value="16,8" />
      <Setter Property="HorizontalAlignment" Value="Stretch" />
    </Style>
  </Window.Styles>

  <DockPanel Margin="24,16">

    <!-- Header -->
    <DockPanel DockPanel.Dock="Top" Margin="0,0,0,12">
      <Button DockPanel.Dock="Right" Content="&#xE711;" FontFamily="Segoe MDL2 Assets" FontSize="14"
              Background="Transparent" Click="OnClose" VerticalAlignment="Center" Padding="8,4" />
      <TextBlock Text="空间分析" FontSize="20" FontWeight="Bold" VerticalAlignment="Center" />
    </DockPanel>

    <!-- Error/Status -->
    <TextBlock DockPanel.Dock="Bottom"
               Text="{Binding ErrorMessage}"
               Foreground="#D32F2F"
               IsVisible="{Binding ErrorMessage, Converter={x:Static StringConverters.IsNotNullOrEmpty}}"
               TextWrapping="Wrap"
               Margin="0,8,0,0" />

    <!-- Execute Button -->
    <Button DockPanel.Dock="Bottom"
            Classes="primary"
            Content="执行分析"
            Command="{Binding ExecuteCommand}"
            IsEnabled="{Binding !IsRunning}"
            Margin="0,12,0,0" />

    <!-- Result summary -->
    <Border DockPanel.Dock="Bottom"
            Background="{DynamicResource SystemControlBackgroundBaseLowBrush}"
            CornerRadius="6" Padding="12" Margin="0,8,0,0"
            IsVisible="{Binding ResultSummary, Converter={x:Static StringConverters.IsNotNullOrEmpty}}">
      <TextBlock Text="{Binding ResultSummary}" TextWrapping="Wrap" />
    </Border>

    <!-- Scrollable content -->
    <ScrollViewer>
      <StackPanel Spacing="4">

        <!-- Analysis Type -->
        <TextBlock Classes="section-title" Text="分析类型" />
        <ComboBox ItemsSource="{Binding AnalysisTypes}"
                  SelectedItem="{Binding SelectedAnalysisType}"
                  HorizontalAlignment="Stretch" />

        <!-- Input Layer A -->
        <TextBlock Classes="section-title" Text="输入图层" />
        <ComboBox ItemsSource="{Binding AvailableLayers}"
                  SelectedItem="{Binding InputLayerA}"
                  DisplayMemberBinding="{Binding Name}"
                  HorizontalAlignment="Stretch" />

        <!-- Input Layer B (for intersection/difference) -->
        <TextBlock Classes="section-title" Text="叠加图层"
                   IsVisible="{Binding NeedsSecondLayer}" />
        <ComboBox ItemsSource="{Binding AvailableLayers}"
                  SelectedItem="{Binding InputLayerB}"
                  DisplayMemberBinding="{Binding Name}"
                  HorizontalAlignment="Stretch"
                  IsVisible="{Binding NeedsSecondLayer}" />

        <!-- Buffer Distance (for buffer analysis) -->
        <TextBlock Classes="section-title" Text="缓冲距离"
                   IsVisible="{Binding IsBufferAnalysis}" />
        <Grid ColumnDefinitions="*,100" IsVisible="{Binding IsBufferAnalysis}">
          <TextBox Grid.Column="0" Text="{Binding BufferDistance}" Watermark="输入距离值" />
          <ComboBox Grid.Column="1" ItemsSource="{Binding DistanceUnits}"
                    SelectedItem="{Binding SelectedDistanceUnit}" Margin="8,0,0,0" />
        </Grid>

        <!-- Merge Results (for buffer) -->
        <CheckBox Content="合并所有缓冲区为一个要素"
                  IsChecked="{Binding MergeResults}"
                  IsVisible="{Binding IsBufferAnalysis}"
                  Margin="0,4,0,0" />

        <!-- Result Layer Name -->
        <TextBlock Classes="section-title" Text="输出图层名称" />
        <TextBox Text="{Binding ResultLayerName}" Watermark="输入结果图层名称" />

      </StackPanel>
    </ScrollViewer>

  </DockPanel>
</Window>
```

- [ ] **Step 2: Create the code-behind**

```csharp
using Avalonia.Controls;
using Avalonia.Interactivity;

namespace Wif.App.Views;

public partial class AnalysisDialog : Window
{
    public AnalysisDialog()
    {
        InitializeComponent();
    }

    private void OnClose(object? sender, RoutedEventArgs e)
    {
        Close();
    }
}
```

- [ ] **Step 3: Verify build**

```bash
dotnet build src/Wif.App/Wif.App.csproj
```

---

### Task 10: Create AnalysisDialogViewModel

**Files:**
- Create: `src/Wif.App/ViewModels/AnalysisDialogViewModel.cs`

- [ ] **Step 1: Create the ViewModel with validation and analysis execution**

```csharp
using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Wif.Domain.Entities;
using Wif.Domain.Repositories;
using Wif.Gis.Analysis;

namespace Wif.App.ViewModels;

/// <summary>分析类型枚举。</summary>
public enum AnalysisType
{
    缓冲区分析,
    交集分析,
    合并分析,
    差异分析
}

/// <summary>距离单位。</summary>
public enum DistanceUnit
{
    度,
    米,
    公里
}

public partial class AnalysisDialogViewModel : ObservableObject
{
    private readonly SpatialAnalysisService _analysisService;
    private readonly IGisLayerRepository _repository;

    public AnalysisDialogViewModel(
        SpatialAnalysisService analysisService,
        IGisLayerRepository repository,
        IReadOnlyList<GisLayer> availableLayers)
    {
        _analysisService = analysisService;
        _repository = repository;

        foreach (var layer in availableLayers)
            AvailableLayers.Add(layer);

        InputLayerA = AvailableLayers.FirstOrDefault();
    }

    // ─── Collections ──────────────────────────────

    public ObservableCollection<GisLayer> AvailableLayers { get; } = [];

    public IReadOnlyList<AnalysisType> AnalysisTypes { get; } =
        Enum.GetValues<AnalysisType>();

    public IReadOnlyList<DistanceUnit> DistanceUnits { get; } =
        Enum.GetValues<DistanceUnit>();

    // ─── Properties ───────────────────────────────

    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(NeedsSecondLayer))]
    [NotifyPropertyChangedFor(nameof(IsBufferAnalysis))]
    private AnalysisType _selectedAnalysisType = AnalysisType.缓冲区分析;

    [ObservableProperty]
    private GisLayer? _inputLayerA;

    [ObservableProperty]
    private GisLayer? _inputLayerB;

    [ObservableProperty]
    private string _bufferDistance = "0.01";

    [ObservableProperty]
    private DistanceUnit _selectedDistanceUnit = DistanceUnit.度;

    [ObservableProperty]
    private bool _mergeResults;

    [ObservableProperty]
    private string _resultLayerName = "分析结果";

    [ObservableProperty]
    private string? _errorMessage;

    [ObservableProperty]
    private string? _resultSummary;

    [ObservableProperty]
    private bool _isRunning;

    /// <summary>分析完成后产生的结果图层（供外部读取）。</summary>
    public GisLayer? ResultLayer { get; private set; }

    // ─── Computed ─────────────────────────────────

    public bool NeedsSecondLayer =>
        SelectedAnalysisType is AnalysisType.交集分析 or AnalysisType.差异分析;

    public bool IsBufferAnalysis =>
        SelectedAnalysisType is AnalysisType.缓冲区分析;

    // ─── Commands ─────────────────────────────────

    [RelayCommand]
    private async Task ExecuteAsync()
    {
        ErrorMessage = null;
        ResultSummary = null;
        ResultLayer = null;

        // Validate
        if (InputLayerA is null)
        {
            ErrorMessage = "请选择输入图层";
            return;
        }

        if (NeedsSecondLayer && InputLayerB is null)
        {
            ErrorMessage = "请选择叠加图层";
            return;
        }

        if (string.IsNullOrWhiteSpace(ResultLayerName))
        {
            ErrorMessage = "请输入输出图层名称";
            return;
        }

        if (IsBufferAnalysis && !double.TryParse(BufferDistance, out var dist))
        {
            ErrorMessage = "缓冲距离必须是有效的数值";
            return;
        }

        IsRunning = true;

        try
        {
            // Load full layer with features
            var layerA = await _repository.GetLayerByIdAsync(InputLayerA.Id);
            if (layerA is null || layerA.Features.Count == 0)
            {
                ErrorMessage = "输入图层为空或不存在";
                return;
            }

            AnalysisResult result;

            switch (SelectedAnalysisType)
            {
                case AnalysisType.缓冲区分析:
                {
                    var distance = ConvertDistanceToDegrees(double.Parse(BufferDistance), SelectedDistanceUnit);
                    result = _analysisService.BufferAnalysis(layerA, distance, ResultLayerName, MergeResults);
                    break;
                }
                case AnalysisType.交集分析:
                {
                    var layerB = await _repository.GetLayerByIdAsync(InputLayerB!.Id);
                    if (layerB is null || layerB.Features.Count == 0)
                    {
                        ErrorMessage = "叠加图层为空或不存在";
                        return;
                    }
                    result = _analysisService.IntersectionAnalysis(layerA, layerB, ResultLayerName);
                    break;
                }
                case AnalysisType.合并分析:
                {
                    result = _analysisService.UnionAnalysis(layerA, ResultLayerName);
                    break;
                }
                case AnalysisType.差异分析:
                {
                    var layerB = await _repository.GetLayerByIdAsync(InputLayerB!.Id);
                    if (layerB is null || layerB.Features.Count == 0)
                    {
                        ErrorMessage = "叠加图层为空或不存在";
                        return;
                    }
                    result = _analysisService.DifferenceAnalysis(layerA, layerB, ResultLayerName);
                    break;
                }
                default:
                    ErrorMessage = "未知的分析类型";
                    return;
            }

            // Save result layer to repository
            await _repository.CreateLayerAsync(result.OutputLayer);

            ResultLayer = result.OutputLayer;
            ResultSummary = result.Summary;
        }
        catch (Exception ex)
        {
            ErrorMessage = $"分析执行失败: {ex.Message}";
            System.Diagnostics.Debug.WriteLine($"[ERROR] Analysis failed: {ex}");
        }
        finally
        {
            IsRunning = false;
        }
    }

    // ─── Unit Conversion ──────────────────────────

    /// <summary>
    /// 将用户输入的距离转换为度（近似转换，用于 NTS 缓冲操作）。
    /// 1度 ≈ 111,320 米（赤道处）。
    /// </summary>
    private static double ConvertDistanceToDegrees(double value, DistanceUnit unit)
    {
        const double metersPerDegree = 111_320.0;
        return unit switch
        {
            DistanceUnit.度 => value,
            DistanceUnit.米 => value / metersPerDegree,
            DistanceUnit.公里 => value * 1000 / metersPerDegree,
            _ => value
        };
    }
}
```

- [ ] **Step 2: Verify build**

```bash
dotnet build src/Wif.App/Wif.App.csproj
```

---

## Build Verification

After completing all tasks:

```bash
dotnet build Wif.sln
```

All new files should compile without errors, except for `MeasureTool.cs` which depends on `IMapTool`, `MapToolEventArgs`, and `ToolResult` types from the gis-editing agent. Those will resolve once gis-editing completes.

## Dependency Notes

- **Task 1** (ProjNET package) must be done before Task 6 and Task 7.
- **Task 2** (CrsCode property) should be done early; Tasks 4, 5, 6, 7 reference it.
- **Task 3** (AnalysisResult) must be done before Task 4.
- **Task 4** (SpatialAnalysisService) must be done before Task 10.
- **Task 5** (MeasurementService) must be done before Task 8.
- **Task 6** (CrsManager) must be done before Task 7.
- **Task 8** (MeasureTool) depends on gis-editing agent's IMapTool interface.
- **Tasks 9-10** (AnalysisDialog) depend on Task 4 but can be done independently of Tasks 6-8.

Recommended execution order: 1 → 2 → 3 → 4, 5, 6 (parallel) → 7 → 8 → 9 → 10.

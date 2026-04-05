# GIS GDAL Integration — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace ProjNET and NTS.IO.ShapeFile with GDAL for broader format support, raster capabilities, and stronger CRS transformations.

**Architecture:** GDAL serves as data IO and projection backbone. Existing Mapsui + NTS architecture unchanged. GDAL reads files → WKT → NTS geometry pipeline. Three parallel agent streams (gdal-io, projection, raster), then integration assembly.

**Tech Stack:** MaxRev.Gdal.Core 3.12.2, Mapsui.Avalonia 5.0.2, NetTopologySuite 2.6.0, xUnit

**Spec:** `docs/superpowers/specs/2026-04-03-gis-gdal-integration-design.md`

---

## File Map

| Action | File | Owner |
|--------|------|-------|
| Modify | `src/Wif.Gis/Wif.Gis.csproj` | prerequisite |
| Create | `src/Wif.Gis/Services/GdalDataService.cs` | gdal-io |
| Rewrite | `src/Wif.Gis/Services/GisFileService.cs` | gdal-io |
| Create | `tests/Wif.Gis.Tests/Services/GdalDataServiceTests.cs` | gdal-io |
| Create | `tests/Wif.Gis.Tests/Services/GisFileServiceTests.cs` | gdal-io |
| Rewrite | `src/Wif.Gis/Projection/CoordinateTransformer.cs` | projection |
| Create | `src/Wif.Gis/Projection/ChinaCrsHelper.cs` | projection |
| Delete | `src/Wif.Gis/Projection/CrsManager.cs` | projection |
| Create | `tests/Wif.Gis.Tests/Projection/CoordinateTransformerTests.cs` | projection |
| Create | `tests/Wif.Gis.Tests/Projection/ChinaCrsHelperTests.cs` | projection |
| Create | `src/Wif.Gis/Services/RasterLayerInfo.cs` | raster |
| Create | `src/Wif.Gis/Services/RasterLayerService.cs` | raster |
| Create | `tests/Wif.Gis.Tests/Services/RasterLayerServiceTests.cs` | raster |
| Modify | `src/Wif.Domain/Entities/GisLayer.cs` | integration |
| Modify | `src/Wif.Domain/Enums/GisLayerSourceType.cs` | integration |
| Modify | `src/Wif.Data/Configurations/GisLayerConfiguration.cs` | integration |
| Modify | `src/Wif.Data/Migrations/GisMigration.cs` | integration |
| Modify | `src/Wif.Core/Services/DependencyInjection.cs` | integration |
| Modify | `src/Wif.Core/Modules/GisModule.cs` | integration |
| Modify | `src/Wif.App/ViewModels/GisViewModel.cs` | integration |

---

## Task 0: NuGet Dependency Changes (prerequisite)

**Files:**
- Modify: `src/Wif.Gis/Wif.Gis.csproj`
- Create: `tests/Wif.Gis.Tests/Wif.Gis.Tests.csproj`

This task runs before any agent starts. It modifies the shared project file.

- [ ] **Step 1: Update Wif.Gis.csproj — remove old packages, add GDAL**

Replace the `<ItemGroup>` containing package references in `src/Wif.Gis/Wif.Gis.csproj`:

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
    <PackageReference Include="NetTopologySuite.Features" Version="2.2.0" />
    <PackageReference Include="MaxRev.Gdal.Core" Version="3.12.2" />
    <PackageReference Include="MaxRev.Gdal.WindowsRuntime.Minimal" Version="3.12.2" />
  </ItemGroup>
</Project>
```

Removed: `NetTopologySuite.IO.ShapeFile 2.1.0`, `ProjNET 2.0.0`.
Added: `MaxRev.Gdal.Core 3.12.2`, `MaxRev.Gdal.WindowsRuntime.Minimal 3.12.2`.

- [ ] **Step 2: Create test project if missing**

```xml
<!-- tests/Wif.Gis.Tests/Wif.Gis.Tests.csproj -->
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net9.0</TargetFramework>
    <ImplicitUsings>enable</ImplicitUsings>
    <Nullable>enable</Nullable>
    <IsPackable>false</IsPackable>
  </PropertyGroup>

  <ItemGroup>
    <PackageReference Include="Microsoft.NET.Test.Sdk" Version="17.12.0" />
    <PackageReference Include="xunit" Version="2.9.3" />
    <PackageReference Include="xunit.runner.visualstudio" Version="3.0.2" />
    <PackageReference Include="coverlet.collector" Version="6.0.4" />
  </ItemGroup>

  <ItemGroup>
    <ProjectReference Include="..\..\src\Wif.Gis\Wif.Gis.csproj" />
    <ProjectReference Include="..\..\src\Wif.Domain\Wif.Domain.csproj" />
  </ItemGroup>
</Project>
```

- [ ] **Step 3: Add test project to solution**

Run: `dotnet sln Wif.sln add tests/Wif.Gis.Tests/Wif.Gis.Tests.csproj`

- [ ] **Step 4: Verify build compiles (expect errors in CrsManager/ShapeFile references)**

Run: `dotnet build src/Wif.Gis/Wif.Gis.csproj 2>&1 | head -30`

Expected: Compilation errors in `GisFileService.cs` (ShapefileDataReader missing) and `CoordinateTransformer.cs` / `CrsManager.cs` (ProjNet missing). These will be fixed by Wave 1 agents.

- [ ] **Step 5: Commit**

```bash
git add src/Wif.Gis/Wif.Gis.csproj tests/Wif.Gis.Tests/Wif.Gis.Tests.csproj Wif.sln
git commit -m "chore: swap NuGet deps — remove ProjNET/ShapeFile, add GDAL"
```

---

## Wave 1 — Parallel Agents

### Task 1: GdalDataService (agent: gdal-io)

**Files:**
- Create: `src/Wif.Gis/Services/GdalDataService.cs`
- Test: `tests/Wif.Gis.Tests/Services/GdalDataServiceTests.cs`

- [ ] **Step 1: Write failing tests**

```csharp
// tests/Wif.Gis.Tests/Services/GdalDataServiceTests.cs
using Wif.Gis.Services;
using MaxRev.Gdal.Core;

namespace Wif.Gis.Tests.Services;

public class GdalDataServiceTests : IClassFixture<GdalFixture>
{
    private readonly GdalDataService _sut = new(new GeometryService());

    [Fact]
    public void DetectFormat_Shapefile_ReturnsEsriShapefile()
    {
        // Create a minimal .shp file for test
        var dir = Path.Combine(Path.GetTempPath(), $"gdaltest_{Guid.NewGuid():N}");
        Directory.CreateDirectory(dir);
        var shpPath = Path.Combine(dir, "test.shp");
        CreateMinimalShapefile(shpPath);

        var format = _sut.DetectFormat(shpPath);

        Assert.Equal("ESRI Shapefile", format);

        Directory.Delete(dir, true);
    }

    [Fact]
    public void ImportVector_GeoPackage_ReturnsLayer()
    {
        var dir = Path.Combine(Path.GetTempPath(), $"gdaltest_{Guid.NewGuid():N}");
        Directory.CreateDirectory(dir);
        var gpkgPath = Path.Combine(dir, "test.gpkg");
        CreateTestGeoPackage(gpkgPath);

        var layer = _sut.ImportVector(gpkgPath, "test-layer");

        Assert.Equal("test-layer", layer.Name);
        Assert.True(layer.Features.Count > 0);
        Assert.NotEmpty(layer.Features[0].GeometryWkt);

        Directory.Delete(dir, true);
    }

    [Fact]
    public void ExportVector_GeoPackage_CreatesFile()
    {
        var dir = Path.Combine(Path.GetTempPath(), $"gdaltest_{Guid.NewGuid():N}");
        Directory.CreateDirectory(dir);
        var gpkgPath = Path.Combine(dir, "output.gpkg");

        var layer = TestData.CreateSampleLayer();

        _sut.ExportVector(layer, gpkgPath, "GPKG");

        Assert.True(File.Exists(gpkgPath));

        // Reimport to verify
        var reimported = _sut.ImportVector(gpkgPath, "verify");
        Assert.Equal(layer.Features.Count, reimported.Features.Count);

        Directory.Delete(dir, true);
    }

    [Fact]
    public void GetSupportedVectorFormats_ContainsExpectedDrivers()
    {
        var formats = _sut.GetSupportedVectorFormats();

        Assert.Contains("ESRI Shapefile", formats);
        Assert.Contains("GPKG", formats);
        Assert.Contains("KML", formats);
        Assert.Contains("GeoJSON", formats);
    }

    // ─── Helpers ────────────────────────────────
    private static void CreateMinimalShapefile(string path)
    {
        var drv = OSGeo.OGR.Ogr.GetDriverByName("ESRI Shapefile");
        var ds = drv.CreateDataSource(path, null);
        var srs = new OSGeo.OSR.SpatialReference("");
        srs.ImportFromEPSG(4326);
        var lyr = ds.CreateLayer("test", srs, OSGeo.OGR.wkbGeometryType.wkbPoint, null);
        var feat = new OSGeo.OGR.Feature(lyr.GetLayerDefn());
        var geom = OSGeo.OGR.Geometry.CreateFromWkt("POINT (116.4 39.9)");
        feat.SetGeometry(geom);
        lyr.CreateFeature(feat);
        ds.Dispose();
    }

    private static void CreateTestGeoPackage(string path)
    {
        var drv = OSGeo.OGR.Ogr.GetDriverByName("GPKG");
        var ds = drv.CreateDataSource(path, null);
        var srs = new OSGeo.OSR.SpatialReference("");
        srs.ImportFromEPSG(4326);
        var lyr = ds.CreateLayer("features", srs, OSGeo.OGR.wkbGeometryType.wkbPolygon, null);
        var fieldDefn = new OSGeo.OGR.FieldDefn("name", OSGeo.OGR.FieldType.OFTString);
        lyr.CreateField(fieldDefn, 1);
        var feat = new OSGeo.OGR.Feature(lyr.GetLayerDefn());
        var geom = OSGeo.OGR.Geometry.CreateFromWkt(
            "POLYGON ((116.3 39.8, 116.5 39.8, 116.5 40.0, 116.3 40.0, 116.3 39.8))");
        feat.SetGeometry(geom);
        feat.SetField("name", "Beijing");
        lyr.CreateFeature(feat);
        ds.Dispose();
    }
}

// Shared GDAL initialization fixture
public class GdalFixture
{
    public GdalFixture() => GdalBase.ConfigureAll();
}

// Shared test data helper
public static class TestData
{
    public static GisLayer CreateSampleLayer()
    {
        var now = DateTimeOffset.UtcNow;
        var layer = new GisLayer
        {
            Id = Ulid.NewUlid(),
            Name = "sample",
            CreatedAt = now,
            UpdatedAt = now
        };
        layer.Features.Add(new GisFeature
        {
            Id = Ulid.NewUlid(),
            LayerId = layer.Id,
            GeometryWkt = "POINT (116.4 39.9)",
            GeometryType = "Point",
            Name = "Beijing",
            CreatedAt = now,
            UpdatedAt = now
        });
        layer.Features.Add(new GisFeature
        {
            Id = Ulid.NewUlid(),
            LayerId = layer.Id,
            GeometryWkt = "POLYGON ((121.4 31.1, 121.5 31.1, 121.5 31.3, 121.4 31.3, 121.4 31.1))",
            GeometryType = "Polygon",
            Name = "Shanghai",
            CreatedAt = now,
            UpdatedAt = now
        });
        return layer;
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `dotnet test tests/Wif.Gis.Tests --filter "FullyQualifiedName~GdalDataServiceTests" -v quiet`
Expected: FAIL — `GdalDataService` class does not exist yet.

- [ ] **Step 3: Implement GdalDataService**

```csharp
// src/Wif.Gis/Services/GdalDataService.cs
using System.Text.Json;
using MaxRev.Gdal.Core;
using OSGeo.OGR;
using OSGeo.OSR;
using Wif.Domain.Entities;

namespace Wif.Gis.Services;

/// <summary>
/// GDAL OGR 矢量数据读写服务。
/// 将 OGR 数据源转换为 GisLayer/GisFeature 域模型（WKT 桥接）。
/// </summary>
public sealed class GdalDataService
{
    private readonly GeometryService _geometry;

    public GdalDataService(GeometryService geometry)
    {
        _geometry = geometry;
    }

    /// <summary>
    /// 自动检测文件的矢量格式驱动名。
    /// </summary>
    public string DetectFormat(string filePath)
    {
        using var ds = Ogr.Open(filePath, 0);
        if (ds is null)
            throw new InvalidOperationException($"无法打开文件: {filePath}");
        return ds.GetDriver().GetName();
    }

    /// <summary>
    /// 导入矢量文件为 GisLayer。
    /// 支持 Shapefile、GeoPackage、KML、GML、CSV、DXF、MapInfo 等 OGR 支持的格式。
    /// </summary>
    public GisLayer ImportVector(string filePath, string layerName)
    {
        using var ds = Ogr.Open(filePath, 0);
        if (ds is null)
            throw new InvalidOperationException($"无法打开矢量数据源: {filePath}");

        var ogrLayer = ds.GetLayerByIndex(0);
        if (ogrLayer is null)
            throw new InvalidOperationException($"数据源中没有图层: {filePath}");

        var now = DateTimeOffset.UtcNow;
        var gisLayer = new GisLayer
        {
            Id = Ulid.NewUlid(),
            Name = layerName,
            CreatedAt = now,
            UpdatedAt = now
        };

        // Detect CRS
        var srs = ogrLayer.GetSpatialRef();
        if (srs is not null)
        {
            srs.AutoIdentifyEPSG();
            var code = srs.GetAuthorityCode(null);
            if (!string.IsNullOrEmpty(code))
                gisLayer.CrsCode = $"EPSG:{code}";
        }

        // Read field names for attribute extraction
        var defn = ogrLayer.GetLayerDefn();
        var fieldCount = defn.GetFieldCount();
        var fieldNames = new string[fieldCount];
        for (int i = 0; i < fieldCount; i++)
            fieldNames[i] = defn.GetFieldDefn(i).GetName();

        // Read features
        ogrLayer.ResetReading();
        Feature ogrFeat;
        while ((ogrFeat = ogrLayer.GetNextFeature()) is not null)
        {
            var geom = ogrFeat.GetGeometryRef();
            if (geom is null)
            {
                ogrFeat.Dispose();
                continue;
            }

            string wkt;
            geom.ExportToWkt(out wkt);

            // Read attributes
            var props = new Dictionary<string, object?>();
            for (int i = 0; i < fieldCount; i++)
            {
                if (ogrFeat.IsFieldSetAndNotNull(i))
                {
                    props[fieldNames[i]] = ogrFeat.GetFieldAsString(i);
                }
            }

            var name = props.TryGetValue("name", out var n) ? n?.ToString()
                     : props.TryGetValue("NAME", out var n2) ? n2?.ToString()
                     : null;

            gisLayer.Features.Add(new GisFeature
            {
                Id = Ulid.NewUlid(),
                LayerId = gisLayer.Id,
                GeometryWkt = wkt,
                GeometryType = geom.GetGeometryName(),
                Name = name,
                PropertiesJson = props.Count > 0 ? JsonSerializer.Serialize(props) : null,
                CreatedAt = now,
                UpdatedAt = now
            });

            ogrFeat.Dispose();
        }

        return gisLayer;
    }

    /// <summary>
    /// 导出 GisLayer 为矢量文件。
    /// </summary>
    /// <param name="layer">图层数据。</param>
    /// <param name="filePath">输出文件路径。</param>
    /// <param name="driverName">OGR驱动名，如 "GPKG"、"ESRI Shapefile"、"KML"。</param>
    public void ExportVector(GisLayer layer, string filePath, string driverName)
    {
        var drv = Ogr.GetDriverByName(driverName);
        if (drv is null)
            throw new ArgumentException($"不支持的导出格式: {driverName}", nameof(driverName));

        // Delete existing file if present
        if (File.Exists(filePath))
            File.Delete(filePath);

        using var ds = drv.CreateDataSource(filePath, null);
        if (ds is null)
            throw new InvalidOperationException($"无法创建数据源: {filePath}");

        var srs = new SpatialReference("");
        var epsgCode = ParseEpsgCode(layer.CrsCode);
        if (epsgCode > 0)
            srs.ImportFromEPSG(epsgCode);
        else
            srs.ImportFromEPSG(4326);

        // Detect geometry type from first feature
        var wkbType = wkbGeometryType.wkbUnknown;
        if (layer.Features.Count > 0)
        {
            wkbType = layer.Features[0].GeometryType.ToUpperInvariant() switch
            {
                "POINT" => wkbGeometryType.wkbPoint,
                "LINESTRING" => wkbGeometryType.wkbLineString,
                "POLYGON" => wkbGeometryType.wkbPolygon,
                "MULTIPOINT" => wkbGeometryType.wkbMultiPoint,
                "MULTILINESTRING" => wkbGeometryType.wkbMultiLineString,
                "MULTIPOLYGON" => wkbGeometryType.wkbMultiPolygon,
                _ => wkbGeometryType.wkbUnknown
            };
        }

        var ogrLayer = ds.CreateLayer(layer.Name, srs, wkbType, null);

        // Create "name" field
        var nameField = new FieldDefn("name", FieldType.OFTString);
        nameField.SetWidth(200);
        ogrLayer.CreateField(nameField, 1);

        // Write features
        foreach (var gisFeature in layer.Features)
        {
            var feat = new Feature(ogrLayer.GetLayerDefn());

            var geom = Geometry.CreateFromWkt(gisFeature.GeometryWkt);
            feat.SetGeometry(geom);

            if (gisFeature.Name is not null)
                feat.SetField("name", gisFeature.Name);

            ogrLayer.CreateFeature(feat);
            feat.Dispose();
        }
    }

    /// <summary>
    /// 列出 GDAL 中所有可用的矢量格式驱动名。
    /// </summary>
    public string[] GetSupportedVectorFormats()
    {
        var formats = new List<string>();
        var count = Ogr.GetDriverCount();
        for (int i = 0; i < count; i++)
        {
            var drv = Ogr.GetDriver(i);
            formats.Add(drv.GetName());
        }
        return formats.ToArray();
    }

    private static int ParseEpsgCode(string? crsCode)
    {
        if (string.IsNullOrEmpty(crsCode)) return 0;
        var parts = crsCode.Split(':');
        if (parts.Length == 2 && int.TryParse(parts[1], out var code))
            return code;
        return 0;
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `dotnet test tests/Wif.Gis.Tests --filter "FullyQualifiedName~GdalDataServiceTests" -v quiet`
Expected: PASS (all 4 tests).

- [ ] **Step 5: Commit**

```bash
git add src/Wif.Gis/Services/GdalDataService.cs tests/Wif.Gis.Tests/Services/GdalDataServiceTests.cs
git commit -m "feat(gis): add GdalDataService for multi-format vector IO"
```

---

### Task 2: Rewrite GisFileService (agent: gdal-io)

**Files:**
- Rewrite: `src/Wif.Gis/Services/GisFileService.cs`
- Test: `tests/Wif.Gis.Tests/Services/GisFileServiceTests.cs`

- [ ] **Step 1: Write failing tests**

```csharp
// tests/Wif.Gis.Tests/Services/GisFileServiceTests.cs
using Wif.Gis.Services;

namespace Wif.Gis.Tests.Services;

public class GisFileServiceTests : IClassFixture<GdalFixture>
{
    private readonly GisFileService _sut;

    public GisFileServiceTests()
    {
        var geometry = new GeometryService();
        var gdal = new GdalDataService(geometry);
        _sut = new GisFileService(geometry, gdal);
    }

    [Fact]
    public void ImportGeoJson_ValidJson_ReturnsLayer()
    {
        var json = """
        {
          "type": "FeatureCollection",
          "features": [{
            "type": "Feature",
            "geometry": {"type": "Point", "coordinates": [116.4, 39.9]},
            "properties": {"name": "Beijing"}
          }]
        }
        """;

        var layer = _sut.ImportGeoJson(json, "test");

        Assert.Equal("test", layer.Name);
        Assert.Single(layer.Features);
        Assert.Equal("Beijing", layer.Features[0].Name);
    }

    [Fact]
    public void ExportGeoJson_RoundTrip()
    {
        var layer = TestData.CreateSampleLayer();
        var json = _sut.ExportGeoJson(layer);
        var reimported = _sut.ImportGeoJson(json, "reimport");

        Assert.Equal(layer.Features.Count, reimported.Features.Count);
    }

    [Fact]
    public void ImportFile_Shapefile_AutoDetects()
    {
        var dir = Path.Combine(Path.GetTempPath(), $"gisfiletest_{Guid.NewGuid():N}");
        Directory.CreateDirectory(dir);

        // Create shapefile via GDAL
        var drv = OSGeo.OGR.Ogr.GetDriverByName("ESRI Shapefile");
        var ds = drv.CreateDataSource(Path.Combine(dir, "test.shp"), null);
        var srs = new OSGeo.OSR.SpatialReference("");
        srs.ImportFromEPSG(4326);
        var lyr = ds.CreateLayer("test", srs, OSGeo.OGR.wkbGeometryType.wkbPoint, null);
        var feat = new OSGeo.OGR.Feature(lyr.GetLayerDefn());
        feat.SetGeometry(OSGeo.OGR.Geometry.CreateFromWkt("POINT (116.4 39.9)"));
        lyr.CreateFeature(feat);
        ds.Dispose();

        var layer = _sut.ImportFile(Path.Combine(dir, "test.shp"));

        Assert.NotNull(layer);
        Assert.True(layer.Features.Count > 0);

        Directory.Delete(dir, true);
    }

    [Fact]
    public void ImportFile_GeoJson_AutoDetects()
    {
        var dir = Path.Combine(Path.GetTempPath(), $"gisfiletest_{Guid.NewGuid():N}");
        Directory.CreateDirectory(dir);
        var path = Path.Combine(dir, "test.geojson");
        File.WriteAllText(path, """
        {
          "type": "FeatureCollection",
          "features": [{
            "type": "Feature",
            "geometry": {"type": "Point", "coordinates": [116.4, 39.9]},
            "properties": {"name": "Test"}
          }]
        }
        """);

        var layer = _sut.ImportFile(path);

        Assert.NotNull(layer);
        Assert.Single(layer.Features);

        Directory.Delete(dir, true);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `dotnet test tests/Wif.Gis.Tests --filter "FullyQualifiedName~GisFileServiceTests" -v quiet`
Expected: FAIL — constructor signature changed, `ImportFile` method missing.

- [ ] **Step 3: Rewrite GisFileService**

```csharp
// src/Wif.Gis/Services/GisFileService.cs
using System.Text.Json;
using NetTopologySuite.Features;
using NetTopologySuite.Geometries;
using NetTopologySuite.IO;
using Wif.Domain.Entities;

namespace Wif.Gis.Services;

/// <summary>
/// 文件导入导出服务。
/// GeoJSON 使用 NTS 原生读写；其他格式委托 GdalDataService。
/// </summary>
public sealed class GisFileService
{
    private readonly GeometryService _geometryService;
    private readonly GdalDataService _gdalData;

    public GisFileService(GeometryService geometryService, GdalDataService gdalData)
    {
        _geometryService = geometryService;
        _gdalData = gdalData;
    }

    // ─── Unified Import ───────────────────────────

    /// <summary>
    /// 自动检测文件格式并导入为图层。
    /// .geojson/.json 使用 NTS；其他格式使用 GDAL。
    /// </summary>
    public GisLayer ImportFile(string filePath)
    {
        var layerName = Path.GetFileNameWithoutExtension(filePath);
        var ext = Path.GetExtension(filePath).ToLowerInvariant();

        if (ext is ".geojson" or ".json")
        {
            var json = File.ReadAllText(filePath);
            return ImportGeoJson(json, layerName);
        }

        return _gdalData.ImportVector(filePath, layerName);
    }

    // ─── GeoJSON (NTS native) ─────────────────────

    public GisLayer ImportGeoJson(string json, string layerName)
    {
        var reader = new GeoJsonReader();
        var featureCollection = reader.Read<FeatureCollection>(json);

        var now = DateTimeOffset.UtcNow;
        var layer = new GisLayer
        {
            Id = Ulid.NewUlid(),
            Name = layerName,
            CreatedAt = now,
            UpdatedAt = now
        };

        foreach (var feature in featureCollection)
        {
            var wkt = _geometryService.ToWkt(feature.Geometry);
            var propsDict = new Dictionary<string, object?>();
            foreach (var attrName in feature.Attributes.GetNames())
            {
                propsDict[attrName] = feature.Attributes[attrName];
            }

            layer.Features.Add(new GisFeature
            {
                Id = Ulid.NewUlid(),
                LayerId = layer.Id,
                GeometryWkt = wkt,
                GeometryType = feature.Geometry.GeometryType,
                Name = propsDict.TryGetValue("name", out var name) ? name?.ToString() : null,
                PropertiesJson = JsonSerializer.Serialize(propsDict),
                CreatedAt = now,
                UpdatedAt = now
            });
        }

        return layer;
    }

    public string ExportGeoJson(GisLayer layer)
    {
        var features = new List<Feature>();

        foreach (var f in layer.Features)
        {
            var geometry = _geometryService.ParseWkt(f.GeometryWkt);
            var attributes = new AttributesTable();

            if (f.Name is not null)
                attributes.Add("name", f.Name);

            if (f.PropertiesJson is not null)
            {
                try
                {
                    var props = JsonSerializer.Deserialize<Dictionary<string, JsonElement>>(f.PropertiesJson);
                    if (props is not null)
                    {
                        foreach (var (key, value) in props)
                        {
                            if (!attributes.Exists(key))
                                attributes.Add(key, value.ToString());
                        }
                    }
                }
                catch { /* skip invalid JSON */ }
            }

            features.Add(new Feature(geometry, attributes));
        }

        var fc = new FeatureCollection();
        foreach (var feat in features)
            fc.Add(feat);

        var writer = new GeoJsonWriter();
        return writer.Write(fc);
    }

    // ─── GDAL-backed formats ──────────────────────

    public GisLayer ImportShapefile(string shpPath, string layerName)
        => _gdalData.ImportVector(shpPath, layerName);

    public void ExportShapefile(GisLayer layer, string outputPath)
        => _gdalData.ExportVector(layer, outputPath, "ESRI Shapefile");

    public GisLayer ImportGeoPackage(string gpkgPath, string layerName)
        => _gdalData.ImportVector(gpkgPath, layerName);

    public void ExportGeoPackage(GisLayer layer, string outputPath)
        => _gdalData.ExportVector(layer, outputPath, "GPKG");

    public GisLayer ImportKml(string kmlPath, string layerName)
        => _gdalData.ImportVector(kmlPath, layerName);

    public void ExportKml(GisLayer layer, string outputPath)
        => _gdalData.ExportVector(layer, outputPath, "KML");
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `dotnet test tests/Wif.Gis.Tests --filter "FullyQualifiedName~GisFileServiceTests" -v quiet`
Expected: PASS (all 4 tests).

- [ ] **Step 5: Commit**

```bash
git add src/Wif.Gis/Services/GisFileService.cs tests/Wif.Gis.Tests/Services/GisFileServiceTests.cs
git commit -m "feat(gis): rewrite GisFileService — delegate non-GeoJSON to GDAL"
```

---

### Task 3: Rewrite CoordinateTransformer (agent: projection)

**Files:**
- Rewrite: `src/Wif.Gis/Projection/CoordinateTransformer.cs`
- Delete: `src/Wif.Gis/Projection/CrsManager.cs`
- Test: `tests/Wif.Gis.Tests/Projection/CoordinateTransformerTests.cs`

- [ ] **Step 1: Write failing tests**

```csharp
// tests/Wif.Gis.Tests/Projection/CoordinateTransformerTests.cs
using Wif.Gis.Projection;
using Wif.Gis.Services;

namespace Wif.Gis.Tests.Projection;

public class CoordinateTransformerTests : IClassFixture<GdalFixture>
{
    private readonly CoordinateTransformer _sut = new();
    private readonly GeometryService _geometry = new();

    [Fact]
    public void Transform_4326_To_3857_ConvertsCorrectly()
    {
        // Beijing: 116.4E, 39.9N
        var wgs84 = _geometry.ParseWkt("POINT (116.4 39.9)");

        var mercator = _sut.Transform(wgs84, "EPSG:4326", "EPSG:3857");

        // Expected: ~12958148, ~4852834 (Mercator meters)
        Assert.InRange(mercator.Centroid.X, 12_950_000, 12_970_000);
        Assert.InRange(mercator.Centroid.Y, 4_840_000, 4_860_000);
    }

    [Fact]
    public void Transform_SameCrs_ReturnsCopy()
    {
        var point = _geometry.ParseWkt("POINT (116.4 39.9)");

        var result = _sut.Transform(point, "EPSG:4326", "EPSG:4326");

        Assert.Equal(point.Centroid.X, result.Centroid.X, 6);
        Assert.Equal(point.Centroid.Y, result.Centroid.Y, 6);
    }

    [Fact]
    public void TransformWkt_RoundTrip()
    {
        var wkt = "POINT (116.4 39.9)";

        var mercatorWkt = _sut.TransformWkt(wkt, "EPSG:4326", "EPSG:3857");
        var backWkt = _sut.TransformWkt(mercatorWkt, "EPSG:3857", "EPSG:4326");

        var backGeom = _geometry.ParseWkt(backWkt);
        Assert.InRange(backGeom.Centroid.X, 116.39, 116.41);
        Assert.InRange(backGeom.Centroid.Y, 39.89, 39.91);
    }

    [Fact]
    public void IsSupported_Epsg4326_ReturnsTrue()
    {
        Assert.True(_sut.IsSupported("EPSG:4326"));
    }

    [Fact]
    public void IsSupported_Epsg4490_Cgcs2000_ReturnsTrue()
    {
        Assert.True(_sut.IsSupported("EPSG:4490"));
    }

    [Fact]
    public void IsSupported_InvalidCode_ReturnsFalse()
    {
        Assert.False(_sut.IsSupported("EPSG:999999"));
    }

    [Fact]
    public void Transform_Polygon_PreservesStructure()
    {
        var poly = _geometry.ParseWkt(
            "POLYGON ((116.3 39.8, 116.5 39.8, 116.5 40.0, 116.3 40.0, 116.3 39.8))");

        var result = _sut.Transform(poly, "EPSG:4326", "EPSG:3857");

        Assert.Equal("Polygon", result.GeometryType);
        Assert.Equal(5, result.Coordinates.Length);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `dotnet test tests/Wif.Gis.Tests --filter "FullyQualifiedName~CoordinateTransformerTests" -v quiet`
Expected: FAIL — constructor signature changed.

- [ ] **Step 3: Rewrite CoordinateTransformer**

```csharp
// src/Wif.Gis/Projection/CoordinateTransformer.cs
using NetTopologySuite.Geometries;
using OSGeo.OSR;

namespace Wif.Gis.Projection;

/// <summary>
/// 坐标转换器 — 基于 GDAL/PROJ。
/// 支持全球 8000+ EPSG 坐标系，包括 CGCS2000 (EPSG:4490)。
/// </summary>
public sealed class CoordinateTransformer
{
    /// <summary>
    /// 将几何对象从源坐标系转换到目标坐标系。
    /// </summary>
    public Geometry Transform(Geometry geometry, string sourceCrsCode, string targetCrsCode)
    {
        if (string.Equals(sourceCrsCode, targetCrsCode, StringComparison.OrdinalIgnoreCase))
            return geometry.Copy();

        using var sourceSrs = CreateSrs(sourceCrsCode);
        using var targetSrs = CreateSrs(targetCrsCode);
        using var transform = new CoordinateTransformation(sourceSrs, targetSrs);

        return TransformGeometry(geometry, transform);
    }

    /// <summary>
    /// 将 WKT 几何字符串从源坐标系转换到目标坐标系。
    /// </summary>
    public string TransformWkt(string wkt, string sourceCrsCode, string targetCrsCode)
    {
        var reader = new NetTopologySuite.IO.WKTReader();
        var writer = new NetTopologySuite.IO.WKTWriter();
        var geom = reader.Read(wkt);
        var transformed = Transform(geom, sourceCrsCode, targetCrsCode);
        return writer.Write(transformed);
    }

    /// <summary>
    /// 检查指定 EPSG 代码是否受支持。
    /// </summary>
    public bool IsSupported(string crsCode)
    {
        var epsg = ParseEpsgCode(crsCode);
        if (epsg <= 0) return false;

        try
        {
            using var srs = new SpatialReference("");
            return srs.ImportFromEPSG(epsg) == 0;
        }
        catch
        {
            return false;
        }
    }

    // ─── Internal ─────────────────────────────────

    private static SpatialReference CreateSrs(string crsCode)
    {
        var epsg = ParseEpsgCode(crsCode);
        if (epsg <= 0)
            throw new ArgumentException($"无效的 EPSG 代码: {crsCode}", nameof(crsCode));

        var srs = new SpatialReference("");
        if (srs.ImportFromEPSG(epsg) != 0)
            throw new ArgumentException($"不支持的 EPSG 代码: {crsCode}", nameof(crsCode));

        // Ensure lon/lat axis order (not lat/lon) for geographic CRS
        srs.SetAxisMappingStrategy(AxisMappingStrategy.OAMS_TRADITIONAL_GIS_ORDER);
        return srs;
    }

    private static Geometry TransformGeometry(Geometry geometry, CoordinateTransformation transform)
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

    private static Polygon TransformPolygon(Polygon polygon, CoordinateTransformation transform, GeometryFactory factory)
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

    private static Coordinate[] TransformCoordinates(Coordinate[] coords, CoordinateTransformation transform)
    {
        var result = new Coordinate[coords.Length];
        for (int i = 0; i < coords.Length; i++)
            result[i] = TransformCoordinate(coords[i], transform);
        return result;
    }

    private static Coordinate TransformCoordinate(Coordinate coord, CoordinateTransformation transform)
    {
        var pts = new double[3];
        transform.TransformPoint(pts, coord.X, coord.Y, 0);
        return new Coordinate(pts[0], pts[1]);
    }

    private static int ParseEpsgCode(string? crsCode)
    {
        if (string.IsNullOrEmpty(crsCode)) return 0;
        var parts = crsCode.Split(':');
        if (parts.Length == 2 && int.TryParse(parts[1], out var code))
            return code;
        return 0;
    }
}
```

- [ ] **Step 4: Delete CrsManager.cs**

Run: `rm src/Wif.Gis/Projection/CrsManager.cs`

- [ ] **Step 5: Run tests to verify they pass**

Run: `dotnet test tests/Wif.Gis.Tests --filter "FullyQualifiedName~CoordinateTransformerTests" -v quiet`
Expected: PASS (all 7 tests).

- [ ] **Step 6: Commit**

```bash
git add src/Wif.Gis/Projection/CoordinateTransformer.cs tests/Wif.Gis.Tests/Projection/CoordinateTransformerTests.cs
git rm src/Wif.Gis/Projection/CrsManager.cs
git commit -m "feat(gis): rewrite CoordinateTransformer with GDAL/PROJ, delete CrsManager"
```

---

### Task 4: ChinaCrsHelper (agent: projection)

**Files:**
- Create: `src/Wif.Gis/Projection/ChinaCrsHelper.cs`
- Test: `tests/Wif.Gis.Tests/Projection/ChinaCrsHelperTests.cs`

- [ ] **Step 1: Write failing tests**

```csharp
// tests/Wif.Gis.Tests/Projection/ChinaCrsHelperTests.cs
using Wif.Gis.Projection;

namespace Wif.Gis.Tests.Projection;

public class ChinaCrsHelperTests
{
    [Fact]
    public void Wgs84ToGcj02_Beijing_OffsetApplied()
    {
        // WGS84 coordinates for a point in Beijing
        var (lon, lat) = ChinaCrsHelper.Wgs84ToGcj02(116.3975, 39.9085);

        // GCJ-02 should be offset from WGS84 (typically 0.002~0.006 degrees in China)
        Assert.NotEqual(116.3975, lon, 4);
        Assert.NotEqual(39.9085, lat, 4);
        Assert.InRange(lon, 116.39, 116.41);
        Assert.InRange(lat, 39.90, 39.92);
    }

    [Fact]
    public void Gcj02ToWgs84_RoundTrip()
    {
        var originalLon = 116.3975;
        var originalLat = 39.9085;

        var (gcjLon, gcjLat) = ChinaCrsHelper.Wgs84ToGcj02(originalLon, originalLat);
        var (backLon, backLat) = ChinaCrsHelper.Gcj02ToWgs84(gcjLon, gcjLat);

        Assert.InRange(backLon, originalLon - 0.0001, originalLon + 0.0001);
        Assert.InRange(backLat, originalLat - 0.0001, originalLat + 0.0001);
    }

    [Fact]
    public void Gcj02ToBd09_Offset()
    {
        var (bdLon, bdLat) = ChinaCrsHelper.Gcj02ToBd09(116.404, 39.915);

        // BD-09 adds further offset from GCJ-02
        Assert.NotEqual(116.404, bdLon, 3);
        Assert.NotEqual(39.915, bdLat, 3);
    }

    [Fact]
    public void Bd09ToGcj02_RoundTrip()
    {
        var gcjLon = 116.404;
        var gcjLat = 39.915;

        var (bdLon, bdLat) = ChinaCrsHelper.Gcj02ToBd09(gcjLon, gcjLat);
        var (backLon, backLat) = ChinaCrsHelper.Bd09ToGcj02(bdLon, bdLat);

        Assert.InRange(backLon, gcjLon - 0.0001, gcjLon + 0.0001);
        Assert.InRange(backLat, gcjLat - 0.0001, gcjLat + 0.0001);
    }

    [Fact]
    public void Wgs84ToGcj02_OutsideChina_NoOffset()
    {
        // Tokyo, Japan - outside China, should return same coordinates
        var (lon, lat) = ChinaCrsHelper.Wgs84ToGcj02(139.6917, 35.6895);

        Assert.Equal(139.6917, lon, 4);
        Assert.Equal(35.6895, lat, 4);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `dotnet test tests/Wif.Gis.Tests --filter "FullyQualifiedName~ChinaCrsHelperTests" -v quiet`
Expected: FAIL — `ChinaCrsHelper` does not exist.

- [ ] **Step 3: Implement ChinaCrsHelper**

```csharp
// src/Wif.Gis/Projection/ChinaCrsHelper.cs
namespace Wif.Gis.Projection;

/// <summary>
/// 中国非标准坐标系偏移工具。
/// GCJ-02 (火星坐标) 和 BD-09 (百度坐标) 不在 PROJ 中，需自行实现。
/// </summary>
public static class ChinaCrsHelper
{
    private const double Pi = Math.PI;
    private const double A = 6378245.0;       // GCJ-02 椭球体长半轴
    private const double Ee = 0.00669342162296594323;  // GCJ-02 偏心率平方
    private const double XPi = Pi * 3000.0 / 180.0;

    /// <summary>WGS84 → GCJ-02。中国境外坐标原样返回。</summary>
    public static (double lon, double lat) Wgs84ToGcj02(double lon, double lat)
    {
        if (!IsInsideChina(lon, lat))
            return (lon, lat);

        var (dLon, dLat) = Delta(lon, lat);
        return (lon + dLon, lat + dLat);
    }

    /// <summary>GCJ-02 → WGS84（迭代逆算，精度 < 0.5m）。</summary>
    public static (double lon, double lat) Gcj02ToWgs84(double gcjLon, double gcjLat)
    {
        if (!IsInsideChina(gcjLon, gcjLat))
            return (gcjLon, gcjLat);

        var wgsLon = gcjLon;
        var wgsLat = gcjLat;

        for (int i = 0; i < 5; i++)
        {
            var (fwdLon, fwdLat) = Wgs84ToGcj02(wgsLon, wgsLat);
            wgsLon += gcjLon - fwdLon;
            wgsLat += gcjLat - fwdLat;
        }

        return (wgsLon, wgsLat);
    }

    /// <summary>GCJ-02 → BD-09。</summary>
    public static (double lon, double lat) Gcj02ToBd09(double gcjLon, double gcjLat)
    {
        var z = Math.Sqrt(gcjLon * gcjLon + gcjLat * gcjLat) + 0.00002 * Math.Sin(gcjLat * XPi);
        var theta = Math.Atan2(gcjLat, gcjLon) + 0.000003 * Math.Cos(gcjLon * XPi);
        var bdLon = z * Math.Cos(theta) + 0.0065;
        var bdLat = z * Math.Sin(theta) + 0.006;
        return (bdLon, bdLat);
    }

    /// <summary>BD-09 → GCJ-02。</summary>
    public static (double lon, double lat) Bd09ToGcj02(double bdLon, double bdLat)
    {
        var x = bdLon - 0.0065;
        var y = bdLat - 0.006;
        var z = Math.Sqrt(x * x + y * y) - 0.00002 * Math.Sin(y * XPi);
        var theta = Math.Atan2(y, x) - 0.000003 * Math.Cos(x * XPi);
        var gcjLon = z * Math.Cos(theta);
        var gcjLat = z * Math.Sin(theta);
        return (gcjLon, gcjLat);
    }

    /// <summary>粗略判断坐标是否在中国境内。</summary>
    private static bool IsInsideChina(double lon, double lat)
        => lon >= 72.004 && lon <= 137.8347 && lat >= 0.8293 && lat <= 55.8271;

    private static (double dLon, double dLat) Delta(double lon, double lat)
    {
        var radLat = lat / 180.0 * Pi;
        var magic = Math.Sin(radLat);
        magic = 1 - Ee * magic * magic;
        var sqrtMagic = Math.Sqrt(magic);

        var dLat = TransformLat(lon - 105.0, lat - 35.0);
        var dLon = TransformLon(lon - 105.0, lat - 35.0);
        dLat = dLat * 180.0 / (A * (1 - Ee) / (magic * sqrtMagic) * Pi);
        dLon = dLon * 180.0 / (A / sqrtMagic * Math.Cos(radLat) * Pi);
        return (dLon, dLat);
    }

    private static double TransformLat(double x, double y)
    {
        var ret = -100.0 + 2.0 * x + 3.0 * y + 0.2 * y * y + 0.1 * x * y + 0.2 * Math.Sqrt(Math.Abs(x));
        ret += (20.0 * Math.Sin(6.0 * x * Pi) + 20.0 * Math.Sin(2.0 * x * Pi)) * 2.0 / 3.0;
        ret += (20.0 * Math.Sin(y * Pi) + 40.0 * Math.Sin(y / 3.0 * Pi)) * 2.0 / 3.0;
        ret += (160.0 * Math.Sin(y / 12.0 * Pi) + 320 * Math.Sin(y * Pi / 30.0)) * 2.0 / 3.0;
        return ret;
    }

    private static double TransformLon(double x, double y)
    {
        var ret = 300.0 + x + 2.0 * y + 0.1 * x * x + 0.1 * x * y + 0.1 * Math.Sqrt(Math.Abs(x));
        ret += (20.0 * Math.Sin(6.0 * x * Pi) + 20.0 * Math.Sin(2.0 * x * Pi)) * 2.0 / 3.0;
        ret += (20.0 * Math.Sin(x * Pi) + 40.0 * Math.Sin(x / 3.0 * Pi)) * 2.0 / 3.0;
        ret += (150.0 * Math.Sin(x / 12.0 * Pi) + 300.0 * Math.Sin(x / 30.0 * Pi)) * 2.0 / 3.0;
        return ret;
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `dotnet test tests/Wif.Gis.Tests --filter "FullyQualifiedName~ChinaCrsHelperTests" -v quiet`
Expected: PASS (all 5 tests).

- [ ] **Step 5: Commit**

```bash
git add src/Wif.Gis/Projection/ChinaCrsHelper.cs tests/Wif.Gis.Tests/Projection/ChinaCrsHelperTests.cs
git commit -m "feat(gis): add ChinaCrsHelper for WGS84/GCJ-02/BD-09 offsets"
```

---

### Task 5: RasterLayerService (agent: raster)

**Files:**
- Create: `src/Wif.Gis/Services/RasterLayerInfo.cs`
- Create: `src/Wif.Gis/Services/RasterLayerService.cs`
- Test: `tests/Wif.Gis.Tests/Services/RasterLayerServiceTests.cs`

- [ ] **Step 1: Write failing tests**

```csharp
// tests/Wif.Gis.Tests/Services/RasterLayerServiceTests.cs
using Wif.Gis.Services;

namespace Wif.Gis.Tests.Services;

public class RasterLayerServiceTests : IClassFixture<GdalFixture>
{
    private readonly RasterLayerService _sut = new();

    [Fact]
    public void LoadRaster_ValidGeoTiff_ReturnsInfo()
    {
        var path = CreateTestGeoTiff();

        var info = _sut.LoadRaster(path);

        Assert.Equal(path, info.FilePath);
        Assert.Equal(4, info.Width);
        Assert.Equal(4, info.Height);
        Assert.Equal(1, info.BandCount);
        Assert.True(info.MaxX > info.MinX);
        Assert.True(info.MaxY > info.MinY);

        File.Delete(path);
    }

    [Fact]
    public void GetElevation_ValidPoint_ReturnsValue()
    {
        var path = CreateTestGeoTiff();
        var info = _sut.LoadRaster(path);

        // Query a point within the raster extent
        var midLon = (info.MinX + info.MaxX) / 2;
        var midLat = (info.MinY + info.MaxY) / 2;
        var elev = _sut.GetElevation(info, midLon, midLat);

        Assert.NotNull(elev);

        File.Delete(path);
    }

    [Fact]
    public void GetElevation_OutsideExtent_ReturnsNull()
    {
        var path = CreateTestGeoTiff();
        var info = _sut.LoadRaster(path);

        var elev = _sut.GetElevation(info, 0, 0); // far outside extent

        Assert.Null(elev);

        File.Delete(path);
    }

    [Fact]
    public void CreateMapLayer_ReturnsNonNull()
    {
        var path = CreateTestGeoTiff();
        var info = _sut.LoadRaster(path);

        var layer = _sut.CreateMapLayer(info);

        Assert.NotNull(layer);
        Assert.Contains(info.Name, layer.Name);

        File.Delete(path);
    }

    /// <summary>
    /// 创建一个 4x4 的最小 GeoTiff 测试文件。
    /// 范围: 116.0~117.0E, 39.0~40.0N，像素值 100~115。
    /// </summary>
    private static string CreateTestGeoTiff()
    {
        var path = Path.Combine(Path.GetTempPath(), $"test_{Guid.NewGuid():N}.tif");

        var drv = OSGeo.GDAL.Gdal.GetDriverByName("GTiff");
        var ds = drv.Create(path, 4, 4, 1, OSGeo.GDAL.DataType.GDT_Float32, null);

        // GeoTransform: [originX, pixelWidth, 0, originY, 0, -pixelHeight]
        ds.SetGeoTransform([116.0, 0.25, 0, 40.0, 0, -0.25]);

        var srs = new OSGeo.OSR.SpatialReference("");
        srs.ImportFromEPSG(4326);
        srs.ExportToWkt(out var wkt, null);
        ds.SetProjection(wkt);

        var band = ds.GetRasterBand(1);
        var data = new float[] { 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115 };
        band.WriteRaster(0, 0, 4, 4, data, 4, 4, 0, 0);
        band.FlushCache();
        ds.FlushCache();
        ds.Dispose();

        return path;
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `dotnet test tests/Wif.Gis.Tests --filter "FullyQualifiedName~RasterLayerServiceTests" -v quiet`
Expected: FAIL — classes do not exist.

- [ ] **Step 3: Implement RasterLayerInfo model**

```csharp
// src/Wif.Gis/Services/RasterLayerInfo.cs
namespace Wif.Gis.Services;

/// <summary>
/// 栅格图层元数据（GeoTIFF/DEM等）。
/// 栅格文件不存入数据库，通过 FilePath 引用外部文件。
/// </summary>
public sealed class RasterLayerInfo
{
    public required string FilePath { get; init; }
    public required string Name { get; init; }
    public int Width { get; init; }
    public int Height { get; init; }
    public int BandCount { get; init; }
    public string CrsCode { get; init; } = "EPSG:4326";
    public double MinX { get; init; }
    public double MinY { get; init; }
    public double MaxX { get; init; }
    public double MaxY { get; init; }

    /// <summary>GeoTransform 六参数: [originX, pixelWidth, rotX, originY, rotY, -pixelHeight]</summary>
    internal double[] GeoTransform { get; init; } = new double[6];
}
```

- [ ] **Step 4: Implement RasterLayerService**

```csharp
// src/Wif.Gis/Services/RasterLayerService.cs
using Mapsui;
using Mapsui.Layers;
using Mapsui.Projections;
using Mapsui.Styles;
using OSGeo.GDAL;

namespace Wif.Gis.Services;

/// <summary>
/// 栅格图层服务 — 只读加载 GeoTIFF/DEM，渲染为 Mapsui 图层。
/// </summary>
public sealed class RasterLayerService
{
    /// <summary>
    /// 读取栅格文件元数据。
    /// </summary>
    public RasterLayerInfo LoadRaster(string filePath)
    {
        using var ds = Gdal.Open(filePath, Access.GA_ReadOnly);
        if (ds is null)
            throw new InvalidOperationException($"无法打开栅格文件: {filePath}");

        var gt = new double[6];
        ds.GetGeoTransform(gt);

        var width = ds.RasterXSize;
        var height = ds.RasterYSize;

        // Compute extent from GeoTransform
        var minX = gt[0];
        var maxY = gt[3];
        var maxX = gt[0] + width * gt[1] + height * gt[2];
        var minY = gt[3] + width * gt[4] + height * gt[5];

        // Detect CRS
        var crsCode = "EPSG:4326";
        var projWkt = ds.GetProjection();
        if (!string.IsNullOrEmpty(projWkt))
        {
            using var srs = new OSGeo.OSR.SpatialReference(projWkt);
            srs.AutoIdentifyEPSG();
            var code = srs.GetAuthorityCode(null);
            if (!string.IsNullOrEmpty(code))
                crsCode = $"EPSG:{code}";
        }

        return new RasterLayerInfo
        {
            FilePath = filePath,
            Name = Path.GetFileNameWithoutExtension(filePath),
            Width = width,
            Height = height,
            BandCount = ds.RasterCount,
            CrsCode = crsCode,
            MinX = Math.Min(minX, maxX),
            MinY = Math.Min(minY, maxY),
            MaxX = Math.Max(minX, maxX),
            MaxY = Math.Max(minY, maxY),
            GeoTransform = gt
        };
    }

    /// <summary>
    /// 创建 Mapsui 可渲染图层（MemoryLayer + 范围框）。
    /// 对于简单展示，渲染栅格的地理范围框。
    /// 完整像素级渲染需要按视口裁剪重采样（后续增强）。
    /// </summary>
    public ILayer CreateMapLayer(RasterLayerInfo info)
    {
        // Project extent to Mercator for Mapsui display
        var (minX, minY) = SphericalMercator.FromLonLat(info.MinX, info.MinY);
        var (maxX, maxY) = SphericalMercator.FromLonLat(info.MaxX, info.MaxY);

        var extent = new MRect(minX, minY, maxX, maxY);

        var layer = new MemoryLayer(info.Name)
        {
            Features = [new RectFeature(extent)],
            Style = new VectorStyle
            {
                Fill = new Brush(new Color(100, 150, 100, 80)),
                Outline = new Pen(new Color(80, 120, 80), 2)
            }
        };

        return layer;
    }

    /// <summary>
    /// 查询指定经纬度处的栅格值（第一波段）。
    /// 坐标在栅格范围外时返回 null。
    /// </summary>
    public double? GetElevation(RasterLayerInfo info, double lon, double lat)
    {
        if (lon < info.MinX || lon > info.MaxX || lat < info.MinY || lat > info.MaxY)
            return null;

        using var ds = Gdal.Open(info.FilePath, Access.GA_ReadOnly);
        if (ds is null) return null;

        var gt = info.GeoTransform;

        // Inverse GeoTransform to get pixel coordinates
        // For simple north-up rasters: col = (lon - originX) / pixelWidth, row = (originY - lat) / pixelHeight
        if (gt[2] != 0 || gt[4] != 0)
            return null; // Rotated rasters not supported for point query

        var col = (int)((lon - gt[0]) / gt[1]);
        var row = (int)((lat - gt[3]) / gt[5]);

        if (col < 0 || col >= info.Width || row < 0 || row >= info.Height)
            return null;

        var band = ds.GetRasterBand(1);
        var buf = new double[1];
        band.ReadRaster(col, row, 1, 1, buf, 1, 1, 0, 0);

        // Check nodata
        band.GetNoDataValue(out var nodata, out var hasNodata);
        if (hasNodata != 0 && Math.Abs(buf[0] - nodata) < 0.001)
            return null;

        return buf[0];
    }
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `dotnet test tests/Wif.Gis.Tests --filter "FullyQualifiedName~RasterLayerServiceTests" -v quiet`
Expected: PASS (all 4 tests).

- [ ] **Step 6: Commit**

```bash
git add src/Wif.Gis/Services/RasterLayerInfo.cs src/Wif.Gis/Services/RasterLayerService.cs tests/Wif.Gis.Tests/Services/RasterLayerServiceTests.cs
git commit -m "feat(gis): add RasterLayerService for GeoTIFF/DEM read-only support"
```

---

## Wave 2 — Integration

### Task 6: Domain, Data, DI, ViewModel Changes (agent: integration)

**Files:**
- Modify: `src/Wif.Domain/Enums/GisLayerSourceType.cs`
- Modify: `src/Wif.Domain/Entities/GisLayer.cs`
- Modify: `src/Wif.Data/Configurations/GisLayerConfiguration.cs`
- Modify: `src/Wif.Data/Migrations/GisMigration.cs`
- Modify: `src/Wif.Core/Services/DependencyInjection.cs`
- Modify: `src/Wif.Core/Modules/GisModule.cs`
- Modify: `src/Wif.App/ViewModels/GisViewModel.cs`

- [ ] **Step 1: Add Raster to GisLayerSourceType**

In `src/Wif.Domain/Enums/GisLayerSourceType.cs`:

```csharp
namespace Wif.Domain.Enums;

public enum GisLayerSourceType
{
    Vector,
    Tile,
    Raster
}
```

- [ ] **Step 2: Add FilePath to GisLayer**

In `src/Wif.Domain/Entities/GisLayer.cs`, add after `CrsCode`:

```csharp
    public string? FilePath { get; set; }
```

- [ ] **Step 3: Map FilePath in EF configuration**

In `src/Wif.Data/Configurations/GisLayerConfiguration.cs`, add before the `HasMany` call:

```csharp
        builder.Property(l => l.FilePath)
            .HasMaxLength(1000);
```

- [ ] **Step 4: Add FilePath migration SQL**

In `src/Wif.Data/Migrations/GisMigration.cs`, add at the end of `Apply` method:

```csharp
        try { db.Database.ExecuteSqlRaw("ALTER TABLE GisLayers ADD COLUMN FilePath TEXT NULL"); }
        catch { /* already exists */ }
```

- [ ] **Step 5: Update DependencyInjection.cs**

Replace the GIS service registration block in `src/Wif.Core/Services/DependencyInjection.cs`:

```csharp
        // GIS -- core services
        services.AddSingleton<GeometryService>();
        services.AddSingleton<GdalDataService>();
        services.AddSingleton<GisFileService>();
        services.AddSingleton<GisToolRegistry>();

        // GIS -- map rendering
        services.AddSingleton<FeatureStyleProvider>();
        services.AddSingleton<TileSourceFactory>();
        services.AddSingleton<MapService>();

        // GIS -- editing tools
        services.AddSingleton<ToolManager>();

        // GIS -- spatial analysis
        services.AddSingleton<SpatialAnalysisService>();
        services.AddSingleton<MeasurementService>();
        services.AddSingleton<CoordinateTransformer>();
        services.AddSingleton<RasterLayerService>();

        // GIS -- WorkItem integration
        services.AddSingleton<WorkItemMapService>();
        services.AddSingleton<GeocodingService>();
        services.AddSingleton<ClusterRenderer>();
```

Changes: Added `GdalDataService`, `RasterLayerService`. Removed `CrsManager`. `GisFileService` now depends on `GdalDataService` (resolved by DI). `CoordinateTransformer` no longer needs `CrsManager`.

- [ ] **Step 6: Initialize GDAL in GisModule**

Replace `src/Wif.Core/Modules/GisModule.cs`:

```csharp
using MaxRev.Gdal.Core;
using Microsoft.Extensions.DependencyInjection;

namespace Wif.Core.Modules;

public class GisModule : IWifModule
{
    public string Id => "gis";
    public string Name => "GIS 空间分析";
    public string? Description => "空间几何分析、地图渲染与图层管理";
    public bool IsEnabled { get; set; } = true;

    public void ConfigureServices(IServiceCollection services) { }

    public Task StartAsync(CancellationToken ct)
    {
        GdalBase.ConfigureAll();
        return Task.CompletedTask;
    }

    public Task StopAsync(CancellationToken ct) => Task.CompletedTask;
}
```

- [ ] **Step 7: Update GisViewModel — add unified ImportFileAsync and raster import**

Add to `src/Wif.App/ViewModels/GisViewModel.cs` — update the constructor to accept `RasterLayerService`:

```csharp
    private readonly RasterLayerService _rasterService;

    public GisViewModel(
        IGisLayerRepository repository,
        GisFileService fileService,
        GeometryService geometryService,
        MapService mapService,
        ToolManager toolManager,
        RasterLayerService rasterService)
    {
        _repository = repository;
        _fileService = fileService;
        _geometryService = geometryService;
        _mapService = mapService;
        _toolManager = toolManager;
        _rasterService = rasterService;

        _toolManager.ToolChanged += toolType =>
            ActiveToolName = toolType.ToString();
    }
```

Add a new unified import command after `ImportShapefileAsync`:

```csharp
    [RelayCommand]
    private async Task ImportFileAsync()
    {
        var topLevel = GetTopLevel();
        if (topLevel is null) return;

        try
        {
            var files = await topLevel.StorageProvider.OpenFilePickerAsync(new FilePickerOpenOptions
            {
                Title = "导入空间数据",
                AllowMultiple = false,
                FileTypeFilter =
                [
                    new FilePickerFileType("所有空间数据") { Patterns = ["*.shp", "*.geojson", "*.json", "*.gpkg", "*.kml", "*.gml", "*.tif", "*.tiff"] },
                    new FilePickerFileType("Shapefile") { Patterns = ["*.shp"] },
                    new FilePickerFileType("GeoJSON") { Patterns = ["*.geojson", "*.json"] },
                    new FilePickerFileType("GeoPackage") { Patterns = ["*.gpkg"] },
                    new FilePickerFileType("KML") { Patterns = ["*.kml"] },
                    new FilePickerFileType("GeoTIFF") { Patterns = ["*.tif", "*.tiff"] }
                ]
            });

            if (files.Count == 0) return;

            IsLoading = true;
            var file = files[0];
            var localPath = file.TryGetLocalPath();
            if (localPath is null) { StatusMessage = "无法获取文件路径"; return; }

            var ext = Path.GetExtension(localPath).ToLowerInvariant();

            if (ext is ".tif" or ".tiff")
            {
                StatusMessage = "正在导入栅格...";
                var info = _rasterService.LoadRaster(localPath);
                var mapLayer = _rasterService.CreateMapLayer(info);

                var now = DateTimeOffset.UtcNow;
                var gisLayer = new GisLayer
                {
                    Id = Ulid.NewUlid(),
                    Name = info.Name,
                    SourceType = GisLayerSourceType.Raster,
                    FilePath = localPath,
                    CrsCode = info.CrsCode,
                    CreatedAt = now,
                    UpdatedAt = now
                };
                await _repository.CreateLayerAsync(gisLayer);

                Layers.Add(new GisLayerItem
                {
                    Id = gisLayer.Id,
                    Name = gisLayer.Name,
                    IsVisible = true,
                    FeatureCount = 0,
                    GeometryType = "栅格"
                });

                _mapService.Map.Layers.Add(mapLayer);
                StatusMessage = $"已导入栅格: {info.Name} ({info.Width}x{info.Height})";
            }
            else
            {
                StatusMessage = "正在导入矢量数据...";
                var layer = _fileService.ImportFile(localPath);
                await _repository.CreateLayerAsync(layer);

                Layers.Add(new GisLayerItem
                {
                    Id = layer.Id,
                    Name = layer.Name,
                    IsVisible = true,
                    FeatureCount = layer.Features.Count,
                    GeometryType = layer.Features.Count > 0 ? layer.Features[0].GeometryType : "混合"
                });

                _mapService.AddOrUpdateLayer(layer);
                StatusMessage = $"已导入 {layer.Name}: {layer.Features.Count} 个要素";
            }
        }
        catch (Exception ex)
        {
            StatusMessage = $"导入失败: {ex.Message}";
        }
        finally
        {
            IsLoading = false;
        }
    }
```

- [ ] **Step 8: Build entire solution**

Run: `dotnet build Wif.sln --verbosity quiet`
Expected: 0 errors, 0 warnings.

- [ ] **Step 9: Commit**

```bash
git add src/Wif.Domain/Enums/GisLayerSourceType.cs src/Wif.Domain/Entities/GisLayer.cs src/Wif.Data/Configurations/GisLayerConfiguration.cs src/Wif.Data/Migrations/GisMigration.cs src/Wif.Core/Services/DependencyInjection.cs src/Wif.Core/Modules/GisModule.cs src/Wif.App/ViewModels/GisViewModel.cs
git commit -m "feat(gis): integrate GDAL services — domain, DI, ViewModel wiring"
```

---

## Wave 3 — Verification

### Task 7: Build + Test Verification (agent: assembler)

- [ ] **Step 1: Full solution build**

Run: `dotnet build Wif.sln --verbosity quiet`
Expected: 0 errors, 0 warnings.

- [ ] **Step 2: Run all existing tests**

Run: `dotnet test Wif.sln --verbosity quiet`
Expected: All tests pass.

- [ ] **Step 3: Run new GIS tests specifically**

Run: `dotnet test tests/Wif.Gis.Tests --verbosity normal`
Expected: All 20+ new tests pass.

- [ ] **Step 4: Verify GDAL initialization**

Run a quick smoke test to confirm GDAL loads at runtime:

```bash
dotnet run --project tests/Wif.Gis.Tests -- --filter "FullyQualifiedName~GdalDataServiceTests.GetSupportedVectorFormats_ContainsExpectedDrivers"
```

Expected: Test passes, confirming GDAL native libraries load and drivers register.

- [ ] **Step 5: Final commit (if any fixups needed)**

```bash
git add -A
git commit -m "fix(gis): resolve build/test issues from GDAL integration"
```

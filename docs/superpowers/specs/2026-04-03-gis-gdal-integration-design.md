# GIS GDAL Integration Design

**Date:** 2026-04-03
**Status:** Approved
**Approach:** IO Layer Refactoring (Method B)

## Overview

Integrate GDAL into the existing Wif.Gis module to replace ProjNET and NTS.IO.ShapeFile, providing broader vector format support, raster data capabilities, and stronger coordinate system transformations. The existing Mapsui + NTS architecture is preserved; GDAL serves as the data IO and projection backbone.

## Dependency Changes

### Remove

| Package | Reason |
|---------|--------|
| `NetTopologySuite.IO.ShapeFile 2.1.0` | GDAL OGR covers Shapefile and many more formats |
| `ProjNET 2.0.0` | GDAL embeds PROJ with full EPSG database (8000+ CRS) |

### Add

| Package | Version | Purpose |
|---------|---------|---------|
| `MaxRev.Gdal.Core` | 3.12.2 | GDAL core C# bindings |
| `MaxRev.Gdal.WindowsRuntime.Minimal` | 3.12.2 | Windows native libraries |

### Retain (unchanged)

| Package | Purpose |
|---------|---------|
| `Mapsui.Avalonia 5.0.2` | Map control |
| `Mapsui.Nts 5.0.2` | NTS-Mapsui bridge |
| `NetTopologySuite 2.6.0` | Geometry computation engine |
| `NetTopologySuite.IO.GeoJSON 4.0.0` | GeoJSON IO (lighter than GDAL for this format) |
| `NetTopologySuite.Features 2.2.0` | Feature/AttributesTable |

### Data Flow Division

```
GDAL (OGR)  -> Shapefile, GeoPackage, KML, GML, DXF, MapInfo, CSV, GeoTIFF
NTS IO      -> GeoJSON (retained, lighter for this format)
NTS         -> All geometry operations (Buffer, Intersection, Union...)
GDAL (PROJ) -> Coordinate system transformations (replaces ProjNET)
```

## New Files

### 1. GdalDataService (`Wif.Gis/Services/GdalDataService.cs`)

Unified wrapper for GDAL OGR vector read/write.

**API:**

```csharp
public sealed class GdalDataService
{
    GisLayer ImportVector(string filePath, string layerName);
    void ExportVector(GisLayer layer, string filePath, string driverName);
    RasterLayerInfo ImportRaster(string filePath);
    string[] GetSupportedVectorFormats();
    string DetectFormat(string filePath);
}
```

**Data conversion pipeline:**

```
File (Shapefile/GeoPackage/KML/...)
  | GDAL OGR.Open()
OGR Feature (geometry + attributes)
  | ExportToWkt()
WKT string
  | stored in GisFeature.GeometryWkt
NTS geometry pipeline (analysis, map rendering, etc.)
```

Export reverses: NTS Geometry -> WKT -> OGR.CreateGeometryFromWkt() -> write to target format.

**Supported formats:**

| Format | Driver | Read | Write |
|--------|--------|:----:|:-----:|
| Shapefile | ESRI Shapefile | Y | Y |
| GeoPackage | GPKG | Y | Y |
| KML | KML | Y | Y |
| GML | GML | Y | Y |
| CSV (coords) | CSV | Y | Y |
| GeoJSON | GeoJSON | Y | Y |
| MapInfo | MapInfo File | Y | Y |
| DXF | DXF | Y | Y |

### 2. RasterLayerService (`Wif.Gis/Services/RasterLayerService.cs`)

Read-only raster support for GeoTIFF/DEM display.

**API:**

```csharp
public sealed class RasterLayerService
{
    RasterLayerInfo LoadRaster(string filePath);
    ILayer CreateMapLayer(RasterLayerInfo info);
    double? GetElevation(RasterLayerInfo info, double lon, double lat);
    string GenerateHillshade(RasterLayerInfo info, string outputPath);
}
```

**RasterLayerInfo model:**

```csharp
public sealed class RasterLayerInfo
{
    public required string FilePath { get; init; }
    public required string Name { get; init; }
    public int Width { get; init; }
    public int Height { get; init; }
    public int BandCount { get; init; }
    public string CrsCode { get; init; }
    public double MinX { get; init; }
    public double MinY { get; init; }
    public double MaxX { get; init; }
    public double MaxY { get; init; }
}
```

**Rendering strategy:** GDAL reads GeoTIFF -> clips/resamples to current viewport -> bitmap -> Mapsui RasterizingLayer. Viewport changes trigger re-sampling with throttle/debounce.

**Raster files are NOT stored in SQLite** (too large). GisLayer references external file via `FilePath` field.

**Out of scope:** No band math (NDVI), no raster editing, no mosaic stitching.

### 3. ChinaCrsHelper (`Wif.Gis/Projection/ChinaCrsHelper.cs`)

Static utility for non-standard Chinese coordinate offsets.

```csharp
public static class ChinaCrsHelper
{
    (double lon, double lat) Wgs84ToGcj02(double lon, double lat);
    (double lon, double lat) Gcj02ToWgs84(double lon, double lat);
    (double lon, double lat) Gcj02ToBd09(double lon, double lat);
    (double lon, double lat) Bd09ToGcj02(double lon, double lat);
}
```

GCJ-02/BD-09 are proprietary offsets not in PROJ. These remain self-implemented.

## Refactored Files

### 4. CoordinateTransformer (rewrite)

Replace ProjNET with GDAL/PROJ:

```csharp
// Before (ProjNET):
var sourceCrs = _crsManager.GetCrs("EPSG:4326");  // manual WKT registry
var transform = factory.CreateFromCoordinateSystems(sourceCrs, targetCrs);

// After (GDAL/PROJ):
var sourceSrs = new SpatialReference("");
sourceSrs.ImportFromEPSG(4326);  // built-in global EPSG database
var transform = new CoordinateTransformation(sourceSrs, targetSrs);
```

Interface preserved:

```csharp
public sealed class CoordinateTransformer
{
    Geometry Transform(Geometry geometry, string sourceCrs, string targetCrs);
    string TransformWkt(string wkt, string sourceCrs, string targetCrs);
    bool IsSupported(string crsCode);  // new: check EPSG availability
}
```

Constructor simplified: no CrsManager dependency.

### 5. GisFileService (partial rewrite)

- `ImportShapefile` / `ExportShapefile` -> delegate to `GdalDataService`
- `ImportGeoJson` / `ExportGeoJson` -> retain NTS implementation (lighter)
- New `ImportFile(filePath)` -> auto-detect format via `GdalDataService.DetectFormat`, unified entry point

## Deleted Files

| File | Reason |
|------|--------|
| `CrsManager.cs` | GDAL has built-in EPSG database; manual WKT registry no longer needed |

## Minor Changes

| File | Change |
|------|--------|
| `GisLayerSourceType.cs` | Add `Raster` enum value |
| `GisLayer.cs` | Add optional `string? FilePath` property (for raster layers) |
| `GisLayerConfiguration.cs` | Map new `FilePath` column |
| `GisMigration.cs` | Add `FilePath` column migration SQL |
| `DependencyInjection.cs` | Register `GdalDataService`, `RasterLayerService`; remove `CrsManager` |
| `GisModule.cs` | Call `GdalBase.ConfigureAll()` in `StartAsync` |
| `GisViewModel.cs` | Extend file picker with GeoPackage/KML/GeoTIFF; add `ImportFileAsync` unified entry |

## Unchanged Code (15+ files)

All geometry computation, map rendering, spatial analysis, editing tools, clustering, and WorkItem integration remain untouched:

- `GeometryService.cs` - pure NTS wrapper
- `MapService.cs` - Mapsui map management
- `TileSourceFactory.cs` - tile basemaps
- `SpatialAnalysisService.cs` - Buffer/Intersection/Union/Difference
- `MeasurementService.cs` - distance/area measurement
- `FeatureStyleProvider.cs` - Mapsui style mapping
- `ClusterRenderer.cs` - clustering algorithm
- `WorkItemMapService.cs` - work item markers
- `ToolManager.cs` + all tool implementations (Drawing/Editing/Selection/Measure)
- `GisToolDefinitions.cs` - AI tool registry
- `AnalysisResult.cs` - analysis result model

## Known Bug Fix

**Layer deletion blocking:** Investigate `GisLayerRepository.DeleteLayerAsync` - likely EF Core cascade delete or SQLite lock issue. Diagnose and fix during implementation.

## Implementation Strategy: Parallel Agent Team

Use an agent team to implement changes in parallel. Work is divided into independent streams that can execute concurrently, with a final assembly phase.

### Wave 1 (parallel — no dependencies between streams)

| Agent | Responsibility | Files |
|-------|---------------|-------|
| **gdal-io** | `GdalDataService`, `GisFileService` rewrite, NuGet changes | Services/GdalDataService.cs, Services/GisFileService.cs, Wif.Gis.csproj |
| **projection** | `CoordinateTransformer` rewrite, `ChinaCrsHelper`, delete `CrsManager` | Projection/CoordinateTransformer.cs, Projection/ChinaCrsHelper.cs |
| **raster** | `RasterLayerService`, `RasterLayerInfo` model | Services/RasterLayerService.cs, Services/RasterLayerInfo.cs |

### Wave 2 (after Wave 1 completes — depends on new services)

| Agent | Responsibility | Files |
|-------|---------------|-------|
| **integration** | Domain changes, DI wiring, migration, ViewModel updates, bug fix | GisLayer.cs, GisLayerSourceType.cs, GisLayerConfiguration.cs, GisMigration.cs, DependencyInjection.cs, GisModule.cs, GisViewModel.cs, GisLayerRepository.cs |

### Wave 3 (after Wave 2 — validation)

| Agent | Responsibility |
|-------|---------------|
| **assembler** | Build verification, test run, end-to-end smoke test |

## Summary

| Metric | Count |
|--------|-------|
| New files | 3 |
| Rewritten files | 2 |
| Deleted files | 1 |
| Minor changes | 7 |
| Unchanged files | 15+ |
| NuGet added | 2 |
| NuGet removed | 2 |

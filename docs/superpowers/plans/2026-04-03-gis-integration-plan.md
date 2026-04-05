# gis-integration Implementation Plan

> **For agentic workers:** Use this plan to implement the gis-integration module task-by-task.

**Goal:** Deep integration between WorkItems and GIS — location fields, map markers, clustering, geocoding, popup info cards, and bidirectional navigation.

**Architecture:** WorkItem gets optional lat/lon + GisFeatureId. WorkItemMapService generates map layers. ClusterRenderer groups markers. GeocodingService resolves addresses.

**Tech Stack:** Avalonia, CommunityToolkit.Mvvm, EF Core, NetTopologySuite, HttpClient (Nominatim)

---

## Task 1: Database Migration — WorkItemGeoMigration

**Files:**
- `src/Wif.Data/Migrations/WorkItemGeoMigration.cs` (NEW)

**Steps:**

- [ ] 1. Create `src/Wif.Data/Migrations/WorkItemGeoMigration.cs` with the following content:

```csharp
using Microsoft.EntityFrameworkCore;

namespace Wif.Data.Migrations;

public static class WorkItemGeoMigration
{
    public static void Apply(WifDbContext db)
    {
        // Add location columns to WorkItems
        try
        {
            db.Database.ExecuteSqlRaw(
                "ALTER TABLE WorkItems ADD COLUMN Longitude REAL NULL");
        }
        catch { /* column already exists */ }

        try
        {
            db.Database.ExecuteSqlRaw(
                "ALTER TABLE WorkItems ADD COLUMN Latitude REAL NULL");
        }
        catch { /* column already exists */ }

        try
        {
            db.Database.ExecuteSqlRaw(
                "ALTER TABLE WorkItems ADD COLUMN GisFeatureId TEXT NULL");
        }
        catch { /* column already exists */ }

        // Add WorkItemId to GisFeatures
        try
        {
            db.Database.ExecuteSqlRaw(
                "ALTER TABLE GisFeatures ADD COLUMN WorkItemId TEXT NULL");
        }
        catch { /* column already exists */ }

        // Indexes for spatial queries
        db.Database.ExecuteSqlRaw(@"
            CREATE INDEX IF NOT EXISTS IX_WorkItems_Longitude_Latitude
                ON WorkItems (Longitude, Latitude)
                WHERE Longitude IS NOT NULL AND Latitude IS NOT NULL;

            CREATE INDEX IF NOT EXISTS IX_WorkItems_GisFeatureId
                ON WorkItems (GisFeatureId)
                WHERE GisFeatureId IS NOT NULL;

            CREATE INDEX IF NOT EXISTS IX_GisFeatures_WorkItemId
                ON GisFeatures (WorkItemId)
                WHERE WorkItemId IS NOT NULL;
        ");
    }
}
```

- [ ] 2. Register the migration in `src/Wif.App/App.axaml.cs` in `OnFrameworkInitializationCompleted`, after existing migrations:

```csharp
WorkItemGeoMigration.Apply(db);
```

- [ ] 3. Build and verify: `dotnet build src/Wif.Data/Wif.Data.csproj`

---

## Task 2: Domain Entity Changes — WorkItem

**Files:**
- `src/Wif.Domain/Entities/WorkItem.cs` (MODIFY)

**Steps:**

- [ ] 1. Add the following properties to the `WorkItem` class, after `UpdatedAt` and before the navigation properties comment:

```csharp
    // GIS location (optional — simple point location)
    public double? Longitude { get; set; }
    public double? Latitude { get; set; }

    // Link to a full GisFeature (optional — complex spatial relationships)
    public Ulid? GisFeatureId { get; set; }
```

- [ ] 2. Add a navigation property inside the navigation properties section:

```csharp
    public GisFeature? GisFeature { get; set; }
```

The full file should look like:

```csharp
using System.Text.Json;
using Wif.Domain.Enums;
using Wif.Domain.ValueObjects;

namespace Wif.Domain.Entities;

public class WorkItem
{
    public Ulid Id { get; init; }
    public required string Title { get; set; }
    public WorkItemStatus Status { get; set; } = WorkItemStatus.Inbox;
    public required Source Source { get; init; }
    public string? RawContent { get; set; }
    public JsonDocument? StructuredData { get; set; }
    public Ulid? ParentId { get; set; }
    public List<string> Tags { get; set; } = [];
    public DateTimeOffset CreatedAt { get; init; }
    public DateTimeOffset UpdatedAt { get; set; }

    // GIS location (optional — simple point location)
    public double? Longitude { get; set; }
    public double? Latitude { get; set; }

    // Link to a full GisFeature (optional — complex spatial relationships)
    public Ulid? GisFeatureId { get; set; }

    // Navigation properties
    public List<Attachment> Attachments { get; init; } = [];
    public List<WorkEvent> Events { get; init; } = [];
    public List<Proposal> Proposals { get; init; } = [];
    public WorkItem? Parent { get; set; }
    public List<WorkItem> Children { get; init; } = [];
    public GisFeature? GisFeature { get; set; }
}
```

- [ ] 3. Build and verify: `dotnet build src/Wif.Domain/Wif.Domain.csproj`

---

## Task 3: Domain Entity Changes — GisFeature

**Files:**
- `src/Wif.Domain/Entities/GisFeature.cs` (MODIFY)

**Steps:**

- [ ] 1. Add `WorkItemId` property and navigation to `GisFeature`:

```csharp
namespace Wif.Domain.Entities;

public class GisFeature
{
    public Ulid Id { get; init; }
    public Ulid LayerId { get; init; }
    public required string GeometryWkt { get; set; }  // WKT format
    public string GeometryType { get; set; } = "Point";
    public string? Name { get; set; }
    public string? PropertiesJson { get; set; }  // JSON bag for arbitrary properties
    public Ulid? WorkItemId { get; set; }
    public DateTimeOffset CreatedAt { get; init; }
    public DateTimeOffset UpdatedAt { get; set; }

    // Navigation
    public GisLayer? Layer { get; set; }
    public WorkItem? WorkItem { get; set; }
}
```

- [ ] 2. Build and verify: `dotnet build src/Wif.Domain/Wif.Domain.csproj`

---

## Task 4: EF Configuration — WorkItemConfiguration

**Files:**
- `src/Wif.Data/Configurations/WorkItemConfiguration.cs` (MODIFY)

**Steps:**

- [ ] 1. Add column mappings for `Longitude`, `Latitude`, and `GisFeatureId` inside the `Configure` method. Add these after the `Tags` property configuration and before the `HasOne(w => w.Parent)` relationship:

```csharp
        builder.Property(w => w.Longitude);

        builder.Property(w => w.Latitude);

        builder.Property(w => w.GisFeatureId)
            .HasConversion(
                v => v.HasValue ? v.Value.ToString() : null,
                v => v != null ? Ulid.Parse(v) : (Ulid?)null)
            .HasMaxLength(26);
```

- [ ] 2. Add the `GisFeature` navigation relationship. Add after the `HasMany(w => w.Proposals)` configuration:

```csharp
        builder.HasOne(w => w.GisFeature)
            .WithOne(f => f.WorkItem)
            .HasForeignKey<WorkItem>(w => w.GisFeatureId)
            .OnDelete(DeleteBehavior.SetNull);
```

- [ ] 3. Add indexes for spatial queries. Add after the existing indexes:

```csharp
        builder.HasIndex(w => w.GisFeatureId);
```

- [ ] 4. Build and verify: `dotnet build src/Wif.Data/Wif.Data.csproj`

---

## Task 5: EF Configuration — GisFeatureConfiguration

**Files:**
- `src/Wif.Data/Configurations/GisFeatureConfiguration.cs` (MODIFY)

**Steps:**

- [ ] 1. Add `WorkItemId` column mapping inside the `Configure` method, after the `UpdatedAt` property configuration:

```csharp
        builder.Property(f => f.WorkItemId)
            .HasConversion(
                v => v.HasValue ? v.Value.ToString() : null,
                v => v != null ? Ulid.Parse(v) : (Ulid?)null)
            .HasMaxLength(26);
```

- [ ] 2. Add an index for `WorkItemId`. Add after the existing `LayerId` index:

```csharp
        builder.HasIndex(f => f.WorkItemId);
```

Note: The relationship is defined in `WorkItemConfiguration` (Task 4) via `HasOne/WithOne`. Do NOT define it again here to avoid duplicate configuration.

- [ ] 3. Build and verify: `dotnet build src/Wif.Data/Wif.Data.csproj`

---

## Task 6: WorkItemMapService

**Files:**
- `src/Wif.Gis/Services/WorkItemMapService.cs` (NEW)

**Steps:**

- [ ] 1. Create `src/Wif.Gis/Services/WorkItemMapService.cs`:

```csharp
using NetTopologySuite.Geometries;
using Wif.Domain.Entities;
using Wif.Domain.Enums;

namespace Wif.Gis.Services;

/// <summary>
/// 工作项地图服务 — 将带有坐标的工作项转换为地图标记点集合。
/// </summary>
public sealed class WorkItemMapService
{
    private readonly GeometryService _geometryService;

    public WorkItemMapService(GeometryService geometryService)
    {
        _geometryService = geometryService;
    }

    /// <summary>
    /// 从工作项列表生成标记点数据，仅包含有坐标的工作项。
    /// </summary>
    public IReadOnlyList<WorkItemMarker> BuildMarkers(IEnumerable<WorkItem> workItems)
    {
        var markers = new List<WorkItemMarker>();
        foreach (var item in workItems)
        {
            if (item.Longitude is null || item.Latitude is null)
                continue;

            markers.Add(new WorkItemMarker
            {
                WorkItemId = item.Id,
                Title = item.Title,
                Status = item.Status,
                Longitude = item.Longitude.Value,
                Latitude = item.Latitude.Value,
                Color = GetColorForStatus(item.Status)
            });
        }
        return markers;
    }

    /// <summary>
    /// 为标记点生成 WKT Point 几何。
    /// </summary>
    public string MarkerToWkt(WorkItemMarker marker)
    {
        return $"POINT ({marker.Longitude} {marker.Latitude})";
    }

    /// <summary>
    /// 查询指定范围内的工作项标记点。
    /// </summary>
    public IReadOnlyList<WorkItemMarker> FilterByExtent(
        IReadOnlyList<WorkItemMarker> markers,
        double minLon, double minLat, double maxLon, double maxLat)
    {
        return markers
            .Where(m =>
                m.Longitude >= minLon && m.Longitude <= maxLon &&
                m.Latitude >= minLat && m.Latitude <= maxLat)
            .ToList();
    }

    /// <summary>
    /// 查找距给定坐标最近的标记点。
    /// </summary>
    public WorkItemMarker? FindNearest(
        IReadOnlyList<WorkItemMarker> markers,
        double longitude, double latitude)
    {
        if (markers.Count == 0)
            return null;

        WorkItemMarker? nearest = null;
        var minDist = double.MaxValue;

        foreach (var marker in markers)
        {
            var dist = DistanceSquared(marker.Longitude, marker.Latitude, longitude, latitude);
            if (dist < minDist)
            {
                minDist = dist;
                nearest = marker;
            }
        }

        return nearest;
    }

    private static string GetColorForStatus(WorkItemStatus status) => status switch
    {
        WorkItemStatus.Inbox => "#FF6B6B",     // 红色 — 待处理
        WorkItemStatus.Active => "#FFA726",     // 橙色 — 进行中
        WorkItemStatus.Waiting => "#42A5F5",    // 蓝色 — 等待中
        WorkItemStatus.Done => "#66BB6A",       // 绿色 — 已完成
        WorkItemStatus.Archived => "#BDBDBD",   // 灰色 — 已归档
        _ => "#42A5F5"
    };

    private static double DistanceSquared(double x1, double y1, double x2, double y2)
    {
        var dx = x1 - x2;
        var dy = y1 - y2;
        return dx * dx + dy * dy;
    }
}

/// <summary>
/// 工作项地图标记点数据。
/// </summary>
public sealed class WorkItemMarker
{
    public Ulid WorkItemId { get; init; }
    public required string Title { get; init; }
    public WorkItemStatus Status { get; init; }
    public double Longitude { get; init; }
    public double Latitude { get; init; }
    public string Color { get; init; } = "#42A5F5";
}
```

- [ ] 2. Build and verify: `dotnet build src/Wif.Gis/Wif.Gis.csproj`

---

## Task 7: GeocodingService

**Files:**
- `src/Wif.Gis/Services/GeocodingService.cs` (NEW)

**Steps:**

- [ ] 1. Create `src/Wif.Gis/Services/GeocodingService.cs`:

```csharp
using System.Net.Http.Json;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Wif.Gis.Services;

/// <summary>
/// 地理编码服务 — 地址转坐标。
/// 主要使用 Nominatim（OpenStreetMap），备用 TianDiTu（天地图）。
/// </summary>
public sealed class GeocodingService : IDisposable
{
    private readonly HttpClient _httpClient;
    private readonly SemaphoreSlim _rateLimiter = new(1, 1);
    private DateTimeOffset _lastRequestTime = DateTimeOffset.MinValue;
    private string? _tiandituToken;

    // Nominatim policy: max 1 request per second
    private static readonly TimeSpan MinRequestInterval = TimeSpan.FromSeconds(1);

    public GeocodingService(HttpClient? httpClient = null)
    {
        _httpClient = httpClient ?? new HttpClient();
        _httpClient.DefaultRequestHeaders.UserAgent.ParseAdd(
            "WifApp/1.0 (WorkItemFlow Desktop)");
    }

    /// <summary>
    /// 设置天地图 API Token（可选，用于备用地理编码）。
    /// </summary>
    public void SetTiandituToken(string token)
    {
        _tiandituToken = token;
    }

    /// <summary>
    /// 地址转坐标。先尝试 Nominatim，失败则回退到天地图。
    /// </summary>
    public async Task<GeocodingResult?> GeocodeAsync(
        string address,
        CancellationToken cancellationToken = default)
    {
        // Try Nominatim first
        var result = await GeocodeNominatimAsync(address, cancellationToken);
        if (result is not null)
            return result;

        // Fallback to TianDiTu if token is available
        if (!string.IsNullOrEmpty(_tiandituToken))
        {
            result = await GeocodeTiandituAsync(address, cancellationToken);
            if (result is not null)
                return result;
        }

        return null;
    }

    private async Task<GeocodingResult?> GeocodeNominatimAsync(
        string address,
        CancellationToken cancellationToken)
    {
        await ThrottleAsync(cancellationToken);

        try
        {
            var encodedAddress = Uri.EscapeDataString(address);
            var url = $"https://nominatim.openstreetmap.org/search?q={encodedAddress}&format=jsonv2&limit=1";

            var response = await _httpClient.GetAsync(url, cancellationToken);
            response.EnsureSuccessStatusCode();

            var results = await response.Content.ReadFromJsonAsync<NominatimResult[]>(
                cancellationToken: cancellationToken);

            if (results is null || results.Length == 0)
                return null;

            var first = results[0];
            if (!double.TryParse(first.Lat, System.Globalization.NumberStyles.Float,
                    System.Globalization.CultureInfo.InvariantCulture, out var lat) ||
                !double.TryParse(first.Lon, System.Globalization.NumberStyles.Float,
                    System.Globalization.CultureInfo.InvariantCulture, out var lon))
            {
                return null;
            }

            return new GeocodingResult
            {
                Latitude = lat,
                Longitude = lon,
                DisplayName = first.DisplayName ?? address,
                Source = "Nominatim"
            };
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"[WARNING] Nominatim geocoding failed: {ex.Message}");
            return null;
        }
    }

    private async Task<GeocodingResult?> GeocodeTiandituAsync(
        string address,
        CancellationToken cancellationToken)
    {
        await ThrottleAsync(cancellationToken);

        try
        {
            var encodedAddress = Uri.EscapeDataString(address);
            var url = $"http://api.tianditu.gov.cn/geocoder?ds={{\"keyWord\":\"{encodedAddress}\"}}&tk={_tiandituToken}";

            var response = await _httpClient.GetAsync(url, cancellationToken);
            response.EnsureSuccessStatusCode();

            var json = await response.Content.ReadAsStringAsync(cancellationToken);
            using var doc = JsonDocument.Parse(json);
            var root = doc.RootElement;

            if (!root.TryGetProperty("status", out var statusProp) ||
                statusProp.GetString() != "0")
            {
                return null;
            }

            if (!root.TryGetProperty("location", out var location))
                return null;

            if (!location.TryGetProperty("lon", out var lonProp) ||
                !location.TryGetProperty("lat", out var latProp))
            {
                return null;
            }

            return new GeocodingResult
            {
                Latitude = latProp.GetDouble(),
                Longitude = lonProp.GetDouble(),
                DisplayName = address,
                Source = "TianDiTu"
            };
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"[WARNING] TianDiTu geocoding failed: {ex.Message}");
            return null;
        }
    }

    private async Task ThrottleAsync(CancellationToken cancellationToken)
    {
        await _rateLimiter.WaitAsync(cancellationToken);
        try
        {
            var elapsed = DateTimeOffset.UtcNow - _lastRequestTime;
            if (elapsed < MinRequestInterval)
            {
                var delay = MinRequestInterval - elapsed;
                await Task.Delay(delay, cancellationToken);
            }
            _lastRequestTime = DateTimeOffset.UtcNow;
        }
        finally
        {
            _rateLimiter.Release();
        }
    }

    public void Dispose()
    {
        _rateLimiter.Dispose();
        _httpClient.Dispose();
    }
}

public sealed class GeocodingResult
{
    public double Latitude { get; init; }
    public double Longitude { get; init; }
    public required string DisplayName { get; init; }
    public required string Source { get; init; }
}

// Nominatim JSON response model
internal sealed class NominatimResult
{
    [JsonPropertyName("lat")]
    public string? Lat { get; set; }

    [JsonPropertyName("lon")]
    public string? Lon { get; set; }

    [JsonPropertyName("display_name")]
    public string? DisplayName { get; set; }
}
```

- [ ] 2. Build and verify: `dotnet build src/Wif.Gis/Wif.Gis.csproj`

> **Note:** The Wif.Gis project may need a reference check. If `System.Net.Http.Json` is not available, it is included in .NET 9 by default, so no extra NuGet package is needed.

---

## Task 8: ClusterRenderer

**Files:**
- `src/Wif.Gis/Rendering/ClusterRenderer.cs` (NEW)

**Steps:**

- [ ] 1. Create the `src/Wif.Gis/Rendering/` directory if it does not exist.

- [ ] 2. Create `src/Wif.Gis/Rendering/ClusterRenderer.cs`:

```csharp
namespace Wif.Gis.Rendering;

/// <summary>
/// 聚合渲染器 — 根据缩放级别将临近标记点聚合为簇。
/// 使用基于网格的空间聚合算法。
/// </summary>
public sealed class ClusterRenderer
{
    /// <summary>
    /// 聚合像素半径（屏幕空间）。同一网格单元内的点合并为一个簇。
    /// </summary>
    public double ClusterRadiusPx { get; set; } = 60.0;

    /// <summary>
    /// 当缩放级别大于此值时，不再聚合（显示个别标记）。
    /// </summary>
    public double MaxClusterZoom { get; set; } = 15.0;

    /// <summary>
    /// 将标记点按当前缩放级别聚合。
    /// </summary>
    /// <param name="markers">标记点列表（经纬度坐标）。</param>
    /// <param name="zoomLevel">当前地图缩放级别。</param>
    /// <returns>聚合后的结果列表。</returns>
    public IReadOnlyList<ClusterResult> Cluster(
        IReadOnlyList<MarkerPoint> markers,
        double zoomLevel)
    {
        if (markers.Count == 0)
            return [];

        // At high zoom levels, return individual markers (no clustering)
        if (zoomLevel > MaxClusterZoom)
        {
            return markers
                .Select(m => new ClusterResult
                {
                    CenterLongitude = m.Longitude,
                    CenterLatitude = m.Latitude,
                    Count = 1,
                    Markers = [m],
                    IsSingleMarker = true
                })
                .ToList();
        }

        // Grid-based clustering:
        // The grid cell size in degrees depends on zoom level.
        // At zoom 0, ~360 degrees visible. Each zoom halves the world.
        // Cell size = ClusterRadiusPx * degreesPerPixel
        var degreesPerPixel = 360.0 / (256.0 * Math.Pow(2, zoomLevel));
        var cellSize = ClusterRadiusPx * degreesPerPixel;

        if (cellSize <= 0)
            cellSize = 0.001;

        var grid = new Dictionary<(int gx, int gy), List<MarkerPoint>>();

        foreach (var marker in markers)
        {
            var gx = (int)Math.Floor(marker.Longitude / cellSize);
            var gy = (int)Math.Floor(marker.Latitude / cellSize);
            var key = (gx, gy);

            if (!grid.TryGetValue(key, out var list))
            {
                list = [];
                grid[key] = list;
            }
            list.Add(marker);
        }

        var results = new List<ClusterResult>(grid.Count);

        foreach (var (_, cellMarkers) in grid)
        {
            var centerLon = cellMarkers.Average(m => m.Longitude);
            var centerLat = cellMarkers.Average(m => m.Latitude);

            results.Add(new ClusterResult
            {
                CenterLongitude = centerLon,
                CenterLatitude = centerLat,
                Count = cellMarkers.Count,
                Markers = cellMarkers,
                IsSingleMarker = cellMarkers.Count == 1
            });
        }

        return results;
    }
}

/// <summary>
/// 输入标记点（经纬度 + 标识）。
/// </summary>
public sealed class MarkerPoint
{
    public required string Id { get; init; }
    public double Longitude { get; init; }
    public double Latitude { get; init; }
    public string? Label { get; init; }
    public string Color { get; init; } = "#42A5F5";
}

/// <summary>
/// 聚合结果 — 代表单个标记或多标记簇。
/// </summary>
public sealed class ClusterResult
{
    public double CenterLongitude { get; init; }
    public double CenterLatitude { get; init; }
    public int Count { get; init; }
    public IReadOnlyList<MarkerPoint> Markers { get; init; } = [];
    public bool IsSingleMarker { get; init; }

    /// <summary>
    /// 生成聚合气泡的显示文本。
    /// </summary>
    public string DisplayLabel => IsSingleMarker
        ? Markers[0].Label ?? string.Empty
        : Count.ToString();
}
```

- [ ] 3. Build and verify: `dotnet build src/Wif.Gis/Wif.Gis.csproj`

---

## Task 9: WorkItemMapPopup (Avalonia View)

**Files:**
- `src/Wif.App/Views/WorkItemMapPopup.axaml` (NEW)
- `src/Wif.App/Views/WorkItemMapPopup.axaml.cs` (NEW)

**Steps:**

- [ ] 1. Create `src/Wif.App/Views/WorkItemMapPopup.axaml`:

```xml
<UserControl xmlns="https://github.com/avaloniaui"
             xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml"
             x:Class="Wif.App.Views.WorkItemMapPopup"
             x:CompileBindings="False"
             Width="280">
  <Border Background="{DynamicResource SystemControlBackgroundAltHighBrush}"
          CornerRadius="8"
          BoxShadow="0 2 8 0 #40000000"
          Padding="12">
    <StackPanel Spacing="8">

      <!-- Title -->
      <TextBlock Text="{Binding Title}"
                 FontSize="14"
                 FontWeight="SemiBold"
                 TextWrapping="Wrap"
                 MaxLines="2" />

      <!-- Status badge + priority color -->
      <StackPanel Orientation="Horizontal" Spacing="8">
        <Border Background="{Binding StatusColor}"
                CornerRadius="4"
                Padding="8,2">
          <TextBlock Text="{Binding StatusText}"
                     Foreground="White"
                     FontSize="11" />
        </Border>
        <Border Background="{Binding PriorityColor}"
                Width="12" Height="12"
                CornerRadius="6"
                VerticalAlignment="Center" />
      </StackPanel>

      <!-- Coordinates -->
      <TextBlock Foreground="{DynamicResource SystemControlForegroundBaseMediumBrush}"
                 FontSize="11"
                 FontFamily="Consolas">
        <TextBlock.Text>
          <MultiBinding StringFormat="经度: {0:F6}  纬度: {1:F6}">
            <Binding Path="Longitude" />
            <Binding Path="Latitude" />
          </MultiBinding>
        </TextBlock.Text>
      </TextBlock>

      <Separator Margin="0,2" />

      <!-- Action buttons -->
      <StackPanel Orientation="Horizontal" Spacing="8">
        <Button Content="查看详情"
                Command="{Binding ViewDetailCommand}"
                Padding="10,4"
                CornerRadius="4"
                Background="#1A73E8"
                Foreground="White"
                FontSize="12" />
        <Button Content="导航"
                Command="{Binding NavigateCommand}"
                Padding="10,4"
                CornerRadius="4"
                FontSize="12" />
      </StackPanel>
    </StackPanel>
  </Border>
</UserControl>
```

- [ ] 2. Create `src/Wif.App/Views/WorkItemMapPopup.axaml.cs`:

```csharp
using Avalonia.Controls;

namespace Wif.App.Views;

public partial class WorkItemMapPopup : UserControl
{
    public WorkItemMapPopup()
    {
        InitializeComponent();
    }
}
```

- [ ] 3. Build and verify: `dotnet build src/Wif.App/Wif.App.csproj`

---

## Task 10: WorkItemDetailView — Location Section

**Files:**
- `src/Wif.App/Views/WorkItemDetailView.axaml` (MODIFY)

**Steps:**

- [ ] 1. Add a location section to the `WorkItemDetailView`. Insert it **before** the final `<!-- Status transition buttons -->` section (before `<Separator Margin="0,4" />` on line 136). Add the following XAML block:

```xml
      <!-- 位置信息 -->
      <TextBlock Text="位置信息" FontWeight="SemiBold" FontSize="16" Margin="0,8,0,0" />
      <Grid ColumnDefinitions="*,*" RowDefinitions="Auto,Auto,Auto,Auto" Margin="0,4,0,0">
        <!-- Longitude input -->
        <StackPanel Grid.Row="0" Grid.Column="0" Spacing="4" Margin="0,0,8,8">
          <TextBlock Text="经度" FontSize="12" Opacity="0.7" />
          <TextBox Text="{Binding LongitudeText}"
                   Watermark="例如: 116.397128"
                   FontFamily="Consolas" />
        </StackPanel>

        <!-- Latitude input -->
        <StackPanel Grid.Row="0" Grid.Column="1" Spacing="4" Margin="0,0,0,8">
          <TextBlock Text="纬度" FontSize="12" Opacity="0.7" />
          <TextBox Text="{Binding LatitudeText}"
                   Watermark="例如: 39.916527"
                   FontFamily="Consolas" />
        </StackPanel>

        <!-- Address geocoding -->
        <StackPanel Grid.Row="1" Grid.Column="0" Grid.ColumnSpan="2"
                    Orientation="Horizontal" Spacing="4" Margin="0,0,0,8">
          <TextBox Text="{Binding AddressInput}"
                   Watermark="输入地址进行地理编码..."
                   Width="280" />
          <Button Content="地理编码"
                  Command="{Binding GeocodeCommand}"
                  Padding="8,4" CornerRadius="4" />
        </StackPanel>

        <!-- Action buttons -->
        <StackPanel Grid.Row="2" Grid.Column="0" Grid.ColumnSpan="2"
                    Orientation="Horizontal" Spacing="8" Margin="0,0,0,8">
          <Button Content="保存位置"
                  Command="{Binding SaveLocationCommand}"
                  Padding="8,4" CornerRadius="4"
                  Background="#1A73E8" Foreground="White" />
          <Button Content="在地图上查看"
                  Command="{Binding NavigateToMapCommand}"
                  Padding="8,4" CornerRadius="4" />
          <Button Content="从地图选点"
                  Command="{Binding PickLocationFromMapCommand}"
                  Padding="8,4" CornerRadius="4" />
          <Button Content="清除位置"
                  Command="{Binding ClearLocationCommand}"
                  Padding="8,4" CornerRadius="4" />
        </StackPanel>

        <!-- Associated GIS feature -->
        <StackPanel Grid.Row="3" Grid.Column="0" Grid.ColumnSpan="2"
                    Orientation="Horizontal" Spacing="8"
                    IsVisible="{Binding HasGisFeature}">
          <TextBlock Text="关联要素:" FontSize="12" Opacity="0.7" VerticalAlignment="Center" />
          <TextBlock Text="{Binding GisFeatureName}" FontSize="12" VerticalAlignment="Center" />
        </StackPanel>
      </Grid>

      <Separator Margin="0,4" />
```

The insert point is right before the existing last `<Separator Margin="0,4" />` and `<!-- Status transition buttons -->` (lines 136-146 in the original file).

- [ ] 2. Build and verify: `dotnet build src/Wif.App/Wif.App.csproj`

---

## Task 11: WorkItemDetailViewModel — Location Properties & Commands

**Files:**
- `src/Wif.App/ViewModels/WorkItemDetailViewModel.cs` (MODIFY)

**Steps:**

- [ ] 1. Add a constructor parameter for `GeocodingService`. Update the constructor:

```csharp
    private readonly GeocodingService _geocodingService;

    public WorkItemDetailViewModel(
        WorkItemService workItemService,
        AnalysisService analysisService,
        ProposalService proposalService,
        ExportService exportService,
        GeocodingService geocodingService)
    {
        _workItemService = workItemService;
        _analysisService = analysisService;
        _proposalService = proposalService;
        _exportService = exportService;
        _geocodingService = geocodingService;
    }
```

- [ ] 2. Add a using directive at the top of the file:

```csharp
using Wif.Gis.Services;
```

- [ ] 3. Add the following observable properties after the existing `_isLoading` field:

```csharp
    [ObservableProperty]
    private string _longitudeText = string.Empty;

    [ObservableProperty]
    private string _latitudeText = string.Empty;

    [ObservableProperty]
    private string _addressInput = string.Empty;

    [ObservableProperty]
    private bool _hasGisFeature;

    [ObservableProperty]
    private string _gisFeatureName = string.Empty;
```

- [ ] 4. Update the `ReloadCurrentItemAsync` method to populate location fields. Add the following at the end of the method, before the proposals section:

```csharp
        // Populate location fields
        if (CurrentItem is not null)
        {
            LongitudeText = CurrentItem.Longitude?.ToString("F6",
                System.Globalization.CultureInfo.InvariantCulture) ?? string.Empty;
            LatitudeText = CurrentItem.Latitude?.ToString("F6",
                System.Globalization.CultureInfo.InvariantCulture) ?? string.Empty;
            HasGisFeature = CurrentItem.GisFeatureId is not null;
            GisFeatureName = CurrentItem.GisFeature?.Name ?? string.Empty;
        }
        else
        {
            LongitudeText = string.Empty;
            LatitudeText = string.Empty;
            HasGisFeature = false;
            GisFeatureName = string.Empty;
        }
```

- [ ] 5. Add the following relay commands:

```csharp
    [RelayCommand]
    private async Task SaveLocationAsync()
    {
        if (CurrentItem is null)
            return;

        try
        {
            double? longitude = null;
            double? latitude = null;

            if (!string.IsNullOrWhiteSpace(LongitudeText) &&
                double.TryParse(LongitudeText,
                    System.Globalization.NumberStyles.Float,
                    System.Globalization.CultureInfo.InvariantCulture, out var lon))
            {
                longitude = lon;
            }

            if (!string.IsNullOrWhiteSpace(LatitudeText) &&
                double.TryParse(LatitudeText,
                    System.Globalization.NumberStyles.Float,
                    System.Globalization.CultureInfo.InvariantCulture, out var lat))
            {
                latitude = lat;
            }

            await _workItemService.UpdateLocationAsync(
                CurrentItem.Id, longitude, latitude, CancellationToken.None);
            await ReloadCurrentItemAsync(CurrentItem.Id);
        }
        catch (Exception ex)
        {
            ErrorMessage = $"保存位置失败: {ex.Message}";
            System.Diagnostics.Debug.WriteLine($"[ERROR] SaveLocationAsync failed: {ex}");
        }
    }

    [RelayCommand]
    private async Task GeocodeAsync()
    {
        if (string.IsNullOrWhiteSpace(AddressInput))
            return;

        try
        {
            ErrorMessage = null;
            var result = await _geocodingService.GeocodeAsync(
                AddressInput.Trim(), CancellationToken.None);

            if (result is null)
            {
                ErrorMessage = "地理编码未找到结果";
                return;
            }

            LongitudeText = result.Longitude.ToString("F6",
                System.Globalization.CultureInfo.InvariantCulture);
            LatitudeText = result.Latitude.ToString("F6",
                System.Globalization.CultureInfo.InvariantCulture);
        }
        catch (Exception ex)
        {
            ErrorMessage = $"地理编码失败: {ex.Message}";
            System.Diagnostics.Debug.WriteLine($"[ERROR] GeocodeAsync failed: {ex}");
        }
    }

    [RelayCommand]
    private void NavigateToMap()
    {
        // TODO: Navigate to GIS view centered on this work item's location.
        // This requires a navigation service or messenger; for now, log intent.
        if (CurrentItem?.Longitude is not null && CurrentItem?.Latitude is not null)
        {
            System.Diagnostics.Debug.WriteLine(
                $"[NAV] Navigate to map: {CurrentItem.Longitude}, {CurrentItem.Latitude}");
        }
    }

    [RelayCommand]
    private void PickLocationFromMap()
    {
        // TODO: Switch to GIS view in "pick location" mode.
        // Requires inter-view messaging; for now, log intent.
        System.Diagnostics.Debug.WriteLine(
            $"[NAV] Pick location from map for WorkItem {CurrentItem?.Id}");
    }

    [RelayCommand]
    private async Task ClearLocationAsync()
    {
        if (CurrentItem is null)
            return;

        try
        {
            await _workItemService.UpdateLocationAsync(
                CurrentItem.Id, null, null, CancellationToken.None);
            await ReloadCurrentItemAsync(CurrentItem.Id);
        }
        catch (Exception ex)
        {
            ErrorMessage = $"清除位置失败: {ex.Message}";
            System.Diagnostics.Debug.WriteLine($"[ERROR] ClearLocationAsync failed: {ex}");
        }
    }
```

- [ ] 6. Build and verify: `dotnet build src/Wif.App/Wif.App.csproj`

> **Note:** The `UpdateLocationAsync` method must be added to `WorkItemService` by the assembler or a separate task. If it does not exist yet, the ViewModel code is ready for it. The method signature should be:
> ```csharp
> Task UpdateLocationAsync(Ulid id, double? longitude, double? latitude, CancellationToken ct);
> ```

---

## Dependencies & Execution Order

```
Task 2 (WorkItem entity)  ──┐
Task 3 (GisFeature entity)──┼── Task 4 (WorkItemConfig) ──┐
                             └── Task 5 (GisFeatureConfig)─┤
                                                           ├── Task 1 (Migration)
Task 6 (WorkItemMapService) ─── no entity deps             │
Task 7 (GeocodingService) ──── no entity deps              │
Task 8 (ClusterRenderer) ───── no entity deps              │
                                                           │
Task 9  (MapPopup XAML) ────── no entity deps              │
Task 10 (DetailView XAML) ──── needs Task 11 bindings      │
Task 11 (DetailViewModel) ──── needs Task 2, Task 7        │
```

**Recommended parallel batches:**

1. **Batch A** (parallel): Tasks 2, 3, 6, 7, 8, 9
2. **Batch B** (parallel, after Batch A): Tasks 4, 5, 11
3. **Batch C** (after Batch B): Tasks 1, 10
4. **Final**: Full solution build `dotnet build Wif.sln`

---

## Verification Checklist

After all tasks are complete:

- [ ] `dotnet build Wif.sln` succeeds with no errors
- [ ] `dotnet test Wif.sln` passes (existing tests must not break)
- [ ] WorkItem entity has `Longitude`, `Latitude`, `GisFeatureId` properties
- [ ] GisFeature entity has `WorkItemId` property and `WorkItem` navigation
- [ ] WorkItemConfiguration maps new columns with correct converters
- [ ] GisFeatureConfiguration maps `WorkItemId` with nullable Ulid converter
- [ ] Migration creates columns and indexes idempotently
- [ ] WorkItemMapService builds markers from WorkItems with coordinates
- [ ] GeocodingService calls Nominatim with rate limiting and TianDiTu fallback
- [ ] ClusterRenderer groups markers by grid cells at low zoom
- [ ] WorkItemMapPopup displays title, status, coordinates, action buttons
- [ ] WorkItemDetailView shows location section with lat/lon inputs
- [ ] WorkItemDetailViewModel exposes location properties and geocode command
- [ ] All UI strings are in Chinese

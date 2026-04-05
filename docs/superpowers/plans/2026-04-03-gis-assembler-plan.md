# gis-assembler Implementation Plan

> **For agentic workers:** Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Merge all parallel agent outputs (gis-map, gis-editing, gis-analysis, gis-integration) into the shared files that no other agent was allowed to touch, then verify the entire solution compiles and runs.

**Architecture:** This agent is the last to execute (Wave 3). It merges domain entity changes, updates EF configurations, registers new DI services, rewrites `GisView.axaml` with the Mapsui MapControl + editing toolbar + attribute table, and rewrites `GisViewModel.cs` to orchestrate all subsystems.

**Tech Stack:** Avalonia 11.2.1, Mapsui.Avalonia, CommunityToolkit.Mvvm 8.4.2, NetTopologySuite 2.6.0, ProjNET 2.0.0

**PREREQUISITE:** All 4 Wave-1 agents (gis-map, gis-editing, gis-analysis, gis-integration) must have completed their tasks before this plan starts. Verify by checking that all their output files exist.

---

## Pre-flight Check

Before starting any task, verify that the Wave 1 agents completed their work:

```bash
# gis-map outputs
test -f src/Wif.Gis/Services/MapService.cs && echo "OK" || echo "MISSING"
test -f src/Wif.Gis/Services/TileSourceFactory.cs && echo "OK" || echo "MISSING"
test -f src/Wif.Gis/Rendering/FeatureStyleProvider.cs && echo "OK" || echo "MISSING"
test -f src/Wif.Domain/Enums/GisLayerSourceType.cs && echo "OK" || echo "MISSING"

# gis-editing outputs
test -f src/Wif.Gis/Editing/IMapTool.cs && echo "OK" || echo "MISSING"
test -f src/Wif.Gis/Editing/ToolManager.cs && echo "OK" || echo "MISSING"
test -f src/Wif.Gis/Editing/DrawingTool.cs && echo "OK" || echo "MISSING"
test -f src/Wif.Gis/Editing/EditingTool.cs && echo "OK" || echo "MISSING"
test -f src/Wif.Gis/Editing/SelectionTool.cs && echo "OK" || echo "MISSING"
test -f src/Wif.App/ViewModels/AttributeTableViewModel.cs && echo "OK" || echo "MISSING"
test -f src/Wif.App/Views/AttributeTablePanel.axaml && echo "OK" || echo "MISSING"

# gis-analysis outputs
test -f src/Wif.Gis/Analysis/SpatialAnalysisService.cs && echo "OK" || echo "MISSING"
test -f src/Wif.Gis/Analysis/MeasurementService.cs && echo "OK" || echo "MISSING"
test -f src/Wif.Gis/Analysis/AnalysisResult.cs && echo "OK" || echo "MISSING"
test -f src/Wif.Gis/Projection/CrsManager.cs && echo "OK" || echo "MISSING"
test -f src/Wif.Gis/Projection/CoordinateTransformer.cs && echo "OK" || echo "MISSING"
test -f src/Wif.Gis/Editing/MeasureTool.cs && echo "OK" || echo "MISSING"
test -f src/Wif.App/Views/AnalysisDialog.axaml && echo "OK" || echo "MISSING"
test -f src/Wif.App/ViewModels/AnalysisDialogViewModel.cs && echo "OK" || echo "MISSING"

# gis-integration outputs
test -f src/Wif.Gis/Services/WorkItemMapService.cs && echo "OK" || echo "MISSING"
test -f src/Wif.Gis/Services/GeocodingService.cs && echo "OK" || echo "MISSING"
test -f src/Wif.Gis/Rendering/ClusterRenderer.cs && echo "OK" || echo "MISSING"
test -f src/Wif.Data/Migrations/WorkItemGeoMigration.cs && echo "OK" || echo "MISSING"
test -f src/Wif.App/Views/WorkItemMapPopup.axaml && echo "OK" || echo "MISSING"
```

If any file is MISSING, do NOT proceed. Report the missing files.

---

### Task 1: Merge GisLayer.cs domain entity

**Files:** Modify `src/Wif.Domain/Entities/GisLayer.cs`

The gis-map agent adds `SourceType` (GisLayerSourceType enum), the gis-analysis agent adds `CrsCode` (string). Both need to be present.

- [ ] **Step 1: Read the current file and verify which properties are already present**

```bash
grep -E "SourceType|CrsCode" src/Wif.Domain/Entities/GisLayer.cs
```

- [ ] **Step 2: Ensure the final file contains both new properties**

The complete file should be:

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
    public string CrsCode { get; set; } = "EPSG:4326";
    public DateTimeOffset CreatedAt { get; init; }
    public DateTimeOffset UpdatedAt { get; set; }

    // Navigation
    public List<GisFeature> Features { get; init; } = [];
}
```

- [ ] **Step 3: Verify build**

```bash
dotnet build src/Wif.Domain/Wif.Domain.csproj
```

---

### Task 2: Merge GisFeature.cs domain entity

**Files:** Modify `src/Wif.Domain/Entities/GisFeature.cs`

The gis-integration agent adds `WorkItemId` (Ulid?) and `WorkItem` navigation property.

- [ ] **Step 1: Check if WorkItemId is already present**

```bash
grep "WorkItemId" src/Wif.Domain/Entities/GisFeature.cs
```

- [ ] **Step 2: Ensure the final file contains the new properties**

The complete file should be:

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

- [ ] **Step 3: Verify build**

```bash
dotnet build src/Wif.Domain/Wif.Domain.csproj
```

---

### Task 3: Merge WorkItem.cs domain entity

**Files:** Modify `src/Wif.Domain/Entities/WorkItem.cs`

The gis-integration agent adds `Longitude` (double?), `Latitude` (double?), `GisFeatureId` (Ulid?), and `GisFeature` navigation.

- [ ] **Step 1: Check if location fields are already present**

```bash
grep -E "Longitude|Latitude|GisFeatureId" src/Wif.Domain/Entities/WorkItem.cs
```

- [ ] **Step 2: Ensure the final file contains the new properties**

The complete file should be:

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
    public double? Longitude { get; set; }
    public double? Latitude { get; set; }
    public Ulid? GisFeatureId { get; set; }
    public DateTimeOffset CreatedAt { get; init; }
    public DateTimeOffset UpdatedAt { get; set; }

    // Navigation properties
    public List<Attachment> Attachments { get; init; } = [];
    public List<WorkEvent> Events { get; init; } = [];
    public List<Proposal> Proposals { get; init; } = [];
    public WorkItem? Parent { get; set; }
    public List<WorkItem> Children { get; init; } = [];
    public GisFeature? GisFeature { get; set; }
}
```

- [ ] **Step 3: Verify build**

```bash
dotnet build src/Wif.Domain/Wif.Domain.csproj
```

---

### Task 4: Merge Wif.Gis.csproj

**Files:** Modify `src/Wif.Gis/Wif.Gis.csproj`

gis-map adds Mapsui.Avalonia + Mapsui.Nts. gis-analysis adds ProjNET. Ensure all are present.

- [ ] **Step 1: Read and verify current state**

```bash
grep "PackageReference" src/Wif.Gis/Wif.Gis.csproj
```

- [ ] **Step 2: Ensure the final csproj has all packages**

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
    <PackageReference Include="ProjNET" Version="2.0.0" />
  </ItemGroup>
</Project>
```

- [ ] **Step 3: Restore and verify build**

```bash
dotnet restore src/Wif.Gis/Wif.Gis.csproj
dotnet build src/Wif.Gis/Wif.Gis.csproj
```

---

### Task 5: Update GisLayerConfiguration.cs

**Files:** Modify `src/Wif.Data/Configurations/GisLayerConfiguration.cs`

Add mappings for `SourceType` and `CrsCode`.

- [ ] **Step 1: Add SourceType and CrsCode mappings**

Add before the `CreatedAt` mapping:

```csharp
builder.Property(l => l.SourceType)
    .HasConversion<string>()
    .HasMaxLength(20);

builder.Property(l => l.CrsCode)
    .HasMaxLength(50);
```

The full file should be:

```csharp
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Metadata.Builders;
using Wif.Domain.Entities;

namespace Wif.Data.Configurations;

public class GisLayerConfiguration : IEntityTypeConfiguration<GisLayer>
{
    public void Configure(EntityTypeBuilder<GisLayer> builder)
    {
        builder.HasKey(l => l.Id);

        builder.Property(l => l.Id)
            .HasConversion(v => v.ToString(), v => Ulid.Parse(v))
            .HasMaxLength(26);

        builder.Property(l => l.Name)
            .IsRequired()
            .HasMaxLength(200);

        builder.Property(l => l.Description)
            .HasMaxLength(1000);

        builder.Property(l => l.SourceType)
            .HasConversion<string>()
            .HasMaxLength(20);

        builder.Property(l => l.FillColor)
            .HasMaxLength(20);

        builder.Property(l => l.StrokeColor)
            .HasMaxLength(20);

        builder.Property(l => l.CrsCode)
            .HasMaxLength(50);

        builder.Property(l => l.CreatedAt)
            .HasConversion(v => v.ToUnixTimeMilliseconds(), v => DateTimeOffset.FromUnixTimeMilliseconds(v));

        builder.Property(l => l.UpdatedAt)
            .HasConversion(v => v.ToUnixTimeMilliseconds(), v => DateTimeOffset.FromUnixTimeMilliseconds(v));

        builder.HasMany(l => l.Features)
            .WithOne(f => f.Layer)
            .HasForeignKey(f => f.LayerId)
            .OnDelete(DeleteBehavior.Cascade);
    }
}
```

- [ ] **Step 2: Verify build**

```bash
dotnet build src/Wif.Data/Wif.Data.csproj
```

---

### Task 6: Update GisFeatureConfiguration.cs

**Files:** Modify `src/Wif.Data/Configurations/GisFeatureConfiguration.cs`

Add `WorkItemId` mapping and navigation to `WorkItem`.

- [ ] **Step 1: Add WorkItemId mapping and relationship**

The full file should be:

```csharp
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Metadata.Builders;
using Wif.Domain.Entities;

namespace Wif.Data.Configurations;

public class GisFeatureConfiguration : IEntityTypeConfiguration<GisFeature>
{
    public void Configure(EntityTypeBuilder<GisFeature> builder)
    {
        builder.HasKey(f => f.Id);

        builder.Property(f => f.Id)
            .HasConversion(v => v.ToString(), v => Ulid.Parse(v))
            .HasMaxLength(26);

        builder.Property(f => f.LayerId)
            .HasConversion(v => v.ToString(), v => Ulid.Parse(v))
            .HasMaxLength(26);

        builder.Property(f => f.GeometryWkt)
            .IsRequired();

        builder.Property(f => f.GeometryType)
            .HasMaxLength(50);

        builder.Property(f => f.Name)
            .HasMaxLength(200);

        builder.Property(f => f.WorkItemId)
            .HasConversion(
                v => v.HasValue ? v.Value.ToString() : null,
                v => v != null ? Ulid.Parse(v) : (Ulid?)null)
            .HasMaxLength(26);

        builder.Property(f => f.CreatedAt)
            .HasConversion(v => v.ToUnixTimeMilliseconds(), v => DateTimeOffset.FromUnixTimeMilliseconds(v));

        builder.Property(f => f.UpdatedAt)
            .HasConversion(v => v.ToUnixTimeMilliseconds(), v => DateTimeOffset.FromUnixTimeMilliseconds(v));

        builder.HasIndex(f => f.LayerId);

        builder.HasIndex(f => f.WorkItemId);

        builder.HasOne(f => f.WorkItem)
            .WithMany()
            .HasForeignKey(f => f.WorkItemId)
            .OnDelete(DeleteBehavior.SetNull);
    }
}
```

- [ ] **Step 2: Verify build**

```bash
dotnet build src/Wif.Data/Wif.Data.csproj
```

---

### Task 7: Update WorkItemConfiguration.cs

**Files:** Modify `src/Wif.Data/Configurations/WorkItemConfiguration.cs`

Add `Longitude`, `Latitude`, `GisFeatureId` mappings and GisFeature navigation.

- [ ] **Step 1: Add location property mappings**

Add before the `builder.HasIndex(w => w.Status)` line:

```csharp
builder.Property(w => w.GisFeatureId)
    .HasConversion(
        v => v.HasValue ? v.Value.ToString() : null,
        v => v != null ? Ulid.Parse(v) : (Ulid?)null)
    .HasMaxLength(26);

builder.HasOne(w => w.GisFeature)
    .WithMany()
    .HasForeignKey(w => w.GisFeatureId)
    .OnDelete(DeleteBehavior.SetNull);
```

And add an index:

```csharp
builder.HasIndex(w => w.GisFeatureId);
```

- [ ] **Step 2: Verify build**

```bash
dotnet build src/Wif.Data/Wif.Data.csproj
```

---

### Task 8: Update GisMigration.cs for new columns

**Files:** Modify `src/Wif.Data/Migrations/GisMigration.cs`

Add idempotent ALTER TABLE statements for the new columns added by agents.

- [ ] **Step 1: Add new columns to the migration**

Append after the existing `CREATE INDEX` statement, still inside the `Apply` method:

```csharp
// New columns added by gis-map, gis-analysis, gis-integration agents
try { db.Database.ExecuteSqlRaw("ALTER TABLE GisLayers ADD COLUMN SourceType TEXT NOT NULL DEFAULT 'Vector'"); }
catch { /* already exists */ }

try { db.Database.ExecuteSqlRaw("ALTER TABLE GisLayers ADD COLUMN CrsCode TEXT NULL DEFAULT 'EPSG:4326'"); }
catch { /* already exists */ }

try { db.Database.ExecuteSqlRaw("ALTER TABLE GisFeatures ADD COLUMN WorkItemId TEXT NULL"); }
catch { /* already exists */ }

db.Database.ExecuteSqlRaw("CREATE INDEX IF NOT EXISTS IX_GisFeatures_WorkItemId ON GisFeatures (WorkItemId) WHERE WorkItemId IS NOT NULL");
```

- [ ] **Step 2: Register WorkItemGeoMigration in App.axaml.cs**

In `src/Wif.App/App.axaml.cs`, find the line:

```csharp
RunMigration(db, "Gis", GisMigration.Apply);
```

Add after it:

```csharp
RunMigration(db, "WorkItemGeo", WorkItemGeoMigration.Apply);
```

- [ ] **Step 3: Verify build**

```bash
dotnet build Wif.sln
```

---

### Task 9: Update DependencyInjection.cs

**Files:** Modify `src/Wif.Core/Services/DependencyInjection.cs`

Register all new services from all 4 agents.

- [ ] **Step 1: Add using directives**

Add these at the top of the file:

```csharp
using Wif.Gis.Editing;
using Wif.Gis.Rendering;
using Wif.Gis.Analysis;
using Wif.Gis.Projection;
```

- [ ] **Step 2: Replace the existing GIS registration block**

Find these lines:

```csharp
// GIS
services.AddSingleton<GeometryService>();
services.AddSingleton<GisFileService>();
services.AddSingleton<GisToolRegistry>();
```

Replace with:

```csharp
// GIS — core services
services.AddSingleton<GeometryService>();
services.AddSingleton<GisFileService>();
services.AddSingleton<GisToolRegistry>();

// GIS — map rendering (gis-map agent)
services.AddSingleton<FeatureStyleProvider>();
services.AddSingleton<TileSourceFactory>();
services.AddSingleton<MapService>();

// GIS — editing tools (gis-editing agent)
services.AddSingleton<ToolManager>();

// GIS — spatial analysis (gis-analysis agent)
services.AddSingleton<SpatialAnalysisService>();
services.AddSingleton<MeasurementService>();
services.AddSingleton<CrsManager>();
services.AddSingleton<CoordinateTransformer>();

// GIS — WorkItem integration (gis-integration agent)
services.AddSingleton<WorkItemMapService>();
services.AddSingleton<GeocodingService>();
services.AddSingleton<ClusterRenderer>();
```

- [ ] **Step 3: Add AttributeTableViewModel and AnalysisDialogViewModel to the ViewModel registrations section**

In `App.axaml.cs`, find the ViewModel registration block and add:

```csharp
sc.AddTransient<AttributeTableViewModel>();
sc.AddTransient<AnalysisDialogViewModel>();
```

after the existing `sc.AddTransient<GisViewModel>();` line.

- [ ] **Step 4: Verify build**

```bash
dotnet build Wif.sln
```

---

### Task 10: Rewrite GisView.axaml

**Files:** Modify `src/Wif.App/Views/GisView.axaml`

Replace the empty Canvas with Mapsui MapControl, add editing toolbar, basemap selector, and bottom attribute table panel.

- [ ] **Step 1: Rewrite the full GisView.axaml**

```xml
<UserControl xmlns="https://github.com/avaloniaui"
             xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml"
             xmlns:mapsui="clr-namespace:Mapsui.UI.Avalonia;assembly=Mapsui.UI.Avalonia"
             xmlns:vm="clr-namespace:Wif.App.ViewModels"
             xmlns:views="clr-namespace:Wif.App.Views"
             x:Class="Wif.App.Views.GisView"
             x:CompileBindings="False">

  <UserControl.Styles>
    <Style Selector="Border.panel">
      <Setter Property="Background" Value="{DynamicResource SystemControlBackgroundAltHighBrush}" />
      <Setter Property="CornerRadius" Value="8" />
      <Setter Property="Padding" Value="12" />
    </Style>
    <Style Selector="TextBlock.panel-title">
      <Setter Property="FontSize" Value="14" />
      <Setter Property="FontWeight" Value="SemiBold" />
      <Setter Property="Margin" Value="0,0,0,8" />
    </Style>
    <Style Selector="Button.tool-btn">
      <Setter Property="Padding" Value="8,6" />
      <Setter Property="Margin" Value="2" />
      <Setter Property="CornerRadius" Value="6" />
      <Setter Property="MinWidth" Value="32" />
      <Setter Property="HorizontalContentAlignment" Value="Center" />
    </Style>
    <Style Selector="Button.tool-btn-active">
      <Setter Property="Background" Value="#1A73E8" />
      <Setter Property="Foreground" Value="White" />
    </Style>
    <Style Selector="Button.primary-btn">
      <Setter Property="Background" Value="#1A73E8" />
      <Setter Property="Foreground" Value="White" />
      <Setter Property="CornerRadius" Value="6" />
      <Setter Property="Padding" Value="12,6" />
    </Style>
    <Style Selector="Border.layer-item">
      <Setter Property="Padding" Value="8,6" />
      <Setter Property="CornerRadius" Value="6" />
      <Setter Property="Margin" Value="0,2" />
      <Setter Property="Cursor" Value="Hand" />
    </Style>
    <Style Selector="Border.layer-item:pointerover">
      <Setter Property="Background" Value="{DynamicResource SystemControlHighlightListLowBrush}" />
    </Style>
  </UserControl.Styles>

  <Grid RowDefinitions="*,Auto,Auto">

    <!-- Main content: 3-column layout -->
    <Grid Grid.Row="0" ColumnDefinitions="240,*,260">

      <!-- LEFT: Layer Panel -->
      <Border Classes="panel" Grid.Column="0" Margin="8">
        <DockPanel>
          <TextBlock Classes="panel-title" DockPanel.Dock="Top">图层管理</TextBlock>

          <!-- Layer toolbar -->
          <StackPanel DockPanel.Dock="Top" Orientation="Horizontal" Margin="0,0,0,8" Spacing="4">
            <Button Classes="tool-btn" Command="{Binding AddLayerCommand}" ToolTip.Tip="新建图层">+</Button>
            <Button Classes="tool-btn" Command="{Binding RemoveLayerCommand}" ToolTip.Tip="删除图层">x</Button>
            <Button Classes="tool-btn" Command="{Binding MoveLayerUpCommand}" ToolTip.Tip="上移">^</Button>
            <Button Classes="tool-btn" Command="{Binding MoveLayerDownCommand}" ToolTip.Tip="下移">v</Button>
          </StackPanel>

          <!-- Layer list -->
          <ListBox ItemsSource="{Binding Layers}"
                   SelectedItem="{Binding SelectedLayer}"
                   Background="Transparent"
                   BorderThickness="0">
            <ListBox.ItemTemplate>
              <DataTemplate>
                <Border Classes="layer-item">
                  <Grid ColumnDefinitions="Auto,*,Auto">
                    <CheckBox Grid.Column="0"
                              IsChecked="{Binding IsVisible}"
                              Margin="0,0,8,0" />
                    <StackPanel Grid.Column="1" VerticalAlignment="Center">
                      <TextBlock Text="{Binding Name}" FontWeight="Medium" />
                      <TextBlock Text="{Binding GeometryType}"
                                 FontSize="11"
                                 Foreground="{DynamicResource SystemControlForegroundBaseMediumBrush}" />
                    </StackPanel>
                    <Border Grid.Column="2"
                            Width="16" Height="16" CornerRadius="3"
                            Background="{Binding FillColor}"
                            BorderBrush="{Binding StrokeColor}"
                            BorderThickness="2"
                            VerticalAlignment="Center" />
                  </Grid>
                </Border>
              </DataTemplate>
            </ListBox.ItemTemplate>
          </ListBox>
        </DockPanel>
      </Border>

      <!-- CENTER: Map Area -->
      <Grid Grid.Column="1" Margin="0,8">
        <!-- Mapsui MapControl -->
        <mapsui:MapControl Name="MapControl" />

        <!-- Editing toolbar overlay (top-left) -->
        <StackPanel HorizontalAlignment="Left" VerticalAlignment="Top"
                    Margin="12" Spacing="4">
          <Border Background="{DynamicResource SystemControlBackgroundAltHighBrush}"
                  CornerRadius="6" Padding="4" BoxShadow="0 1 4 0 #30000000">
            <StackPanel Spacing="2">
              <TextBlock Text="工具" FontSize="10" HorizontalAlignment="Center"
                         Foreground="{DynamicResource SystemControlForegroundBaseMediumBrush}"
                         Margin="0,0,0,2" />
              <Button Classes="tool-btn" Command="{Binding SetSelectToolCommand}" ToolTip.Tip="选择 (S)">S</Button>
              <Button Classes="tool-btn" Command="{Binding SetDrawPointCommand}" ToolTip.Tip="画点 (P)">P</Button>
              <Button Classes="tool-btn" Command="{Binding SetDrawLineCommand}" ToolTip.Tip="画线 (L)">L</Button>
              <Button Classes="tool-btn" Command="{Binding SetDrawPolygonCommand}" ToolTip.Tip="画面 (G)">G</Button>
              <Button Classes="tool-btn" Command="{Binding SetEditToolCommand}" ToolTip.Tip="编辑 (E)">E</Button>
              <Separator Margin="0,4" />
              <Button Classes="tool-btn" Command="{Binding SetMeasureDistanceCommand}" ToolTip.Tip="测距">D</Button>
              <Button Classes="tool-btn" Command="{Binding SetMeasureAreaCommand}" ToolTip.Tip="测面">A</Button>
              <Separator Margin="0,4" />
              <Button Classes="tool-btn" Command="{Binding UndoCommand}" ToolTip.Tip="撤销">↩</Button>
              <Button Classes="tool-btn" Command="{Binding RedoCommand}" ToolTip.Tip="重做">↪</Button>
            </StackPanel>
          </Border>
        </StackPanel>

        <!-- Basemap selector overlay (top-right) -->
        <Border HorizontalAlignment="Right" VerticalAlignment="Top"
                Margin="12"
                Background="{DynamicResource SystemControlBackgroundAltHighBrush}"
                CornerRadius="6" Padding="8" BoxShadow="0 1 4 0 #30000000">
          <StackPanel Spacing="4">
            <TextBlock Text="底图" FontSize="10"
                       Foreground="{DynamicResource SystemControlForegroundBaseMediumBrush}" />
            <ComboBox SelectedIndex="{Binding SelectedBasemapIndex}"
                      MinWidth="120">
              <ComboBoxItem>OpenStreetMap</ComboBoxItem>
              <ComboBoxItem>天地图</ComboBoxItem>
              <ComboBoxItem>无底图</ComboBoxItem>
            </ComboBox>
          </StackPanel>
        </Border>

        <!-- Zoom controls overlay (bottom-right) -->
        <StackPanel HorizontalAlignment="Right" VerticalAlignment="Bottom"
                    Margin="12" Spacing="4">
          <Border Background="{DynamicResource SystemControlBackgroundAltHighBrush}"
                  CornerRadius="6" Padding="4" BoxShadow="0 1 4 0 #30000000">
            <StackPanel Spacing="2">
              <Button Classes="tool-btn" Command="{Binding ZoomInCommand}" ToolTip.Tip="放大">+</Button>
              <Button Classes="tool-btn" Command="{Binding ZoomOutCommand}" ToolTip.Tip="缩小">-</Button>
              <Button Classes="tool-btn" Command="{Binding ZoomToAllCommand}" ToolTip.Tip="全图">[ ]</Button>
            </StackPanel>
          </Border>
        </StackPanel>

        <!-- Center message when no layers -->
        <TextBlock Text="在左侧面板添加图层或导入数据文件开始使用"
                   HorizontalAlignment="Center"
                   VerticalAlignment="Center"
                   Foreground="{DynamicResource SystemControlForegroundBaseMediumBrush}"
                   FontSize="14"
                   IsVisible="{Binding !Layers.Count}" />
      </Grid>

      <!-- RIGHT: Tools Panel -->
      <Border Classes="panel" Grid.Column="2" Margin="8">
        <ScrollViewer>
          <StackPanel Spacing="16">

            <!-- Import/Export -->
            <StackPanel Spacing="4">
              <TextBlock Classes="panel-title">数据导入导出</TextBlock>
              <Button Classes="primary-btn" Command="{Binding ImportGeoJsonCommand}"
                      HorizontalAlignment="Stretch">导入 GeoJSON</Button>
              <Button Classes="primary-btn" Command="{Binding ImportShapefileCommand}"
                      HorizontalAlignment="Stretch">导入 Shapefile</Button>
              <Button Command="{Binding ExportGeoJsonCommand}"
                      HorizontalAlignment="Stretch" Padding="12,6" CornerRadius="6">
                导出 GeoJSON
              </Button>
            </StackPanel>

            <Separator />

            <!-- Spatial Analysis -->
            <StackPanel Spacing="4">
              <TextBlock Classes="panel-title">空间分析</TextBlock>
              <Button Command="{Binding OpenAnalysisDialogCommand}"
                      HorizontalAlignment="Stretch" Padding="12,6" CornerRadius="6">
                空间分析...
              </Button>
              <Button Command="{Binding CalculateAreaCommand}"
                      HorizontalAlignment="Stretch" Padding="12,6" CornerRadius="6">
                面积计算
              </Button>
              <Button Command="{Binding CalculateDistanceCommand}"
                      HorizontalAlignment="Stretch" Padding="12,6" CornerRadius="6">
                距离计算
              </Button>
            </StackPanel>

            <Separator />

            <!-- Analysis Results -->
            <StackPanel Spacing="4">
              <TextBlock Classes="panel-title">分析结果</TextBlock>
              <Border Background="{DynamicResource SystemControlBackgroundBaseLowBrush}"
                      CornerRadius="6" Padding="8" MinHeight="80">
                <TextBlock Text="{Binding AnalysisResult}"
                           TextWrapping="Wrap"
                           Foreground="{DynamicResource SystemControlForegroundBaseHighBrush}" />
              </Border>
            </StackPanel>

            <Separator />

            <!-- Attribute Table Toggle -->
            <Button Command="{Binding ToggleAttributeTableCommand}"
                    HorizontalAlignment="Stretch" Padding="12,6" CornerRadius="6">
              属性表
            </Button>
          </StackPanel>
        </ScrollViewer>
      </Border>
    </Grid>

    <!-- BOTTOM: Attribute Table Panel (collapsible) -->
    <views:AttributeTablePanel Grid.Row="1"
                                DataContext="{Binding AttributeTable}"
                                MaxHeight="250" />

    <!-- BOTTOM: Status Bar -->
    <Border Grid.Row="2"
            Background="{DynamicResource SystemControlBackgroundChromeMediumBrush}"
            Padding="12,6">
      <Grid ColumnDefinitions="*,Auto,Auto">
        <TextBlock Grid.Column="0"
                   Text="{Binding StatusMessage}"
                   VerticalAlignment="Center"
                   Foreground="{DynamicResource SystemControlForegroundBaseMediumBrush}" />
        <TextBlock Grid.Column="1"
                   Text="{Binding ActiveToolName, StringFormat='工具: {0}'}"
                   VerticalAlignment="Center"
                   Foreground="{DynamicResource SystemControlForegroundBaseMediumBrush}"
                   Margin="0,0,16,0" />
        <TextBlock Grid.Column="2"
                   Text="{Binding CursorPosition, StringFormat='坐标: {0}'}"
                   VerticalAlignment="Center"
                   Foreground="{DynamicResource SystemControlForegroundBaseMediumBrush}"
                   FontFamily="Consolas" />
      </Grid>
    </Border>
  </Grid>
</UserControl>
```

- [ ] **Step 2: Verify build**

```bash
dotnet build src/Wif.App/Wif.App.csproj
```

---

### Task 11: Rewrite GisView.axaml.cs

**Files:** Modify `src/Wif.App/Views/GisView.axaml.cs`

Wire MapControl to ViewModel's MapService, forward pointer events to ToolManager.

- [ ] **Step 1: Rewrite GisView.axaml.cs**

```csharp
using Avalonia.Controls;
using Avalonia.Input;
using Mapsui.Projections;
using Microsoft.Extensions.DependencyInjection;
using Wif.App.ViewModels;

namespace Wif.App.Views;

public partial class GisView : UserControl
{
    public GisView()
    {
        InitializeComponent();
        DataContext = App.Services.GetRequiredService<GisViewModel>();

        Loaded += async (_, _) =>
        {
            try
            {
                if (DataContext is GisViewModel vm)
                {
                    // Bind MapService.Map to the MapControl
                    MapControl.Map = vm.MapInstance;

                    // Forward pointer events to ViewModel for tool dispatching
                    MapControl.PointerPressed += (s, e) =>
                    {
                        var screenPoint = e.GetPosition(MapControl);
                        var worldPoint = MapControl.Map.Navigator.Viewport.ScreenToWorld(
                            screenPoint.X, screenPoint.Y);
                        var (lon, lat) = SphericalMercator.ToLonLat(worldPoint.X, worldPoint.Y);
                        vm.OnMapPointerPressed(lon, lat,
                            e.KeyModifiers.HasFlag(KeyModifiers.Shift),
                            e.ClickCount >= 2,
                            screenPoint.X, screenPoint.Y);
                    };

                    MapControl.PointerMoved += (s, e) =>
                    {
                        var screenPoint = e.GetPosition(MapControl);
                        var worldPoint = MapControl.Map.Navigator.Viewport.ScreenToWorld(
                            screenPoint.X, screenPoint.Y);
                        var (lon, lat) = SphericalMercator.ToLonLat(worldPoint.X, worldPoint.Y);
                        vm.OnMapPointerMoved(lon, lat, screenPoint.X, screenPoint.Y);
                    };

                    MapControl.PointerReleased += (s, e) =>
                    {
                        var screenPoint = e.GetPosition(MapControl);
                        var worldPoint = MapControl.Map.Navigator.Viewport.ScreenToWorld(
                            screenPoint.X, screenPoint.Y);
                        var (lon, lat) = SphericalMercator.ToLonLat(worldPoint.X, worldPoint.Y);
                        vm.OnMapPointerReleased(lon, lat, screenPoint.X, screenPoint.Y);
                    };

                    // Keyboard events for tool shortcuts
                    KeyDown += (s, e) =>
                    {
                        vm.OnMapKeyDown(e.Key.ToString());
                    };

                    await vm.LoadCommand.ExecuteAsync(null);
                }
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"[ERROR] GisView initialization failed: {ex}");
            }
        };
    }
}
```

- [ ] **Step 2: Verify build**

```bash
dotnet build src/Wif.App/Wif.App.csproj
```

---

### Task 12: Rewrite GisViewModel.cs

**Files:** Modify `src/Wif.App/ViewModels/GisViewModel.cs`

This is the largest task — orchestrate MapService, ToolManager, analysis, and WorkItem integration.

- [ ] **Step 1: Rewrite the full GisViewModel.cs**

```csharp
using System.Collections.ObjectModel;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Platform.Storage;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Mapsui;
using Wif.Domain.Entities;
using Wif.Domain.Repositories;
using Wif.Gis.Editing;
using Wif.Gis.Services;

namespace Wif.App.ViewModels;

public partial class GisViewModel : ObservableObject
{
    private readonly IGisLayerRepository _repository;
    private readonly GisFileService _fileService;
    private readonly GeometryService _geometryService;
    private readonly MapService _mapService;
    private readonly ToolManager _toolManager;

    public GisViewModel(
        IGisLayerRepository repository,
        GisFileService fileService,
        GeometryService geometryService,
        MapService mapService,
        ToolManager toolManager)
    {
        _repository = repository;
        _fileService = fileService;
        _geometryService = geometryService;
        _mapService = mapService;
        _toolManager = toolManager;

        // Subscribe to tool change events
        _toolManager.ActiveToolChanged += tool =>
            ActiveToolName = tool?.Name ?? "无";
    }

    // ─── Map ──────────────────────────────────────

    /// <summary>Mapsui Map 实例，由 GisView.axaml.cs 绑定到 MapControl.Map</summary>
    public Map MapInstance => _mapService.Map;

    // ─── Layer Management ─────────────────────────

    public ObservableCollection<GisLayerItem> Layers { get; } = [];

    [ObservableProperty]
    private GisLayerItem? _selectedLayer;

    [ObservableProperty]
    private bool _isLoading;

    [ObservableProperty]
    private string _statusMessage = string.Empty;

    [ObservableProperty]
    private string _cursorPosition = "0.000000, 0.000000";

    [ObservableProperty]
    private string _activeToolName = "无";

    [ObservableProperty]
    private int _selectedBasemapIndex;

    // ─── Analysis Results ─────────────────────────

    [ObservableProperty]
    private string _analysisResult = string.Empty;

    // ─── Attribute Table ──────────────────────────

    [ObservableProperty]
    private AttributeTableViewModel? _attributeTable;

    // ─── Load ─────────────────────────────────────

    [RelayCommand]
    private async Task LoadAsync()
    {
        IsLoading = true;
        StatusMessage = "正在加载图层...";
        try
        {
            var layers = await _repository.ListLayersAsync();
            Layers.Clear();
            foreach (var layer in layers)
            {
                Layers.Add(new GisLayerItem
                {
                    Id = layer.Id,
                    Name = layer.Name,
                    IsVisible = layer.IsVisible,
                    FillColor = layer.FillColor,
                    StrokeColor = layer.StrokeColor,
                    FeatureCount = layer.Features.Count,
                    GeometryType = layer.Features.Count > 0
                        ? layer.Features[0].GeometryType
                        : "空"
                });
            }

            // Load layers onto the map
            _mapService.LoadLayers(layers);

            StatusMessage = $"已加载 {Layers.Count} 个图层";
        }
        catch (Exception ex)
        {
            StatusMessage = $"加载失败: {ex.Message}";
            System.Diagnostics.Debug.WriteLine($"[ERROR] GIS LoadAsync failed: {ex}");
        }
        finally
        {
            IsLoading = false;
        }
    }

    // ─── Layer CRUD ───────────────────────────────

    [RelayCommand]
    private async Task AddLayerAsync()
    {
        try
        {
            var now = DateTimeOffset.UtcNow;
            var entity = new GisLayer
            {
                Id = Ulid.NewUlid(),
                Name = $"新图层 {Layers.Count + 1}",
                DisplayOrder = Layers.Count,
                CreatedAt = now,
                UpdatedAt = now
            };
            await _repository.CreateLayerAsync(entity);

            var item = new GisLayerItem
            {
                Id = entity.Id,
                Name = entity.Name,
                IsVisible = true,
                FeatureCount = 0,
                GeometryType = "空"
            };
            Layers.Add(item);
            SelectedLayer = item;

            _mapService.AddOrUpdateLayer(entity);
            StatusMessage = $"已创建图层: {entity.Name}";
        }
        catch (Exception ex)
        {
            StatusMessage = $"创建图层失败: {ex.Message}";
        }
    }

    [RelayCommand]
    private async Task RemoveLayerAsync()
    {
        if (SelectedLayer is null) return;
        try
        {
            var name = SelectedLayer.Name;
            var id = SelectedLayer.Id;
            await _repository.DeleteLayerAsync(id);
            Layers.Remove(SelectedLayer);
            SelectedLayer = Layers.FirstOrDefault();

            _mapService.RemoveLayer(id);
            StatusMessage = $"已删除图层: {name}";
        }
        catch (Exception ex)
        {
            StatusMessage = $"删除图层失败: {ex.Message}";
        }
    }

    [RelayCommand]
    private void ToggleLayerVisibility(GisLayerItem? layer)
    {
        if (layer is null) return;
        layer.IsVisible = !layer.IsVisible;
        _mapService.SetLayerVisibility(layer.Id, layer.IsVisible);
    }

    [RelayCommand]
    private void MoveLayerUp()
    {
        if (SelectedLayer is null) return;
        var index = Layers.IndexOf(SelectedLayer);
        if (index > 0) Layers.Move(index, index - 1);
    }

    [RelayCommand]
    private void MoveLayerDown()
    {
        if (SelectedLayer is null) return;
        var index = Layers.IndexOf(SelectedLayer);
        if (index < Layers.Count - 1) Layers.Move(index, index + 1);
    }

    // ─── Basemap ──────────────────────────────────

    partial void OnSelectedBasemapIndexChanged(int value)
    {
        switch (value)
        {
            case 0:
                _mapService.SetOsmBasemap();
                StatusMessage = "底图: OpenStreetMap";
                break;
            case 1:
                // TODO: Read TianDiTu API key from app settings
                StatusMessage = "天地图需要配置 API Key";
                break;
            case 2:
                // No basemap — remove current
                StatusMessage = "底图: 无";
                break;
        }
    }

    // ─── Zoom ─────────────────────────────────────

    [RelayCommand]
    private void ZoomIn() => MapInstance.Navigator.ZoomIn();

    [RelayCommand]
    private void ZoomOut() => MapInstance.Navigator.ZoomOut();

    [RelayCommand]
    private void ZoomToAll() => _mapService.ZoomToAll();

    // ─── Tools ────────────────────────────────────

    [RelayCommand]
    private void SetSelectTool() => _toolManager.SetActiveTool("选择");

    [RelayCommand]
    private void SetDrawPoint()
    {
        _toolManager.SetActiveTool("绘制");
        if (_toolManager.ActiveTool is DrawingTool dt)
            dt.GeometryType = DrawGeometryType.Point;
    }

    [RelayCommand]
    private void SetDrawLine()
    {
        _toolManager.SetActiveTool("绘制");
        if (_toolManager.ActiveTool is DrawingTool dt)
            dt.GeometryType = DrawGeometryType.LineString;
    }

    [RelayCommand]
    private void SetDrawPolygon()
    {
        _toolManager.SetActiveTool("绘制");
        if (_toolManager.ActiveTool is DrawingTool dt)
            dt.GeometryType = DrawGeometryType.Polygon;
    }

    [RelayCommand]
    private void SetEditTool() => _toolManager.SetActiveTool("编辑");

    [RelayCommand]
    private void SetMeasureDistance() => _toolManager.SetActiveTool("量测");

    [RelayCommand]
    private void SetMeasureArea() => _toolManager.SetActiveTool("量测");

    [RelayCommand]
    private void Undo()
    {
        _toolManager.Undo();
        StatusMessage = "已撤销";
    }

    [RelayCommand]
    private void Redo()
    {
        _toolManager.Redo();
        StatusMessage = "已重做";
    }

    // ─── Pointer Event Forwarding (called from GisView.axaml.cs) ──

    public void OnMapPointerPressed(double lon, double lat, bool shift, bool dblClick, double sx, double sy)
    {
        _toolManager.OnPointerPressed(new ToolPointerEventArgs
        {
            Longitude = lon, Latitude = lat,
            IsShiftPressed = shift, IsDoubleClick = dblClick,
            ScreenX = sx, ScreenY = sy
        });
    }

    public void OnMapPointerMoved(double lon, double lat, double sx, double sy)
    {
        CursorPosition = $"{lon:F6}, {lat:F6}";
        _toolManager.OnPointerMoved(new ToolPointerEventArgs
        {
            Longitude = lon, Latitude = lat,
            ScreenX = sx, ScreenY = sy
        });
    }

    public void OnMapPointerReleased(double lon, double lat, double sx, double sy)
    {
        _toolManager.OnPointerReleased(new ToolPointerEventArgs
        {
            Longitude = lon, Latitude = lat,
            ScreenX = sx, ScreenY = sy
        });
    }

    public void OnMapKeyDown(string key)
    {
        // Single-char keys trigger tool switch
        if (key.Length == 1)
        {
            _toolManager.SwitchToolByKey(key[0]);
        }
        _toolManager.OnKeyDown(new ToolKeyEventArgs { Key = key });
    }

    // ─── Import ───────────────────────────────────

    [RelayCommand]
    private async Task ImportGeoJsonAsync()
    {
        var topLevel = GetTopLevel();
        if (topLevel is null) return;

        try
        {
            var files = await topLevel.StorageProvider.OpenFilePickerAsync(new FilePickerOpenOptions
            {
                Title = "选择 GeoJSON 文件",
                AllowMultiple = false,
                FileTypeFilter = [new FilePickerFileType("GeoJSON") { Patterns = ["*.geojson", "*.json"] }]
            });

            if (files.Count == 0) return;

            IsLoading = true;
            StatusMessage = "正在导入 GeoJSON...";

            var file = files[0];
            await using var stream = await file.OpenReadAsync();
            using var reader = new StreamReader(stream);
            var json = await reader.ReadToEndAsync();

            var layerName = Path.GetFileNameWithoutExtension(file.Name);
            var layer = _fileService.ImportGeoJson(json, layerName);

            await _repository.CreateLayerAsync(layer);

            Layers.Add(new GisLayerItem
            {
                Id = layer.Id,
                Name = layer.Name,
                IsVisible = true,
                FeatureCount = layer.Features.Count,
                GeometryType = layer.Features.Count > 0
                    ? layer.Features[0].GeometryType
                    : "混合"
            });

            _mapService.AddOrUpdateLayer(layer);
            StatusMessage = $"已导入 {layer.Name}: {layer.Features.Count} 个要素";
        }
        catch (Exception ex)
        {
            StatusMessage = $"导入 GeoJSON 失败: {ex.Message}";
        }
        finally
        {
            IsLoading = false;
        }
    }

    [RelayCommand]
    private async Task ImportShapefileAsync()
    {
        var topLevel = GetTopLevel();
        if (topLevel is null) return;

        try
        {
            var files = await topLevel.StorageProvider.OpenFilePickerAsync(new FilePickerOpenOptions
            {
                Title = "选择 Shapefile 文件",
                AllowMultiple = false,
                FileTypeFilter = [new FilePickerFileType("Shapefile") { Patterns = ["*.shp"] }]
            });

            if (files.Count == 0) return;

            IsLoading = true;
            StatusMessage = "正在导入 Shapefile...";

            var file = files[0];
            var localPath = file.TryGetLocalPath();
            if (localPath is null)
            {
                StatusMessage = "无法获取文件路径";
                return;
            }

            var layerName = Path.GetFileNameWithoutExtension(file.Name);
            var layer = _fileService.ImportShapefile(localPath, layerName);

            await _repository.CreateLayerAsync(layer);

            Layers.Add(new GisLayerItem
            {
                Id = layer.Id,
                Name = layer.Name,
                IsVisible = true,
                FeatureCount = layer.Features.Count,
                GeometryType = layer.Features.Count > 0
                    ? layer.Features[0].GeometryType
                    : "混合"
            });

            _mapService.AddOrUpdateLayer(layer);
            StatusMessage = $"已导入 {layer.Name}: {layer.Features.Count} 个要素";
        }
        catch (Exception ex)
        {
            StatusMessage = $"导入 Shapefile 失败: {ex.Message}";
        }
        finally
        {
            IsLoading = false;
        }
    }

    // ─── Export ────────────────────────────────────

    [RelayCommand]
    private async Task ExportGeoJsonAsync()
    {
        if (SelectedLayer is null)
        {
            StatusMessage = "请先选择要导出的图层";
            return;
        }

        var topLevel = GetTopLevel();
        if (topLevel is null) return;

        try
        {
            var file = await topLevel.StorageProvider.SaveFilePickerAsync(new FilePickerSaveOptions
            {
                Title = "导出 GeoJSON",
                SuggestedFileName = $"{SelectedLayer.Name}.geojson",
                FileTypeChoices = [new FilePickerFileType("GeoJSON") { Patterns = ["*.geojson"] }]
            });

            if (file is null) return;

            IsLoading = true;
            StatusMessage = "正在导出 GeoJSON...";

            var layer = await _repository.GetLayerByIdAsync(SelectedLayer.Id);
            if (layer is null)
            {
                StatusMessage = "图层不存在";
                return;
            }

            var json = _fileService.ExportGeoJson(layer);

            await using var stream = await file.OpenWriteAsync();
            await using var writer = new StreamWriter(stream);
            await writer.WriteAsync(json);

            StatusMessage = $"已导出: {file.Name}";
        }
        catch (Exception ex)
        {
            StatusMessage = $"导出 GeoJSON 失败: {ex.Message}";
        }
        finally
        {
            IsLoading = false;
        }
    }

    // ─── Spatial Analysis ─────────────────────────

    [RelayCommand]
    private async Task CalculateAreaAsync()
    {
        if (SelectedLayer is null)
        {
            AnalysisResult = "请先选择一个图层";
            return;
        }

        try
        {
            var layer = await _repository.GetLayerByIdAsync(SelectedLayer.Id);
            if (layer is null || layer.Features.Count == 0)
            {
                AnalysisResult = "图层为空，无法计算面积";
                return;
            }

            var lines = new List<string> { $"图层 [{layer.Name}] 面积计算结果:" };
            foreach (var f in layer.Features)
            {
                var area = _geometryService.Area(f.GeometryWkt);
                var name = f.Name ?? f.Id.ToString()[..8];
                lines.Add($"  {name}: {area:F6}");
            }
            AnalysisResult = string.Join("\n", lines);
        }
        catch (Exception ex)
        {
            AnalysisResult = $"计算失败: {ex.Message}";
        }
    }

    [RelayCommand]
    private async Task CalculateDistanceAsync()
    {
        if (SelectedLayer is null)
        {
            AnalysisResult = "请先选择一个图层";
            return;
        }

        try
        {
            var layer = await _repository.GetLayerByIdAsync(SelectedLayer.Id);
            if (layer is null || layer.Features.Count < 2)
            {
                AnalysisResult = "至少需要两个要素来计算距离";
                return;
            }

            var f1 = layer.Features[0];
            var f2 = layer.Features[1];
            var distance = _geometryService.Distance(f1.GeometryWkt, f2.GeometryWkt);
            var name1 = f1.Name ?? f1.Id.ToString()[..8];
            var name2 = f2.Name ?? f2.Id.ToString()[..8];
            AnalysisResult = $"距离计算:\n  {name1} ↔ {name2}\n  距离: {distance:F6}";
        }
        catch (Exception ex)
        {
            AnalysisResult = $"计算失败: {ex.Message}";
        }
    }

    [RelayCommand]
    private void OpenAnalysisDialog()
    {
        // TODO: Open AnalysisDialog as modal window
        // The AnalysisDialogViewModel will handle the actual analysis
        StatusMessage = "空间分析对话框 (待集成)";
    }

    // ─── Attribute Table ──────────────────────────

    [RelayCommand]
    private void ToggleAttributeTable()
    {
        if (AttributeTable is null)
        {
            AttributeTable = App.Services.GetRequiredService<AttributeTableViewModel>();
        }
        AttributeTable.IsVisible = !AttributeTable.IsVisible;

        if (AttributeTable.IsVisible && SelectedLayer is not null)
        {
            _ = AttributeTable.LoadLayerCommand.ExecuteAsync(SelectedLayer.Id);
        }
    }

    // ─── Helpers ──────────────────────────────────

    private static TopLevel? GetTopLevel()
    {
        if (Application.Current?.ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
            return desktop.MainWindow;
        return null;
    }
}

// ─── Layer Item (Observable wrapper) ──────────

public partial class GisLayerItem : ObservableObject
{
    public Ulid Id { get; set; }

    [ObservableProperty]
    private string _name = string.Empty;

    [ObservableProperty]
    private bool _isVisible = true;

    [ObservableProperty]
    private string _fillColor = "#3388FF40";

    [ObservableProperty]
    private string _strokeColor = "#3388FF";

    [ObservableProperty]
    private int _featureCount;

    [ObservableProperty]
    private string _geometryType = "混合";
}
```

- [ ] **Step 2: Verify build**

```bash
dotnet build src/Wif.App/Wif.App.csproj
```

---

### Task 13: Add Wif.Gis project reference to Wif.App

**Files:** Modify `src/Wif.App/Wif.App.csproj`

The GisView.axaml.cs uses `Mapsui.UI.Avalonia` and `Mapsui.Projections` — these are from the Mapsui NuGet packages in Wif.Gis. The App project needs to reference them either via a transitive dependency or directly.

- [ ] **Step 1: Check if Wif.App already references Wif.Gis transitively via Wif.Core**

```bash
grep -r "Wif.Gis\|Wif.Core\|Mapsui" src/Wif.App/Wif.App.csproj
grep -r "Wif.Gis" src/Wif.Core/Wif.Core.csproj
```

- [ ] **Step 2: If Mapsui types are not resolved, add a direct PackageReference in Wif.App.csproj**

Add to the NuGet `<ItemGroup>`:

```xml
<PackageReference Include="Mapsui.Avalonia" Version="5.0.2" />
<PackageReference Include="Mapsui.Nts" Version="5.0.2" />
```

- [ ] **Step 3: Verify the entire solution builds**

```bash
dotnet build Wif.sln
```

---

### Task 14: Full Build and Smoke Test

- [ ] **Step 1: Clean build the entire solution**

```bash
dotnet clean Wif.sln
dotnet build Wif.sln
```

Expected: 0 errors.

- [ ] **Step 2: Run all existing tests to verify no regressions**

```bash
dotnet test Wif.sln
```

Expected: All existing tests pass.

- [ ] **Step 3: Fix any compilation errors**

If errors occur, read the error messages and fix them. Common issues:
- Missing `using` directives
- Type name mismatches between agent outputs (check IMapTool event args naming)
- Missing DI registrations
- Namespace conflicts

- [ ] **Step 4: Commit all changes**

```bash
git add -A
git commit -m "feat(gis): integrate all GIS subsystems — map rendering, editing tools, spatial analysis, WorkItem integration

- Merge domain entity changes (GisLayer.SourceType/CrsCode, GisFeature.WorkItemId, WorkItem location fields)
- Wire MapService/ToolManager/Analysis/Integration into GisViewModel
- Replace empty Canvas with Mapsui MapControl in GisView
- Register all new services in DependencyInjection
- Update EF configurations and add database migrations"
```

---

## File Summary

| File | Action | Description |
|------|--------|-------------|
| `src/Wif.Domain/Entities/GisLayer.cs` | Merge | Add SourceType + CrsCode |
| `src/Wif.Domain/Entities/GisFeature.cs` | Merge | Add WorkItemId + WorkItem nav |
| `src/Wif.Domain/Entities/WorkItem.cs` | Merge | Add Longitude/Latitude/GisFeatureId + GisFeature nav |
| `src/Wif.Gis/Wif.Gis.csproj` | Merge | Ensure Mapsui + ProjNET packages |
| `src/Wif.Data/Configurations/GisLayerConfiguration.cs` | Modify | Map SourceType + CrsCode |
| `src/Wif.Data/Configurations/GisFeatureConfiguration.cs` | Modify | Map WorkItemId + relationship |
| `src/Wif.Data/Configurations/WorkItemConfiguration.cs` | Modify | Map GisFeatureId + relationship |
| `src/Wif.Data/Migrations/GisMigration.cs` | Modify | Add new columns idempotently |
| `src/Wif.Core/Services/DependencyInjection.cs` | Modify | Register all new GIS services |
| `src/Wif.App/App.axaml.cs` | Modify | Register WorkItemGeo migration + new ViewModels |
| `src/Wif.App/Views/GisView.axaml` | Rewrite | MapControl + toolbar + basemap selector + attribute table |
| `src/Wif.App/Views/GisView.axaml.cs` | Rewrite | Wire MapControl events to ViewModel |
| `src/Wif.App/ViewModels/GisViewModel.cs` | Rewrite | Full orchestration of all GIS subsystems |
| `src/Wif.App/Wif.App.csproj` | Modify | Add Mapsui package references if needed |

## Cross-Agent Type Resolution Notes

If compilation fails due to type mismatches between agents, check these common sources:

1. **IMapTool event args**: The gis-editing agent may name the event args differently (e.g., `MapToolEventArgs` vs `ToolPointerEventArgs`). Read the actual file and use the type names defined there.
2. **DrawGeometryType location**: May be in `DrawingTool.cs` as a nested type or as a standalone enum. Check the actual file.
3. **MeasureTool**: Depends on IMapTool from gis-editing. If the interface changed, adapt MeasureTool.
4. **AnalysisDialogViewModel**: May use types from gis-analysis (SpatialAnalysisService, AnalysisResult). Verify constructor params match.
5. **GetRequiredService<T>**: Ensure all types resolved via DI have matching registrations.

The assembler's job is to make it all compile — read the actual agent output files and resolve any inconsistencies.

# gis-editing Implementation Plan

> **For agentic workers:** Use this plan to implement the gis-editing module task-by-task.

**Goal:** Provide map editing tools (draw/edit/select) and attribute table for GIS features.

**Architecture:** IMapTool interface + ToolManager orchestration + Command pattern for undo/redo. AttributeTablePanel is an independent UserControl.

**Tech Stack:** Avalonia 11.2.1, CommunityToolkit.Mvvm 8.4.2, NetTopologySuite 2.6.0, Mapsui

---

## File Inventory

| # | File (NEW) | Purpose |
|---|------------|---------|
| 1 | `src/Wif.Gis/Editing/IMapTool.cs` | Tool interface + event args + tool result |
| 2 | `src/Wif.Gis/Editing/IEditCommand.cs` | Command interface for undo/redo |
| 3 | `src/Wif.Gis/Editing/ToolManager.cs` | Active tool management, undo/redo stack |
| 4 | `src/Wif.Gis/Editing/DrawingTool.cs` | Draw Point/LineString/Polygon interactively |
| 5 | `src/Wif.Gis/Editing/EditingTool.cs` | Select and modify feature geometry/vertices |
| 6 | `src/Wif.Gis/Editing/SelectionTool.cs` | Click/box selection with hit-testing |
| 7 | `src/Wif.App/Views/AttributeTablePanel.axaml` | Bottom collapsible panel with DataGrid |
| 8 | `src/Wif.App/Views/AttributeTablePanel.axaml.cs` | Code-behind for AttributeTablePanel |
| 9 | `src/Wif.App/ViewModels/AttributeTableViewModel.cs` | ViewModel for attribute table |

**DO NOT modify:** `GisView.axaml`, `GisViewModel.cs`, `DependencyInjection.cs` -- the assembler agent handles those.

---

## Existing Types Reference

### GisFeature (`src/Wif.Domain/Entities/GisFeature.cs`)
```csharp
public class GisFeature
{
    public Ulid Id { get; init; }
    public Ulid LayerId { get; init; }
    public required string GeometryWkt { get; set; }  // WKT format
    public string GeometryType { get; set; } = "Point";
    public string? Name { get; set; }
    public string? PropertiesJson { get; set; }  // JSON bag
    public DateTimeOffset CreatedAt { get; init; }
    public DateTimeOffset UpdatedAt { get; set; }
    public GisLayer? Layer { get; set; }
}
```

### GisLayer (`src/Wif.Domain/Entities/GisLayer.cs`)
```csharp
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
    public DateTimeOffset CreatedAt { get; init; }
    public DateTimeOffset UpdatedAt { get; set; }
    public List<GisFeature> Features { get; init; } = [];
}
```

### IGisLayerRepository (`src/Wif.Domain/Repositories/IGisLayerRepository.cs`)
```csharp
public interface IGisLayerRepository
{
    Task<GisLayer> CreateLayerAsync(GisLayer layer, CancellationToken ct = default);
    Task<GisLayer?> GetLayerByIdAsync(Ulid id, CancellationToken ct = default);
    Task<IReadOnlyList<GisLayer>> ListLayersAsync(CancellationToken ct = default);
    Task UpdateLayerAsync(GisLayer layer, CancellationToken ct = default);
    Task DeleteLayerAsync(Ulid id, CancellationToken ct = default);
    Task<GisFeature> AddFeatureAsync(GisFeature feature, CancellationToken ct = default);
    Task<IReadOnlyList<GisFeature>> GetFeaturesByLayerAsync(Ulid layerId, CancellationToken ct = default);
    Task UpdateFeatureAsync(GisFeature feature, CancellationToken ct = default);
    Task DeleteFeatureAsync(Ulid id, CancellationToken ct = default);
}
```

### GeometryService (`src/Wif.Gis/Services/GeometryService.cs`)
```csharp
public sealed class GeometryService
{
    public Geometry ParseWkt(string wkt) => ...;
    public string ToWkt(Geometry geometry) => ...;
    public string Buffer(string wkt, double distance) => ...;
    public string Intersection(string wktA, string wktB) => ...;
    public double Distance(string wktA, string wktB) => ...;
    public double Area(string wkt) => ...;
    public bool Contains(string wktA, string wktB) => ...;
    public bool Intersects(string wktA, string wktB) => ...;
    public string Centroid(string wkt) => ...;
    public string Envelope(string wkt) => ...;
    public bool IsValid(string wkt) => ...;
    // ... plus Union, Difference, SymmetricDifference, ConvexHull, Simplify, Length
}
```

### Wif.Gis Project (`src/Wif.Gis/Wif.Gis.csproj`)
```xml
<TargetFramework>net9.0</TargetFramework>
<PackageReference Include="NetTopologySuite" Version="2.6.0" />
<PackageReference Include="NetTopologySuite.IO.GeoJSON" Version="4.0.0" />
<PackageReference Include="NetTopologySuite.IO.ShapeFile" Version="2.1.0" />
<PackageReference Include="NetTopologySuite.Features" Version="2.2.0" />
```

---

## Task 1: IMapTool Interface + Event Args + Commands

**Files:** `src/Wif.Gis/Editing/IMapTool.cs`, `src/Wif.Gis/Editing/IEditCommand.cs`

These define the contracts for all editing tools and the undo/redo command pattern.

- [ ] 1.1 Create `src/Wif.Gis/Editing/IMapTool.cs` with the complete content below:

```csharp
using Mapsui;
using Wif.Domain.Entities;

namespace Wif.Gis.Editing;

/// <summary>
/// 地图工具事件参数。所有坐标均为世界坐标（Spherical Mercator）。
/// </summary>
public sealed record MapToolEventArgs(
    MPoint ScreenPosition,
    MPoint WorldPosition,
    Map Map,
    bool IsShiftPressed = false,
    bool IsCtrlPressed = false);

/// <summary>
/// 工具操作结果，指示工具处理后发生了什么变化。
/// </summary>
public sealed record ToolResult(
    bool Handled,
    IEditCommand? Command = null,
    IReadOnlyList<GisFeature>? AffectedFeatures = null,
    bool RefreshPreviewLayer = false);

/// <summary>
/// 地图工具类型枚举。
/// </summary>
public enum MapToolType
{
    None,
    Select,
    Draw,
    Edit
}

/// <summary>
/// 绘制几何类型。
/// </summary>
public enum DrawGeometryType
{
    Point,
    LineString,
    Polygon
}

/// <summary>
/// 地图交互工具接口。每种工具实现此接口处理指针和键盘事件。
/// </summary>
public interface IMapTool
{
    /// <summary>工具名称（用于 UI 显示）。</summary>
    string Name { get; }

    /// <summary>工具类型。</summary>
    MapToolType ToolType { get; }

    /// <summary>工具是否处于活动操作中（如正在绘制多边形）。</summary>
    bool IsActive { get; }

    /// <summary>指针按下事件。</summary>
    ToolResult OnPointerPressed(MapToolEventArgs args);

    /// <summary>指针移动事件。</summary>
    ToolResult OnPointerMoved(MapToolEventArgs args);

    /// <summary>指针释放事件。</summary>
    ToolResult OnPointerReleased(MapToolEventArgs args);

    /// <summary>键盘按下事件。</summary>
    ToolResult OnKeyDown(string key);

    /// <summary>重置工具状态（取消当前操作）。</summary>
    void Reset();

    /// <summary>获取当前预览几何体（用于橡皮筋效果），返回 WKT 列表。</summary>
    IReadOnlyList<string> GetPreviewGeometries();
}
```

- [ ] 1.2 Create `src/Wif.Gis/Editing/IEditCommand.cs` with the complete content below:

```csharp
using Wif.Domain.Entities;

namespace Wif.Gis.Editing;

/// <summary>
/// 可撤销/重做的编辑命令接口（Command 模式）。
/// </summary>
public interface IEditCommand
{
    /// <summary>命令描述（用于 UI 显示）。</summary>
    string Description { get; }

    /// <summary>执行命令。</summary>
    Task ExecuteAsync(CancellationToken ct = default);

    /// <summary>撤销命令。</summary>
    Task UndoAsync(CancellationToken ct = default);
}

/// <summary>
/// 添加要素命令。
/// </summary>
public sealed class AddFeatureCommand : IEditCommand
{
    private readonly GisFeature _feature;
    private readonly Func<GisFeature, CancellationToken, Task> _addFunc;
    private readonly Func<Ulid, CancellationToken, Task> _deleteFunc;

    public AddFeatureCommand(
        GisFeature feature,
        Func<GisFeature, CancellationToken, Task> addFunc,
        Func<Ulid, CancellationToken, Task> deleteFunc)
    {
        _feature = feature;
        _addFunc = addFunc;
        _deleteFunc = deleteFunc;
    }

    public string Description => $"添加要素: {_feature.Name ?? _feature.Id.ToString()[..8]}";

    public Task ExecuteAsync(CancellationToken ct = default) =>
        _addFunc(_feature, ct);

    public Task UndoAsync(CancellationToken ct = default) =>
        _deleteFunc(_feature.Id, ct);
}

/// <summary>
/// 删除要素命令。
/// </summary>
public sealed class RemoveFeatureCommand : IEditCommand
{
    private readonly GisFeature _feature;
    private readonly Func<GisFeature, CancellationToken, Task> _addFunc;
    private readonly Func<Ulid, CancellationToken, Task> _deleteFunc;

    public RemoveFeatureCommand(
        GisFeature feature,
        Func<GisFeature, CancellationToken, Task> addFunc,
        Func<Ulid, CancellationToken, Task> deleteFunc)
    {
        _feature = feature;
        _addFunc = addFunc;
        _deleteFunc = deleteFunc;
    }

    public string Description => $"删除要素: {_feature.Name ?? _feature.Id.ToString()[..8]}";

    public Task ExecuteAsync(CancellationToken ct = default) =>
        _deleteFunc(_feature.Id, ct);

    public Task UndoAsync(CancellationToken ct = default) =>
        _addFunc(_feature, ct);
}

/// <summary>
/// 修改要素几何命令（顶点编辑）。
/// </summary>
public sealed class ModifyGeometryCommand : IEditCommand
{
    private readonly Ulid _featureId;
    private readonly string _oldWkt;
    private readonly string _newWkt;
    private readonly Func<Ulid, string, CancellationToken, Task> _updateFunc;

    public ModifyGeometryCommand(
        Ulid featureId,
        string oldWkt,
        string newWkt,
        Func<Ulid, string, CancellationToken, Task> updateFunc)
    {
        _featureId = featureId;
        _oldWkt = oldWkt;
        _newWkt = newWkt;
        _updateFunc = updateFunc;
    }

    public string Description => $"修改几何: {_featureId.ToString()[..8]}";

    public Task ExecuteAsync(CancellationToken ct = default) =>
        _updateFunc(_featureId, _newWkt, ct);

    public Task UndoAsync(CancellationToken ct = default) =>
        _updateFunc(_featureId, _oldWkt, ct);
}

/// <summary>
/// 移动要素命令（整体平移）。
/// </summary>
public sealed class MoveFeatureCommand : IEditCommand
{
    private readonly Ulid _featureId;
    private readonly string _oldWkt;
    private readonly string _newWkt;
    private readonly Func<Ulid, string, CancellationToken, Task> _updateFunc;

    public MoveFeatureCommand(
        Ulid featureId,
        string oldWkt,
        string newWkt,
        Func<Ulid, string, CancellationToken, Task> updateFunc)
    {
        _featureId = featureId;
        _oldWkt = oldWkt;
        _newWkt = newWkt;
        _updateFunc = updateFunc;
    }

    public string Description => $"移动要素: {_featureId.ToString()[..8]}";

    public Task ExecuteAsync(CancellationToken ct = default) =>
        _updateFunc(_featureId, _newWkt, ct);

    public Task UndoAsync(CancellationToken ct = default) =>
        _updateFunc(_featureId, _oldWkt, ct);
}
```

- [ ] 1.3 Verify build:
```bash
dotnet build src/Wif.Gis/Wif.Gis.csproj
```

> **Note:** This will fail until the Mapsui package reference is added to `Wif.Gis.csproj`. The gis-map agent or assembler agent adds `Mapsui` and `Mapsui.Avalonia` packages. If the package is not yet present when you implement this task, add the following to `src/Wif.Gis/Wif.Gis.csproj` inside the existing `<ItemGroup>` with package references:
> ```xml
> <PackageReference Include="Mapsui" Version="5.0.0-beta.1" />
> ```
> Use the same Mapsui version that the gis-map agent uses. Check if already present before adding.

---

## Task 2: ToolManager (Active Tool + Undo/Redo Stack)

**Files:** `src/Wif.Gis/Editing/ToolManager.cs`

The ToolManager orchestrates tool switching and maintains the undo/redo stacks.

- [ ] 2.1 Create `src/Wif.Gis/Editing/ToolManager.cs` with the complete content below:

```csharp
namespace Wif.Gis.Editing;

/// <summary>
/// 工具管理器：管理当前活动工具、工具切换、撤销/重做栈。
/// </summary>
public sealed class ToolManager
{
    private readonly Dictionary<MapToolType, IMapTool> _tools = [];
    private readonly Stack<IEditCommand> _undoStack = new();
    private readonly Stack<IEditCommand> _redoStack = new();

    private IMapTool? _activeTool;

    /// <summary>当前活动工具。</summary>
    public IMapTool? ActiveTool => _activeTool;

    /// <summary>当前活动工具类型。</summary>
    public MapToolType ActiveToolType => _activeTool?.ToolType ?? MapToolType.None;

    /// <summary>是否可以撤销。</summary>
    public bool CanUndo => _undoStack.Count > 0;

    /// <summary>是否可以重做。</summary>
    public bool CanRedo => _redoStack.Count > 0;

    /// <summary>撤销栈描述（用于 UI 提示）。</summary>
    public string? UndoDescription => _undoStack.Count > 0 ? _undoStack.Peek().Description : null;

    /// <summary>重做栈描述（用于 UI 提示）。</summary>
    public string? RedoDescription => _redoStack.Count > 0 ? _redoStack.Peek().Description : null;

    /// <summary>工具切换后触发。</summary>
    public event Action<MapToolType>? ToolChanged;

    /// <summary>撤销/重做栈变化后触发。</summary>
    public event Action? UndoRedoChanged;

    /// <summary>注册工具实例。</summary>
    public void RegisterTool(IMapTool tool)
    {
        _tools[tool.ToolType] = tool;
    }

    /// <summary>切换到指定工具类型。</summary>
    public void SetActiveTool(MapToolType toolType)
    {
        if (_activeTool?.ToolType == toolType) return;

        _activeTool?.Reset();

        _activeTool = _tools.GetValueOrDefault(toolType);
        ToolChanged?.Invoke(toolType);
    }

    /// <summary>取消当前工具操作并切换到无工具状态。</summary>
    public void Deactivate()
    {
        _activeTool?.Reset();
        _activeTool = null;
        ToolChanged?.Invoke(MapToolType.None);
    }

    // ─── Event Dispatch ──────────────────────────

    public ToolResult HandlePointerPressed(MapToolEventArgs args)
    {
        if (_activeTool is null) return new ToolResult(false);
        var result = _activeTool.OnPointerPressed(args);
        ProcessResult(result);
        return result;
    }

    public ToolResult HandlePointerMoved(MapToolEventArgs args)
    {
        if (_activeTool is null) return new ToolResult(false);
        return _activeTool.OnPointerMoved(args);
    }

    public ToolResult HandlePointerReleased(MapToolEventArgs args)
    {
        if (_activeTool is null) return new ToolResult(false);
        var result = _activeTool.OnPointerReleased(args);
        ProcessResult(result);
        return result;
    }

    public ToolResult HandleKeyDown(string key)
    {
        if (_activeTool is null) return new ToolResult(false);
        var result = _activeTool.OnKeyDown(key);
        ProcessResult(result);
        return result;
    }

    // ─── Undo / Redo ─────────────────────────────

    /// <summary>撤销上一次操作。</summary>
    public async Task UndoAsync(CancellationToken ct = default)
    {
        if (_undoStack.Count == 0) return;
        var command = _undoStack.Pop();
        await command.UndoAsync(ct);
        _redoStack.Push(command);
        UndoRedoChanged?.Invoke();
    }

    /// <summary>重做上一次撤销的操作。</summary>
    public async Task RedoAsync(CancellationToken ct = default)
    {
        if (_redoStack.Count == 0) return;
        var command = _redoStack.Pop();
        await command.ExecuteAsync(ct);
        _undoStack.Push(command);
        UndoRedoChanged?.Invoke();
    }

    /// <summary>将命令推入撤销栈（由工具在完成操作后调用）。</summary>
    public async Task ExecuteCommandAsync(IEditCommand command, CancellationToken ct = default)
    {
        await command.ExecuteAsync(ct);
        _undoStack.Push(command);
        _redoStack.Clear();
        UndoRedoChanged?.Invoke();
    }

    /// <summary>清空撤销/重做栈。</summary>
    public void ClearHistory()
    {
        _undoStack.Clear();
        _redoStack.Clear();
        UndoRedoChanged?.Invoke();
    }

    // ─── Internal ────────────────────────────────

    private void ProcessResult(ToolResult result)
    {
        if (result.Command is not null)
        {
            _undoStack.Push(result.Command);
            _redoStack.Clear();
            UndoRedoChanged?.Invoke();
        }
    }
}
```

- [ ] 2.2 Verify build:
```bash
dotnet build src/Wif.Gis/Wif.Gis.csproj
```

---

## Task 3: DrawingTool

**Files:** `src/Wif.Gis/Editing/DrawingTool.cs`

Handles interactive drawing of Point, LineString, and Polygon geometries by collecting vertex clicks.

- [ ] 3.1 Create `src/Wif.Gis/Editing/DrawingTool.cs` with the complete content below:

```csharp
using Mapsui;
using NetTopologySuite.Geometries;
using Wif.Domain.Entities;
using Wif.Gis.Services;

namespace Wif.Gis.Editing;

/// <summary>
/// 绘制工具：通过点击地图添加顶点，双击或 Enter 完成绘制，Escape 取消。
/// 支持 Point（单击即完成）、LineString、Polygon 三种模式。
/// </summary>
public sealed class DrawingTool : IMapTool
{
    private readonly GeometryService _geometryService;
    private readonly Func<GisFeature, CancellationToken, Task> _addFeatureFunc;
    private readonly Func<Ulid, CancellationToken, Task> _deleteFeatureFunc;
    private readonly Func<Ulid> _activeLayerIdFunc;

    private readonly List<Coordinate> _vertices = [];
    private Coordinate? _cursorPosition;
    private DrawGeometryType _drawType = DrawGeometryType.Point;
    private DateTime _lastClickTime = DateTime.MinValue;

    /// <summary>双击检测的时间阈值（毫秒）。</summary>
    private const int DoubleClickThresholdMs = 300;

    public DrawingTool(
        GeometryService geometryService,
        Func<GisFeature, CancellationToken, Task> addFeatureFunc,
        Func<Ulid, CancellationToken, Task> deleteFeatureFunc,
        Func<Ulid> activeLayerIdFunc)
    {
        _geometryService = geometryService;
        _addFeatureFunc = addFeatureFunc;
        _deleteFeatureFunc = deleteFeatureFunc;
        _activeLayerIdFunc = activeLayerIdFunc;
    }

    public string Name => "绘制工具";
    public MapToolType ToolType => MapToolType.Draw;
    public bool IsActive => _vertices.Count > 0;

    /// <summary>当前绘制的几何类型。</summary>
    public DrawGeometryType DrawType
    {
        get => _drawType;
        set
        {
            if (_drawType != value)
            {
                Reset();
                _drawType = value;
            }
        }
    }

    /// <summary>绘制完成时触发，传递新创建的要素和对应的命令。</summary>
    public event Action<GisFeature, IEditCommand>? FeatureCreated;

    public ToolResult OnPointerPressed(MapToolEventArgs args)
    {
        var coord = new Coordinate(args.WorldPosition.X, args.WorldPosition.Y);
        var now = DateTime.UtcNow;

        // 双击检测：完成绘制
        if ((now - _lastClickTime).TotalMilliseconds < DoubleClickThresholdMs
            && _vertices.Count > 0)
        {
            _lastClickTime = DateTime.MinValue;
            return FinishDrawing();
        }

        _lastClickTime = now;

        // Point 模式：单击即完成
        if (_drawType == DrawGeometryType.Point)
        {
            _vertices.Add(coord);
            return FinishDrawing();
        }

        // LineString / Polygon：累加顶点
        _vertices.Add(coord);
        return new ToolResult(true, RefreshPreviewLayer: true);
    }

    public ToolResult OnPointerMoved(MapToolEventArgs args)
    {
        if (_vertices.Count == 0) return new ToolResult(false);

        _cursorPosition = new Coordinate(args.WorldPosition.X, args.WorldPosition.Y);
        return new ToolResult(true, RefreshPreviewLayer: true);
    }

    public ToolResult OnPointerReleased(MapToolEventArgs args)
    {
        return new ToolResult(false);
    }

    public ToolResult OnKeyDown(string key)
    {
        return key switch
        {
            "Escape" => CancelDrawing(),
            "Enter" => _vertices.Count > 0 ? FinishDrawing() : new ToolResult(false),
            "Back" or "Backspace" => RemoveLastVertex(),
            _ => new ToolResult(false)
        };
    }

    public void Reset()
    {
        _vertices.Clear();
        _cursorPosition = null;
        _lastClickTime = DateTime.MinValue;
    }

    public IReadOnlyList<string> GetPreviewGeometries()
    {
        if (_vertices.Count == 0) return [];

        var previews = new List<string>();
        var coords = new List<Coordinate>(_vertices);

        // 添加光标位置作为临时最后一个顶点（橡皮筋效果）
        if (_cursorPosition is not null)
            coords.Add(_cursorPosition);

        try
        {
            switch (_drawType)
            {
                case DrawGeometryType.Point:
                    // 显示已点击的点
                    foreach (var v in _vertices)
                    {
                        var pt = new Point(v);
                        previews.Add(_geometryService.ToWkt(pt));
                    }
                    break;

                case DrawGeometryType.LineString:
                    if (coords.Count >= 2)
                    {
                        var line = new LineString([.. coords]);
                        previews.Add(_geometryService.ToWkt(line));
                    }
                    // 显示已确认的顶点
                    foreach (var v in _vertices)
                    {
                        previews.Add(_geometryService.ToWkt(new Point(v)));
                    }
                    break;

                case DrawGeometryType.Polygon:
                    if (coords.Count >= 3)
                    {
                        // 闭合多边形预览
                        var ringCoords = new List<Coordinate>(coords) { coords[0] };
                        var ring = new LinearRing([.. ringCoords]);
                        var polygon = new Polygon(ring);
                        previews.Add(_geometryService.ToWkt(polygon));
                    }
                    else if (coords.Count == 2)
                    {
                        var line = new LineString([.. coords]);
                        previews.Add(_geometryService.ToWkt(line));
                    }
                    // 显示已确认的顶点
                    foreach (var v in _vertices)
                    {
                        previews.Add(_geometryService.ToWkt(new Point(v)));
                    }
                    break;
            }
        }
        catch
        {
            // 几何构造异常时静默忽略预览
        }

        return previews;
    }

    // ─── Internal ────────────────────────────────

    private ToolResult FinishDrawing()
    {
        if (_vertices.Count == 0) return new ToolResult(false);

        Geometry? geometry = null;
        string geometryType;

        try
        {
            switch (_drawType)
            {
                case DrawGeometryType.Point:
                    if (_vertices.Count < 1) return CancelDrawing();
                    geometry = new Point(_vertices[0]);
                    geometryType = "Point";
                    break;

                case DrawGeometryType.LineString:
                    if (_vertices.Count < 2) return CancelDrawing();
                    geometry = new LineString([.. _vertices]);
                    geometryType = "LineString";
                    break;

                case DrawGeometryType.Polygon:
                    if (_vertices.Count < 3) return CancelDrawing();
                    var ringCoords = new List<Coordinate>(_vertices) { _vertices[0] };
                    var ring = new LinearRing([.. ringCoords]);
                    geometry = new Polygon(ring);
                    geometryType = "Polygon";
                    break;

                default:
                    return CancelDrawing();
            }
        }
        catch
        {
            return CancelDrawing();
        }

        var wkt = _geometryService.ToWkt(geometry);
        var now = DateTimeOffset.UtcNow;
        var feature = new GisFeature
        {
            Id = Ulid.NewUlid(),
            LayerId = _activeLayerIdFunc(),
            GeometryWkt = wkt,
            GeometryType = geometryType,
            CreatedAt = now,
            UpdatedAt = now
        };

        var command = new AddFeatureCommand(feature, _addFeatureFunc, _deleteFeatureFunc);
        Reset();

        FeatureCreated?.Invoke(feature, command);
        return new ToolResult(true, command, [feature], RefreshPreviewLayer: true);
    }

    private ToolResult CancelDrawing()
    {
        Reset();
        return new ToolResult(true, RefreshPreviewLayer: true);
    }

    private ToolResult RemoveLastVertex()
    {
        if (_vertices.Count == 0) return new ToolResult(false);
        _vertices.RemoveAt(_vertices.Count - 1);
        return new ToolResult(true, RefreshPreviewLayer: true);
    }
}
```

- [ ] 3.2 Verify build:
```bash
dotnet build src/Wif.Gis/Wif.Gis.csproj
```

---

## Task 4: SelectionTool

**Files:** `src/Wif.Gis/Editing/SelectionTool.cs`

Handles click-to-select with hit-testing, Shift+click multi-select, and drag-box selection.

- [ ] 4.1 Create `src/Wif.Gis/Editing/SelectionTool.cs` with the complete content below:

```csharp
using Mapsui;
using NetTopologySuite.Geometries;
using Wif.Domain.Entities;
using Wif.Gis.Services;

namespace Wif.Gis.Editing;

/// <summary>
/// 选择工具：点击选择（距离命中测试）、Shift+点击多选、拖动框选。
/// </summary>
public sealed class SelectionTool : IMapTool
{
    private readonly GeometryService _geometryService;
    private readonly Func<IReadOnlyList<GisFeature>> _featuresFunc;

    private readonly List<GisFeature> _selectedFeatures = [];
    private MPoint? _dragStart;
    private bool _isDragging;

    /// <summary>命中测试的像素容差（世界坐标距离由视口分辨率转换）。</summary>
    public double HitTolerancePixels { get; set; } = 10.0;

    public SelectionTool(
        GeometryService geometryService,
        Func<IReadOnlyList<GisFeature>> featuresFunc)
    {
        _geometryService = geometryService;
        _featuresFunc = featuresFunc;
    }

    public string Name => "选择工具";
    public MapToolType ToolType => MapToolType.Select;
    public bool IsActive => _isDragging;

    /// <summary>当前选中的要素列表。</summary>
    public IReadOnlyList<GisFeature> SelectedFeatures => _selectedFeatures;

    /// <summary>选择变化时触发。</summary>
    public event Action<IReadOnlyList<GisFeature>>? SelectionChanged;

    public ToolResult OnPointerPressed(MapToolEventArgs args)
    {
        _dragStart = args.WorldPosition;
        _isDragging = false;
        return new ToolResult(true);
    }

    public ToolResult OnPointerMoved(MapToolEventArgs args)
    {
        if (_dragStart is null) return new ToolResult(false);

        var dx = Math.Abs(args.ScreenPosition.X - _dragStart.X);
        var dy = Math.Abs(args.ScreenPosition.Y - _dragStart.Y);

        // 超过阈值开始框选
        if (dx > 3 || dy > 3)
            _isDragging = true;

        if (_isDragging)
            return new ToolResult(true, RefreshPreviewLayer: true);

        return new ToolResult(false);
    }

    public ToolResult OnPointerReleased(MapToolEventArgs args)
    {
        if (_isDragging && _dragStart is not null)
        {
            // 框选模式
            var result = BoxSelect(_dragStart, args.WorldPosition, args.IsShiftPressed);
            _dragStart = null;
            _isDragging = false;
            return result;
        }

        // 点选模式
        _dragStart = null;
        _isDragging = false;
        return PointSelect(args);
    }

    public ToolResult OnKeyDown(string key)
    {
        if (key is "Escape")
        {
            ClearSelection();
            return new ToolResult(true);
        }

        if (key is "Delete" or "Back" or "Backspace")
        {
            // 通知外部删除选中要素（不在此处直接执行命令）
            var affected = _selectedFeatures.ToList();
            ClearSelection();
            return new ToolResult(true, AffectedFeatures: affected);
        }

        return new ToolResult(false);
    }

    public void Reset()
    {
        _dragStart = null;
        _isDragging = false;
        ClearSelection();
    }

    public IReadOnlyList<string> GetPreviewGeometries()
    {
        if (!_isDragging || _dragStart is null) return [];

        // 返回选择框的矩形预览（以当前光标为终点，但这里没有光标，
        // 由调用方在 OnPointerMoved 时更新）
        return [];
    }

    /// <summary>获取框选矩形的 WKT（供外部渲染）。</summary>
    public string? GetSelectionBoxWkt(MPoint currentWorld)
    {
        if (_dragStart is null || !_isDragging) return null;

        try
        {
            var minX = Math.Min(_dragStart.X, currentWorld.X);
            var minY = Math.Min(_dragStart.Y, currentWorld.Y);
            var maxX = Math.Max(_dragStart.X, currentWorld.X);
            var maxY = Math.Max(_dragStart.Y, currentWorld.Y);

            var envelope = new Envelope(minX, maxX, minY, maxY);
            var box = new GeometryFactory().ToGeometry(envelope);
            return _geometryService.ToWkt(box);
        }
        catch
        {
            return null;
        }
    }

    // ─── Internal ────────────────────────────────

    private ToolResult PointSelect(MapToolEventArgs args)
    {
        var features = _featuresFunc();
        var worldPos = new Coordinate(args.WorldPosition.X, args.WorldPosition.Y);

        // 计算世界坐标的容差（基于视口分辨率）
        var resolution = args.Map.Navigator.Viewport.Resolution;
        var tolerance = HitTolerancePixels * resolution;

        GisFeature? hit = null;
        var minDistance = double.MaxValue;

        foreach (var feature in features)
        {
            try
            {
                var geom = _geometryService.ParseWkt(feature.GeometryWkt);
                var distance = geom.Distance(new Point(worldPos));

                if (distance <= tolerance && distance < minDistance)
                {
                    minDistance = distance;
                    hit = feature;
                }
            }
            catch
            {
                // 跳过无效几何
            }
        }

        if (hit is not null)
        {
            if (args.IsShiftPressed)
            {
                // Shift+点击：切换选中状态
                if (_selectedFeatures.Contains(hit))
                    _selectedFeatures.Remove(hit);
                else
                    _selectedFeatures.Add(hit);
            }
            else
            {
                // 普通点击：替换选择
                _selectedFeatures.Clear();
                _selectedFeatures.Add(hit);
            }
        }
        else if (!args.IsShiftPressed)
        {
            _selectedFeatures.Clear();
        }

        SelectionChanged?.Invoke(_selectedFeatures);
        return new ToolResult(true, AffectedFeatures: _selectedFeatures.ToList());
    }

    private ToolResult BoxSelect(MPoint start, MPoint end, bool addToSelection)
    {
        var minX = Math.Min(start.X, end.X);
        var minY = Math.Min(start.Y, end.Y);
        var maxX = Math.Max(start.X, end.X);
        var maxY = Math.Max(start.Y, end.Y);

        var envelope = new Envelope(minX, maxX, minY, maxY);
        var box = new GeometryFactory().ToGeometry(envelope);

        var features = _featuresFunc();

        if (!addToSelection)
            _selectedFeatures.Clear();

        foreach (var feature in features)
        {
            try
            {
                var geom = _geometryService.ParseWkt(feature.GeometryWkt);
                if (box.Intersects(geom) && !_selectedFeatures.Contains(feature))
                    _selectedFeatures.Add(feature);
            }
            catch
            {
                // 跳过无效几何
            }
        }

        SelectionChanged?.Invoke(_selectedFeatures);
        return new ToolResult(true, AffectedFeatures: _selectedFeatures.ToList(), RefreshPreviewLayer: true);
    }

    private void ClearSelection()
    {
        _selectedFeatures.Clear();
        SelectionChanged?.Invoke(_selectedFeatures);
    }
}
```

- [ ] 4.2 Verify build:
```bash
dotnet build src/Wif.Gis/Wif.Gis.csproj
```

---

## Task 5: EditingTool

**Files:** `src/Wif.Gis/Editing/EditingTool.cs`

Select a feature, show vertex handles, drag handles to modify geometry, drag body to move.

- [ ] 5.1 Create `src/Wif.Gis/Editing/EditingTool.cs` with the complete content below:

```csharp
using Mapsui;
using NetTopologySuite.Geometries;
using Wif.Domain.Entities;
using Wif.Gis.Services;

namespace Wif.Gis.Editing;

/// <summary>
/// 编辑工具：选择要素后显示顶点手柄，拖拽手柄修改顶点，拖拽要素整体移动。
/// </summary>
public sealed class EditingTool : IMapTool
{
    private readonly GeometryService _geometryService;
    private readonly Func<IReadOnlyList<GisFeature>> _featuresFunc;
    private readonly Func<Ulid, string, CancellationToken, Task> _updateGeometryFunc;

    private GisFeature? _editingFeature;
    private Geometry? _originalGeometry;
    private string? _originalWkt;

    // 顶点编辑状态
    private int _dragVertexIndex = -1;
    private bool _isDraggingFeature;
    private MPoint? _dragStartWorld;

    /// <summary>顶点手柄命中测试的像素容差。</summary>
    public double VertexTolerancePixels { get; set; } = 8.0;

    /// <summary>要素命中测试的像素容差。</summary>
    public double FeatureTolerancePixels { get; set; } = 10.0;

    public EditingTool(
        GeometryService geometryService,
        Func<IReadOnlyList<GisFeature>> featuresFunc,
        Func<Ulid, string, CancellationToken, Task> updateGeometryFunc)
    {
        _geometryService = geometryService;
        _featuresFunc = featuresFunc;
        _updateGeometryFunc = updateGeometryFunc;
    }

    public string Name => "编辑工具";
    public MapToolType ToolType => MapToolType.Edit;
    public bool IsActive => _editingFeature is not null;

    /// <summary>当前正在编辑的要素。</summary>
    public GisFeature? EditingFeature => _editingFeature;

    /// <summary>编辑的要素发生变化时触发。</summary>
    public event Action<GisFeature?>? EditingFeatureChanged;

    /// <summary>几何修改完成时触发（用于外部处理命令入栈）。</summary>
    public event Action<GisFeature, string, string>? GeometryModified;

    public ToolResult OnPointerPressed(MapToolEventArgs args)
    {
        var worldPos = args.WorldPosition;
        var resolution = args.Map.Navigator.Viewport.Resolution;

        // 如果已有编辑要素，先检查是否点击了顶点手柄
        if (_editingFeature is not null && _originalGeometry is not null)
        {
            var vertexTolerance = VertexTolerancePixels * resolution;
            var vertexIndex = FindNearestVertexIndex(
                _originalGeometry, worldPos, vertexTolerance);

            if (vertexIndex >= 0)
            {
                _dragVertexIndex = vertexIndex;
                _dragStartWorld = worldPos;
                _originalWkt = _editingFeature.GeometryWkt;
                return new ToolResult(true);
            }

            // 检查是否点击了要素本体（整体拖拽）
            var featureTolerance = FeatureTolerancePixels * resolution;
            var clickPoint = new Point(new Coordinate(worldPos.X, worldPos.Y));
            if (_originalGeometry.Distance(clickPoint) <= featureTolerance)
            {
                _isDraggingFeature = true;
                _dragStartWorld = worldPos;
                _originalWkt = _editingFeature.GeometryWkt;
                return new ToolResult(true);
            }
        }

        // 尝试选择新要素进行编辑
        var features = _featuresFunc();
        var hitTolerance = FeatureTolerancePixels * resolution;

        GisFeature? hit = null;
        var minDist = double.MaxValue;

        foreach (var f in features)
        {
            try
            {
                var geom = _geometryService.ParseWkt(f.GeometryWkt);
                var dist = geom.Distance(
                    new Point(new Coordinate(worldPos.X, worldPos.Y)));

                if (dist <= hitTolerance && dist < minDist)
                {
                    minDist = dist;
                    hit = f;
                }
            }
            catch
            {
                // 跳过无效几何
            }
        }

        SetEditingFeature(hit);
        return new ToolResult(true, RefreshPreviewLayer: true);
    }

    public ToolResult OnPointerMoved(MapToolEventArgs args)
    {
        if (_editingFeature is null || _originalGeometry is null)
            return new ToolResult(false);

        var worldPos = args.WorldPosition;

        // 拖拽顶点
        if (_dragVertexIndex >= 0 && _dragStartWorld is not null)
        {
            var modified = MoveVertex(
                _originalGeometry, _dragVertexIndex,
                new Coordinate(worldPos.X, worldPos.Y));

            if (modified is not null)
            {
                _editingFeature.GeometryWkt = _geometryService.ToWkt(modified);
                _originalGeometry = modified;
            }

            return new ToolResult(true, RefreshPreviewLayer: true);
        }

        // 拖拽整体移动
        if (_isDraggingFeature && _dragStartWorld is not null)
        {
            var dx = worldPos.X - _dragStartWorld.X;
            var dy = worldPos.Y - _dragStartWorld.Y;

            var moved = TranslateGeometry(_originalGeometry, dx, dy);
            if (moved is not null)
            {
                _editingFeature.GeometryWkt = _geometryService.ToWkt(moved);
                // 注意：不更新 _originalGeometry，保持增量计算
                // 但需要用新位置作为下次增量的起点
                _originalGeometry = moved;
                _dragStartWorld = worldPos;
            }

            return new ToolResult(true, RefreshPreviewLayer: true);
        }

        return new ToolResult(false);
    }

    public ToolResult OnPointerReleased(MapToolEventArgs args)
    {
        if (_editingFeature is null) return new ToolResult(false);

        IEditCommand? command = null;

        // 完成顶点拖拽
        if (_dragVertexIndex >= 0 && _originalWkt is not null)
        {
            var newWkt = _editingFeature.GeometryWkt;
            if (_originalWkt != newWkt)
            {
                command = new ModifyGeometryCommand(
                    _editingFeature.Id, _originalWkt, newWkt, _updateGeometryFunc);
                GeometryModified?.Invoke(_editingFeature, _originalWkt, newWkt);
            }
        }

        // 完成整体移动
        if (_isDraggingFeature && _originalWkt is not null)
        {
            var newWkt = _editingFeature.GeometryWkt;
            if (_originalWkt != newWkt)
            {
                command = new MoveFeatureCommand(
                    _editingFeature.Id, _originalWkt, newWkt, _updateGeometryFunc);
                GeometryModified?.Invoke(_editingFeature, _originalWkt, newWkt);
            }
        }

        _dragVertexIndex = -1;
        _isDraggingFeature = false;
        _dragStartWorld = null;
        _originalWkt = null;

        return new ToolResult(command is not null, command, RefreshPreviewLayer: true);
    }

    public ToolResult OnKeyDown(string key)
    {
        if (key is "Escape")
        {
            SetEditingFeature(null);
            return new ToolResult(true, RefreshPreviewLayer: true);
        }

        return new ToolResult(false);
    }

    public void Reset()
    {
        _dragVertexIndex = -1;
        _isDraggingFeature = false;
        _dragStartWorld = null;
        _originalWkt = null;
        SetEditingFeature(null);
    }

    public IReadOnlyList<string> GetPreviewGeometries()
    {
        if (_editingFeature is null || _originalGeometry is null)
            return [];

        var previews = new List<string>();

        // 返回每个顶点作为 Point（用于渲染手柄）
        var coords = _originalGeometry.Coordinates;
        foreach (var coord in coords)
        {
            previews.Add(_geometryService.ToWkt(new Point(coord)));
        }

        return previews;
    }

    // ─── Internal ────────────────────────────────

    private void SetEditingFeature(GisFeature? feature)
    {
        _editingFeature = feature;
        _originalGeometry = feature is not null
            ? _geometryService.ParseWkt(feature.GeometryWkt)
            : null;
        _dragVertexIndex = -1;
        _isDraggingFeature = false;
        EditingFeatureChanged?.Invoke(feature);
    }

    private static int FindNearestVertexIndex(
        Geometry geometry, MPoint worldPos, double tolerance)
    {
        var coords = geometry.Coordinates;
        var minDist = double.MaxValue;
        var index = -1;

        for (var i = 0; i < coords.Length; i++)
        {
            var dx = coords[i].X - worldPos.X;
            var dy = coords[i].Y - worldPos.Y;
            var dist = Math.Sqrt(dx * dx + dy * dy);

            if (dist <= tolerance && dist < minDist)
            {
                minDist = dist;
                index = i;
            }
        }

        return index;
    }

    private static Geometry? MoveVertex(
        Geometry geometry, int vertexIndex, Coordinate newPosition)
    {
        var coords = geometry.Coordinates;
        if (vertexIndex < 0 || vertexIndex >= coords.Length) return null;

        var newCoords = new Coordinate[coords.Length];
        for (var i = 0; i < coords.Length; i++)
        {
            newCoords[i] = i == vertexIndex
                ? new Coordinate(newPosition.X, newPosition.Y)
                : new Coordinate(coords[i].X, coords[i].Y);
        }

        // 多边形需要闭合首尾
        if (geometry is Polygon && newCoords.Length > 0)
        {
            if (vertexIndex == 0)
                newCoords[^1] = new Coordinate(newPosition.X, newPosition.Y);
            else if (vertexIndex == newCoords.Length - 1)
                newCoords[0] = new Coordinate(newPosition.X, newPosition.Y);
        }

        try
        {
            return geometry switch
            {
                Point => new Point(newCoords[0]),
                LineString => new LineString(newCoords),
                Polygon => new Polygon(new LinearRing(newCoords)),
                _ => null
            };
        }
        catch
        {
            return null;
        }
    }

    private static Geometry? TranslateGeometry(Geometry geometry, double dx, double dy)
    {
        var coords = geometry.Coordinates;
        var newCoords = new Coordinate[coords.Length];

        for (var i = 0; i < coords.Length; i++)
        {
            newCoords[i] = new Coordinate(coords[i].X + dx, coords[i].Y + dy);
        }

        try
        {
            return geometry switch
            {
                Point => new Point(newCoords[0]),
                LineString => new LineString(newCoords),
                Polygon => new Polygon(new LinearRing(newCoords)),
                _ => null
            };
        }
        catch
        {
            return null;
        }
    }
}
```

- [ ] 5.2 Verify build:
```bash
dotnet build src/Wif.Gis/Wif.Gis.csproj
```

---

## Task 6: AttributeTableViewModel

**Files:** `src/Wif.App/ViewModels/AttributeTableViewModel.cs`

ViewModel for the attribute table panel: loads features for selected layer, supports cell editing, emits zoom/highlight events.

- [ ] 6.1 Create `src/Wif.App/ViewModels/AttributeTableViewModel.cs` with the complete content below:

```csharp
using System.Collections.ObjectModel;
using System.Text.Json;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Wif.Domain.Entities;
using Wif.Domain.Repositories;

namespace Wif.App.ViewModels;

/// <summary>
/// 属性表中一行数据的视图模型。
/// </summary>
public sealed partial class AttributeRowViewModel : ObservableObject
{
    public Ulid FeatureId { get; init; }

    [ObservableProperty]
    private string _name = string.Empty;

    [ObservableProperty]
    private string _geometryType = string.Empty;

    [ObservableProperty]
    private string _geometryWkt = string.Empty;

    /// <summary>属性键值对（从 PropertiesJson 解析）。</summary>
    public ObservableCollection<AttributeEntry> Attributes { get; } = [];

    [ObservableProperty]
    private bool _isSelected;
}

/// <summary>
/// 单个属性键值对。
/// </summary>
public sealed partial class AttributeEntry : ObservableObject
{
    [ObservableProperty]
    private string _key = string.Empty;

    [ObservableProperty]
    private string _value = string.Empty;
}

/// <summary>
/// 属性表面板的 ViewModel：加载指定图层的要素列表，支持选择、编辑属性、缩放定位。
/// </summary>
public sealed partial class AttributeTableViewModel : ObservableObject
{
    private readonly IGisLayerRepository _repository;

    public AttributeTableViewModel(IGisLayerRepository repository)
    {
        _repository = repository;
    }

    // ─── State ───────────────────────────────────

    public ObservableCollection<AttributeRowViewModel> Rows { get; } = [];

    [ObservableProperty]
    private AttributeRowViewModel? _selectedRow;

    [ObservableProperty]
    private bool _isExpanded = true;

    [ObservableProperty]
    private bool _isLoading;

    [ObservableProperty]
    private string _panelTitle = "属性表";

    [ObservableProperty]
    private Ulid? _activeLayerId;

    /// <summary>所有属性列名（动态生成）。</summary>
    public ObservableCollection<string> ColumnNames { get; } = [];

    // ─── Events ──────────────────────────────────

    /// <summary>请求缩放到指定要素时触发。</summary>
    public event Action<string>? ZoomToFeatureRequested;

    /// <summary>请求在地图上高亮指定要素时触发。</summary>
    public event Action<Ulid>? HighlightFeatureRequested;

    /// <summary>属性被编辑后触发（featureId, key, newValue）。</summary>
    public event Action<Ulid, string, string>? AttributeEdited;

    // ─── Load Features ───────────────────────────

    /// <summary>加载指定图层的所有要素。</summary>
    [RelayCommand]
    private async Task LoadFeaturesAsync(Ulid? layerId)
    {
        if (layerId is null)
        {
            Rows.Clear();
            ColumnNames.Clear();
            PanelTitle = "属性表";
            ActiveLayerId = null;
            return;
        }

        IsLoading = true;
        ActiveLayerId = layerId;

        try
        {
            var layer = await _repository.GetLayerByIdAsync(layerId.Value);
            if (layer is null)
            {
                Rows.Clear();
                PanelTitle = "属性表 - 图层不存在";
                return;
            }

            PanelTitle = $"属性表 - {layer.Name} ({layer.Features.Count} 个要素)";

            // 收集所有属性列名
            var allKeys = new HashSet<string>();
            foreach (var feature in layer.Features)
            {
                if (feature.PropertiesJson is not null)
                {
                    try
                    {
                        using var doc = JsonDocument.Parse(feature.PropertiesJson);
                        foreach (var prop in doc.RootElement.EnumerateObject())
                        {
                            allKeys.Add(prop.Name);
                        }
                    }
                    catch
                    {
                        // 跳过无效 JSON
                    }
                }
            }

            ColumnNames.Clear();
            foreach (var key in allKeys.OrderBy(k => k))
            {
                ColumnNames.Add(key);
            }

            // 构建行数据
            Rows.Clear();
            foreach (var feature in layer.Features)
            {
                var row = new AttributeRowViewModel
                {
                    FeatureId = feature.Id,
                    Name = feature.Name ?? feature.Id.ToString()[..8],
                    GeometryType = feature.GeometryType,
                    GeometryWkt = feature.GeometryWkt
                };

                // 解析属性
                if (feature.PropertiesJson is not null)
                {
                    try
                    {
                        using var doc = JsonDocument.Parse(feature.PropertiesJson);
                        foreach (var key in allKeys.OrderBy(k => k))
                        {
                            var value = doc.RootElement.TryGetProperty(key, out var prop)
                                ? prop.ToString()
                                : string.Empty;
                            row.Attributes.Add(new AttributeEntry { Key = key, Value = value });
                        }
                    }
                    catch
                    {
                        foreach (var key in allKeys.OrderBy(k => k))
                        {
                            row.Attributes.Add(new AttributeEntry { Key = key, Value = string.Empty });
                        }
                    }
                }
                else
                {
                    foreach (var key in allKeys.OrderBy(k => k))
                    {
                        row.Attributes.Add(new AttributeEntry { Key = key, Value = string.Empty });
                    }
                }

                Rows.Add(row);
            }
        }
        catch (Exception ex)
        {
            PanelTitle = $"属性表 - 加载失败: {ex.Message}";
        }
        finally
        {
            IsLoading = false;
        }
    }

    // ─── Row Actions ─────────────────────────────

    /// <summary>双击行时缩放到对应要素。</summary>
    [RelayCommand]
    private void ZoomToFeature(AttributeRowViewModel? row)
    {
        if (row is null) return;
        ZoomToFeatureRequested?.Invoke(row.GeometryWkt);
    }

    /// <summary>选中行时高亮对应要素。</summary>
    partial void OnSelectedRowChanged(AttributeRowViewModel? value)
    {
        if (value is not null)
        {
            HighlightFeatureRequested?.Invoke(value.FeatureId);
        }
    }

    /// <summary>切换面板展开/折叠。</summary>
    [RelayCommand]
    private void ToggleExpanded()
    {
        IsExpanded = !IsExpanded;
    }

    /// <summary>保存属性编辑。</summary>
    [RelayCommand]
    private async Task SaveAttributeAsync(AttributeEntry? entry)
    {
        if (SelectedRow is null || entry is null) return;

        // 重建 PropertiesJson
        var props = new Dictionary<string, string>();
        foreach (var attr in SelectedRow.Attributes)
        {
            props[attr.Key] = attr.Value;
        }

        var json = JsonSerializer.Serialize(props);

        var features = await _repository.GetFeaturesByLayerAsync(
            ActiveLayerId ?? default);
        var feature = features.FirstOrDefault(f => f.Id == SelectedRow.FeatureId);

        if (feature is not null)
        {
            feature.PropertiesJson = json;
            feature.UpdatedAt = DateTimeOffset.UtcNow;
            await _repository.UpdateFeatureAsync(feature);
            AttributeEdited?.Invoke(feature.Id, entry.Key, entry.Value);
        }
    }

    /// <summary>刷新当前图层。</summary>
    [RelayCommand]
    private async Task RefreshAsync()
    {
        if (ActiveLayerId is not null)
        {
            await LoadFeaturesAsync(ActiveLayerId);
        }
    }
}
```

- [ ] 6.2 Verify build:
```bash
dotnet build src/Wif.App/Wif.App.csproj
```

---

## Task 7: AttributeTablePanel (AXAML + Code-Behind)

**Files:** `src/Wif.App/Views/AttributeTablePanel.axaml`, `src/Wif.App/Views/AttributeTablePanel.axaml.cs`

Bottom collapsible panel with DataGrid showing feature attributes for the selected layer.

- [ ] 7.1 Create `src/Wif.App/Views/AttributeTablePanel.axaml` with the complete content below:

```xml
<UserControl xmlns="https://github.com/avaloniaui"
             xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml"
             xmlns:vm="clr-namespace:Wif.App.ViewModels"
             x:Class="Wif.App.Views.AttributeTablePanel"
             x:CompileBindings="False">

  <UserControl.Styles>
    <Style Selector="Border.attr-panel-header">
      <Setter Property="Background" Value="{DynamicResource SystemControlBackgroundChromeMediumBrush}" />
      <Setter Property="Padding" Value="8,4" />
      <Setter Property="Cursor" Value="Hand" />
    </Style>
    <Style Selector="DataGrid">
      <Setter Property="GridLinesVisibility" Value="All" />
      <Setter Property="BorderThickness" Value="1" />
      <Setter Property="BorderBrush" Value="{DynamicResource SystemControlForegroundBaseLowBrush}" />
      <Setter Property="RowBackground" Value="{DynamicResource SystemControlBackgroundAltHighBrush}" />
      <Setter Property="AlternatingRowBackground" Value="{DynamicResource SystemControlBackgroundListLowBrush}" />
    </Style>
    <Style Selector="Button.attr-btn">
      <Setter Property="Padding" Value="6,2" />
      <Setter Property="Margin" Value="2,0" />
      <Setter Property="CornerRadius" Value="4" />
      <Setter Property="FontSize" Value="12" />
    </Style>
  </UserControl.Styles>

  <Border BorderThickness="0,1,0,0"
          BorderBrush="{DynamicResource SystemControlForegroundBaseLowBrush}">
    <DockPanel>

      <!-- Header: title + collapse toggle -->
      <Border Classes="attr-panel-header" DockPanel.Dock="Top">
        <Grid ColumnDefinitions="*,Auto,Auto,Auto">
          <TextBlock Grid.Column="0"
                     Text="{Binding PanelTitle}"
                     VerticalAlignment="Center"
                     FontWeight="SemiBold"
                     FontSize="12" />

          <Button Grid.Column="1"
                  Classes="attr-btn"
                  Command="{Binding RefreshCommand}"
                  ToolTip.Tip="刷新">
            R
          </Button>

          <Button Grid.Column="2"
                  Classes="attr-btn"
                  Command="{Binding ToggleExpandedCommand}"
                  Content="{Binding IsExpanded, Converter={x:Static BoolConverters.ToString}}"
                  ToolTip.Tip="展开/折叠" />

          <!-- Simple expand/collapse indicator -->
          <TextBlock Grid.Column="3"
                     Text="{Binding IsExpanded, 
                       StringFormat={}{0}}"
                     VerticalAlignment="Center"
                     Margin="4,0"
                     FontSize="10"
                     IsVisible="False" />
        </Grid>
      </Border>

      <!-- Content: DataGrid (visible when expanded) -->
      <DataGrid DockPanel.Dock="Top"
                IsVisible="{Binding IsExpanded}"
                ItemsSource="{Binding Rows}"
                SelectedItem="{Binding SelectedRow}"
                AutoGenerateColumns="False"
                CanUserReorderColumns="True"
                CanUserResizeColumns="True"
                CanUserSortColumns="True"
                SelectionMode="Single"
                IsReadOnly="False"
                MaxHeight="250"
                MinHeight="100"
                Name="AttributeDataGrid">

        <DataGrid.Columns>
          <DataGridTextColumn Header="名称"
                              Binding="{Binding Name}"
                              Width="120"
                              IsReadOnly="True" />
          <DataGridTextColumn Header="几何类型"
                              Binding="{Binding GeometryType}"
                              Width="80"
                              IsReadOnly="True" />
          <DataGridTextColumn Header="要素 ID"
                              Binding="{Binding FeatureId}"
                              Width="100"
                              IsReadOnly="True" />
        </DataGrid.Columns>
      </DataGrid>

      <!-- Loading indicator -->
      <ProgressBar DockPanel.Dock="Top"
                   IsIndeterminate="True"
                   IsVisible="{Binding IsLoading}"
                   Height="2" />

    </DockPanel>
  </Border>
</UserControl>
```

- [ ] 7.2 Create `src/Wif.App/Views/AttributeTablePanel.axaml.cs` with the complete content below:

```csharp
using Avalonia.Controls;
using Avalonia.Input;
using Wif.App.ViewModels;

namespace Wif.App.Views;

public partial class AttributeTablePanel : UserControl
{
    public AttributeTablePanel()
    {
        InitializeComponent();
    }

    /// <summary>
    /// DataGrid 双击行时触发缩放到要素。
    /// </summary>
    protected override void OnLoaded(Avalonia.Interactivity.RoutedEventArgs e)
    {
        base.OnLoaded(e);

        var dataGrid = this.FindControl<DataGrid>("AttributeDataGrid");
        if (dataGrid is not null)
        {
            dataGrid.DoubleTapped += OnDataGridDoubleTapped;
        }
    }

    private void OnDataGridDoubleTapped(object? sender, TappedEventArgs e)
    {
        if (DataContext is AttributeTableViewModel vm && vm.SelectedRow is not null)
        {
            vm.ZoomToFeatureCommand.Execute(vm.SelectedRow);
        }
    }
}
```

- [ ] 7.3 Verify build:
```bash
dotnet build src/Wif.App/Wif.App.csproj
```

---

## Integration Notes (for Assembler Agent)

The assembler agent is responsible for wiring these components into `GisView.axaml`, `GisViewModel.cs`, and `DependencyInjection.cs`. The following integration points are expected:

### ToolManager Registration
```csharp
// In DependencyInjection.cs
services.AddSingleton<ToolManager>();
services.AddTransient<AttributeTableViewModel>();
```

### GisViewModel Integration
The assembler should add to `GisViewModel`:
1. A `ToolManager` property
2. Commands for `SetSelectTool`, `SetDrawTool`, `SetEditTool`, `Undo`, `Redo`
3. A `DrawGeometryType` property to switch between Point/LineString/Polygon
4. Forward MapControl pointer events to `ToolManager.Handle*()` methods
5. Create a `MemoryLayer` for tool preview geometries (rubber-band, vertex handles)
6. Wire `AttributeTableViewModel.ZoomToFeatureRequested` to `Map.Navigator.NavigateTo()`

### GisView.axaml Integration
The assembler should add to the view:
1. Tool buttons in the map toolbar (Select/Draw/Edit)
2. Draw mode toggle (Point/Line/Polygon)
3. Undo/Redo buttons
4. `<views:AttributeTablePanel>` in the bottom row
5. Wire MapControl pointer events to ViewModel commands

### Tool Construction
Tools need lambdas/delegates for repository operations:
```csharp
var drawingTool = new DrawingTool(
    geometryService,
    addFunc: async (f, ct) => { await repo.AddFeatureAsync(f, ct); },
    deleteFunc: async (id, ct) => { await repo.DeleteFeatureAsync(id, ct); },
    activeLayerIdFunc: () => SelectedLayer?.Id ?? default);

var selectionTool = new SelectionTool(
    geometryService,
    featuresFunc: () => _currentLayerFeatures);

var editingTool = new EditingTool(
    geometryService,
    featuresFunc: () => _currentLayerFeatures,
    updateFunc: async (id, wkt, ct) => { /* update feature geometry */ });

toolManager.RegisterTool(selectionTool);
toolManager.RegisterTool(drawingTool);
toolManager.RegisterTool(editingTool);
```

---

## Build Verification

After all tasks are complete, run:

```bash
dotnet build src/Wif.Gis/Wif.Gis.csproj
dotnet build src/Wif.App/Wif.App.csproj
dotnet build Wif.sln
```

All three must compile without errors.

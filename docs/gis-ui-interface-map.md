# GIS 模块 UI 接口映射文档

### 状态定义

- **已完成**：入口、后端行为、持久化/回显闭环齐全
- **半完成**：有真实逻辑，但缺关键交互、集成或验证
- **占位**：只有命令入口、状态提示或说明性实现
- **缺失**：文档定义但代码不存在
- **失真**：文档状态与代码证据不一致
- **计划中**：文档定义但尚未开始实现

> 状态定义与 `docs/superpowers/platform-capability-matrix.md` 保持一致。
> 最近审计日期：2026-04-03

## 1. UI 结构图

```
+-----------------------------------------------------------------------+
|  Menu Bar                                                              |
|  工程 | 编辑 | 视图 | 图层 | 矢量 | 栅格 | 数据处理                       |
+-----------------------------------------------------------------------+
|  Toolbar Row 1: 文件操作 | 导入导出 | 导航 | 底图选择 | 刷新             |
+-----------------------------------------------------------------------+
|  Toolbar Row 2: 选择/编辑 | 绘制 | 要素编辑 | 撤销重做 | 测量 | 属性     |
+-------+---+-----------+----------------------------------------------+
|       |   |           |                                              |
| Left  | S |           |                                              |
| Panel | p |           |         Map Area                             |
|       | l |           |         (Mapsui MapControl)                  |
| Tab:  | i |           |                                              |
| 浏览器 | t |           |                                              |
| 图层  | t |           |                                              |
|       | e |           |                                              |
|       | r |           |                                              |
+-------+---+-----------+----------------------------------------------+
|  Attribute Table (collapsible DataGrid)                               |
+-----------------------------------------------------------------------+
|  Status Bar: 状态消息 | 坐标 | 比例尺 | CRS                           |
+-----------------------------------------------------------------------+
```

## 2. 文件结构

| 文件 | 职责 |
|------|------|
| `Views/GisView.axaml` | 主视图：菜单栏 + 工具栏 + 左面板/地图布局 + 状态栏 |
| `Views/GisView.axaml.cs` | Code-behind：绑定 MapControl、转发指针事件 |
| `Views/GisBrowserPanel.axaml` | 浏览器面板子视图：数据源树 |
| `Views/GisBrowserPanel.axaml.cs` | Code-behind (空) |
| `Views/GisLayerPanel.axaml` | 图层面板子视图：图层列表 + 工具栏 |
| `Views/GisLayerPanel.axaml.cs` | Code-behind (空) |
| `Views/AttributeTablePanel.axaml` | 属性表面板：可折叠 DataGrid |
| `ViewModels/GisViewModel.cs` | 主 ViewModel：所有命令、属性、状态 |
| `ViewModels/AttributeTableViewModel.cs` | 属性表 ViewModel |

## 3. 菜单栏接口映射

### 3.1 工程菜单

| 菜单项 | ViewModel Command | 后端 Service | 状态 |
|--------|------------------|-------------|------|
| 新建工程 | `NewProjectCommand` | - | 计划中 |
| 打开工程 | `OpenProjectCommand` | - | 计划中 |
| 保存工程 | `SaveProjectCommand` | - | 计划中 |
| 工程另存为 | `SaveProjectAsCommand` | - | 计划中 |

### 3.2 编辑菜单

| 菜单项 | ViewModel Command | 后端 Service | 状态 |
|--------|------------------|-------------|------|
| 撤销 | `UndoCommand` | `ToolManager.UndoAsync()` | 已完成 |
| 重做 | `RedoCommand` | `ToolManager.RedoAsync()` | 已完成 |
| 复制要素 | `EditCopyCommand` | `ToolManager` | 已完成 |
| 粘贴要素 | `EditPasteCommand` | `ToolManager` | 已完成 |
| 删除选中 | `EditDeleteCommand` | `ToolManager` | 已完成 |
| 全选 | `SelectAllCommand` | `ToolManager` | 已完成 |
| 取消全选 | `DeselectAllCommand` | `ToolManager` | 已完成 |

### 3.3 视图菜单

| 菜单项 | ViewModel Command | 后端 Service | 状态 |
|--------|------------------|-------------|------|
| 浏览器面板 | `ToggleBrowserPanelCommand` | - (UI toggle) | 已完成 |
| 图层面板 | `ToggleLayerPanelCommand` | - (UI toggle) | 已完成 |
| 属性表 | `ToggleAttributeTablePanelCommand` | `AttributeTableViewModel` | 已完成 |
| 放大 | `ZoomInCommand` | `Map.Navigator.ZoomIn()` | 已完成 |
| 缩小 | `ZoomOutCommand` | `Map.Navigator.ZoomOut()` | 已完成 |
| 全图范围 | `ZoomToAllCommand` | `MapService.ZoomToAll()` | 已完成 |
| 缩放到图层 | `ZoomToLayerCommand` | `MapService.ZoomToLayer()` | 已完成 |
| 刷新地图 | `RefreshMapCommand` | `Map.Refresh()` | 已完成 |

### 3.4 图层菜单

| 菜单项 | ViewModel Command | 后端 Service | 状态 |
|--------|------------------|-------------|------|
| 新建空图层 | `AddLayerCommand` | `IGisLayerRepository.CreateLayerAsync()` | 已完成 |
| 添加矢量图层 | `AddVectorLayerCommand` | `GisFileService` | 已完成 |
| 添加栅格图层 | `AddRasterLayerCommand` | `RasterLayerService` | 已完成 |
| 添加 WMS/WMTS | `AddWmsLayerCommand` | `TileSourceFactory` | 半完成 |
| 添加 XYZ 瓦片 | `AddXyzLayerCommand` | `TileSourceFactory` | 半完成 |
| 导入空间数据 | `ImportFileCommand` | `GisFileService.ImportFile()` | 已完成 |
| 导入 Shapefile | `ImportShapefileCommand` | `GisFileService.ImportShapefile()` | 已完成 |
| 导入 GeoJSON | `ImportGeoJsonCommand` | `GisFileService.ImportGeoJson()` | 已完成 |
| 导出 GeoJSON | `ExportGeoJsonCommand` | `GisFileService.ExportGeoJson()` | 已完成 |
| 切换编辑模式 | `ToggleEditingCommand` | `ToolManager` | 已完成 |
| 保存编辑 | `SaveEditsCommand` | `IGisLayerRepository` | 已完成 |
| 删除图层 | `RemoveLayerCommand` | `IGisLayerRepository.DeleteLayerAsync()` | 已完成 |
| 图层属性 | `LayerPropertiesCommand` | `IGisLayerRepository` | 半完成 |

### 3.5 矢量菜单

| 菜单项 | ViewModel Command | 后端 Service | 状态 |
|--------|------------------|-------------|------|
| 缓冲区 | `VectorBufferCommand` | `GeometryService.Buffer()` | 已完成 |
| 相交 | `VectorIntersectCommand` | `GeometryService.Intersection()` | 已完成 |
| 合并 | `VectorUnionCommand` | `GeometryService.Union()` | 已完成 |
| 差异 | `VectorDifferenceCommand` | `GeometryService.Difference()` | 已完成 |
| 裁剪 | `VectorClipCommand` | `GeometryService.Clip()` | 已完成 |
| 融合 | `VectorDissolveCommand` | `GeometryService.UnaryUnion()` | 已完成 |
| 面积计算 | `CalculateAreaCommand` | `GeometryService.Area()` | 半完成 |
| 距离计算 | `CalculateDistanceCommand` | `GeometryService.Distance()` | 半完成 |

### 3.6 栅格菜单

| 菜单项 | ViewModel Command | 后端 Service | 状态 |
|--------|------------------|-------------|------|
| 栅格信息 | `RasterInfoCommand` | `RasterLayerService` | 已完成 |
| 栅格计算器 | `RasterCalculatorCommand` | `RasterLayerService` | 半完成 |
| 重投影 | `RasterReprojectCommand` | `CoordinateTransformer` | 占位 |
| 裁剪 | `RasterClipCommand` | `RasterLayerService` | 占位 |

### 3.7 数据处理菜单

| 菜单项 | ViewModel Command | 后端 Service | 状态 |
|--------|------------------|-------------|------|
| 工具箱 | `ProcessingToolboxCommand` | - | 计划中 |
| 处理历史 | `ProcessingHistoryCommand` | - | 计划中 |
| 批处理 | `ProcessingBatchCommand` | - | 计划中 |

## 4. 工具栏接口映射

### 4.1 工具栏第一行（文件 + 导航）

| 按钮 | ViewModel Command | 后端 Service | 状态 |
|------|------------------|-------------|------|
| 新建工程 | `NewProjectCommand` | - | 计划中 |
| 打开工程 | `OpenProjectCommand` | - | 计划中 |
| 保存工程 | `SaveProjectCommand` | - | 计划中 |
| 导入空间数据 | `ImportFileCommand` | `GisFileService` | 已完成 |
| 导出 GeoJSON | `ExportGeoJsonCommand` | `GisFileService` | 已完成 |
| 平移 | `PanToolCommand` | `ToolManager` | 已完成 |
| 放大 | `ZoomInCommand` | `Map.Navigator` | 已完成 |
| 缩小 | `ZoomOutCommand` | `Map.Navigator` | 已完成 |
| 全图范围 | `ZoomToAllCommand` | `MapService` | 已完成 |
| 缩放到选中 | `ZoomToSelectedCommand` | `MapService` | 已完成 |
| 前一视图 | `ZoomPreviousCommand` | `ViewportHistory` | 已完成 |
| 后一视图 | `ZoomNextCommand` | `ViewportHistory` | 已完成 |
| 底图选择 | `SelectedBasemapIndex` (属性) | `MapService.SetOsmBasemap()` 等 | 半完成 |
| 刷新地图 | `RefreshMapCommand` | `Map.Refresh()` | 已完成 |

### 4.2 工具栏第二行（编辑 + 测量）

| 按钮 | ViewModel Command | 后端 Service | 状态 |
|------|------------------|-------------|------|
| 选择工具 | `SetSelectToolCommand` | `ToolManager` | 已完成 |
| 编辑节点 | `SetEditToolCommand` | `ToolManager` | 已完成 |
| 画点 | `SetDrawPointCommand` | `ToolManager` + `DrawingTool` | 已完成 |
| 画线 | `SetDrawLineCommand` | `ToolManager` + `DrawingTool` | 已完成 |
| 画面 | `SetDrawPolygonCommand` | `ToolManager` + `DrawingTool` | 已完成 |
| 移动要素 | `MoveFeatureCommand` | `ToolManager` | 已完成 |
| 拆分要素 | `SplitFeatureCommand` | `ToolManager` | 已完成 |
| 合并要素 | `MergeFeaturesCommand` | `ToolManager` | 已完成 |
| 撤销 | `UndoCommand` | `ToolManager` | 已完成 |
| 重做 | `RedoCommand` | `ToolManager` | 已完成 |
| 测距 | `SetMeasureDistanceCommand` | `MeasureTool` | 已完成 |
| 测面 | `SetMeasureAreaCommand` | `MeasureTool` | 已完成 |
| 识别要素 | `IdentifyFeatureCommand` | `ToolManager` | 半完成 |
| 属性表 | `ToggleAttributeTableCommand` | `AttributeTableViewModel` | 已完成 |
| 字段计算器 | `OpenFieldCalculatorCommand` | `AttributeTableViewModel` | 半完成 |
| 切换标注 | `ToggleLabelsCommand` | `FeatureStyleProvider` | 占位 |

## 5. 左侧面板接口映射

### 5.1 浏览器面板

| UI 元素 | ViewModel Command/Property | 后端 Service | 状态 |
|---------|---------------------------|-------------|------|
| 数据源树 | `BrowserNodes` (集合) | - | 已完成 (静态) |
| 选中节点 | `SelectedBrowserNode` | - | 已完成 |
| 添加目录 | `BrowserAddDirectoryCommand` | `PopulateBrowserNodes()` | 已完成 |
| 移除条目 | `BrowserRemoveEntryCommand` | `PopulateBrowserNodes()` | 已完成 |
| 刷新 | `RefreshBrowserCommand` | `PopulateBrowserNodes()` | 已完成 |
| 过滤 | `BrowserFilterCommand` | `PopulateBrowserNodes()` | 已完成 |

### 5.2 图层面板

| UI 元素 | ViewModel Command/Property | 后端 Service | 状态 |
|---------|---------------------------|-------------|------|
| 图层列表 | `Layers` (集合) | `IGisLayerRepository` | 已完成 |
| 选中图层 | `SelectedLayer` | - | 已完成 |
| 新建图层 | `AddLayerCommand` | `IGisLayerRepository` | 已完成 |
| 删除图层 | `RemoveLayerCommand` | `IGisLayerRepository` | 已完成 |
| 上移图层 | `MoveLayerUpCommand` | - (UI only) | 已完成 |
| 下移图层 | `MoveLayerDownCommand` | - (UI only) | 已完成 |
| 切换编辑 | `ToggleEditingCommand` | `ToolManager` | 已完成 |
| 缩放到图层 | `ZoomToLayerCommand` | `MapService.ZoomToLayer()` | 已完成 |
| 图层属性 | `LayerPropertiesCommand` | `IGisLayerRepository` | 半完成 |
| 可见性切换 | `IsVisible` (GisLayerItem) | `MapService.SetLayerVisibility()` | 已完成 |

## 6. 状态栏接口映射

| UI 元素 | ViewModel Property | 后端 Service | 状态 |
|---------|-------------------|-------------|------|
| 状态消息 | `StatusMessage` | - | 已完成 |
| 坐标显示 | `CursorPosition` | `SphericalMercator.ToLonLat()` | 已完成 |
| 比例尺 | `MapScale` | `Map.Navigator.Viewport` | 已完成 |
| CRS | `MapCrs` | `Map.CRS` | 已完成 |
| 放大倍数 | `MapMagnification` | `Map.Navigator.Viewport` | 已完成 |
| 旋转角度 | `MapRotation` | `Map.Navigator.Viewport` | 已完成 |

## 7. 实现状态汇总

| 状态 | 数量 | 说明 |
|------|------|------|
| 已完成 | 76 | 入口 + 后端 + 持久化/回显闭环齐全 |
| 半完成 | 10 | 有真实逻辑，缺关键交互或集成（WMS/WMTS + XYZ 瓦片 + 底图选择 + 面积/距离计算 + 栅格计算器 + 图层属性x2 + 识别要素 + 字段计算器） |
| 占位 | 3 | 命令入口或状态提示（栅格重投影 + 栅格裁剪 + 切换标注） |
| 计划中 | 10 | 文档定义但尚未开始实现（工程管理4 + 数据处理3 + 工具栏工程按钮3） |

> 审计方法：以 `GisViewModel.cs` 代码实现为真相源，凡 `StatusMessage = "功能开发中"` / `待集成` / 纯文本信息输出 / 无持久化的功能，均不标记为"已完成"。

## 8. 扩展指南：如何为 stub command 接入真实后端

### 步骤

1. **在 `GisViewModel.cs` 中找到对应的 stub command**
   - 搜索 `功能开发中` 即可定位所有 stub
   - 每个 stub 都是一个 `[RelayCommand]` 方法

2. **替换 stub 实现**
   ```csharp
   // 替换前 (stub):
   [RelayCommand]
   private void VectorBuffer() => StatusMessage = "功能开发中: 缓冲区分析";

   // 替换后 (实现):
   [RelayCommand]
   private async Task VectorBufferAsync()
   {
       if (SelectedLayer is null) { StatusMessage = "请先选择图层"; return; }
       try
       {
           var layer = await _repository.GetLayerByIdAsync(SelectedLayer.Id);
           // 调用 GeometryService 执行缓冲区分析
           // ...
           StatusMessage = "缓冲区分析完成";
       }
       catch (Exception ex)
       {
           StatusMessage = $"缓冲区分析失败: {ex.Message}";
       }
   }
   ```

3. **如果需要新的后端服务**
   - 在 `Wif.Gis/Services/` 中创建或扩展服务
   - 在 `Wif.Core/Services/DependencyInjection.cs` 中注册
   - 在 `GisViewModel` 构造函数中注入

4. **如果需要新的对话框**
   - 在 `Views/` 中创建新的 `Window` 或 `UserControl`
   - 在 `ViewModels/` 中创建对应的 ViewModel
   - 从 command 中打开对话框

### 后端服务参考

| 服务 | 文件 | 职责 |
|------|------|------|
| `MapService` | `Wif.Gis/Services/MapService.cs` | 地图实例管理、底图切换、图层渲染 |
| `GisFileService` | `Wif.Gis/Services/GisFileService.cs` | 矢量数据导入导出 (GDAL) |
| `GeometryService` | `Wif.Gis/Services/GeometryService.cs` | NTS 空间分析 (面积、距离、WKT) |
| `RasterLayerService` | `Wif.Gis/Services/RasterLayerService.cs` | 栅格数据加载 (GDAL) |
| `GdalDataService` | `Wif.Gis/Services/GdalDataService.cs` | GDAL 底层数据读取 |
| `CoordinateTransformer` | `Wif.Gis/Projection/CoordinateTransformer.cs` | CRS 坐标变换 (GDAL PROJ) |
| `TileSourceFactory` | `Wif.Gis/Services/TileSourceFactory.cs` | 瓦片源工厂 (OSM/天地图/XYZ) |
| `ToolManager` | `Wif.Gis/Editing/ToolManager.cs` | 地图工具管理 (选择/绘制/编辑) |
| `FeatureStyleProvider` | `Wif.Gis/Rendering/FeatureStyleProvider.cs` | 矢量要素样式 |

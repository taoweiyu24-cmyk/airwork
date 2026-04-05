# GIS 功能完善设计文档

> 日期: 2026-04-03
> 状态: 已审批

## 概述

完善 WorkItemFlow 的 GIS 模块，从当前的空壳状态（空白 Canvas + 桩函数）升级为功能完整的空间数据管理工具，并与工作项系统深度集成。

采用**自底向上4阶段**交付：P1 地图基础 → P2 交互编辑 → P3 空间分析 → P4 深度集成。

## 现状

### 已有

- **Domain**: `GisLayer` + `GisFeature` 实体，`IGisLayerRepository` 接口
- **Data**: SQLite 表 + EF Core 仓库 + 迁移
- **Wif.Gis**: `GeometryService`（NTS 空间计算 12 种操作）、`GisFileService`（GeoJSON/Shapefile 导入导出）、`GisToolRegistry`（AI 工具定义）
- **UI**: 3 列布局（图层面板 + 空白 Canvas + 工具面板），图层 CRUD，导入导出命令
- **Module**: `GisModule`（空壳，ConfigureServices 为空）

### 缺失

1. 地图渲染（中心区域是空白 Canvas）
2. 缓冲区/交集分析命令为桩函数
3. 无坐标系/投影处理
4. 无要素编辑（绘制/修改/删除）
5. 无属性表视图
6. 无底图/瓦片图层支持
7. 无 WorkItem 地理关联

## 技术选型

| 组件 | 选择 | 理由 |
|------|------|------|
| 地图渲染 | **Mapsui.Avalonia** | 原生 Avalonia 控件，内置瓦片/矢量渲染，与 NTS 无缝集成 |
| 底图源 | **OSM + 天地图 + 自定义 XYZ** | OSM 免费无 Key，天地图服务国内地图，自定义 XYZ 最灵活 |
| 坐标投影 | **ProjNET** | 纯 .NET 实现，无本地库依赖 |
| 几何引擎 | **NetTopologySuite**（已有） | 继续使用，新增投影转换配合 ProjNET |

## P1 — 地图基础

### 目标

将空白 Canvas 替换为 Mapsui MapControl，实现底图显示、矢量图层渲染和基础交互。

### 文件清单

| 文件 | 类型 | 职责 |
|------|------|------|
| `Wif.Gis/Services/MapService.cs` | 新增 | 管理 Mapsui Map 实例、底图切换、GisLayer → Mapsui Layer 转换 |
| `Wif.Gis/Services/TileSourceFactory.cs` | 新增 | 创建 OSM / 天地图 / 自定义 XYZ 瓦片源 |
| `Wif.Gis/Rendering/FeatureStyleProvider.cs` | 新增 | 将 GisLayer 样式属性（FillColor/StrokeColor/StrokeWidth）映射为 Mapsui Style |
| `Wif.App/Views/GisView.axaml` | 修改 | Canvas → Mapsui MapControl，添加底图切换下拉框 |
| `Wif.App/ViewModels/GisViewModel.cs` | 修改 | 新增 Map 属性、底图切换命令、图层→地图同步逻辑 |
| `Wif.Domain/Entities/GisLayer.cs` | 修改 | 新增 SourceType 枚举（矢量/瓦片） |
| `Wif.Gis/Wif.Gis.csproj` | 修改 | 添加 Mapsui.Avalonia NuGet 包 |

### 关键设计

- **坐标系统**: 内部统一 EPSG:4326 (WGS84)，Mapsui 显示用 SphericalMercator (EPSG:3857)，MapService 负责转换
- **矢量渲染管线**: GisFeature.GeometryWkt → NTS Geometry → Mapsui GeometryFeature + Style
- **底图切换**: TileSourceFactory 封装三种瓦片源，天地图需要 WMTS 格式 + API Key（在设置中配置）
- **状态栏**: 实时显示鼠标经纬度 + 当前底图名称 + 缩放级别

## P2 — 交互编辑

### 目标

实现要素绘制（点/线/面）、选中编辑（顶点拖拽/整体移动）、删除，以及底部属性表面板。

### 文件清单

| 文件 | 类型 | 职责 |
|------|------|------|
| `Wif.Gis/Editing/IMapTool.cs` | 新增 | 工具接口：统一处理 PointerPressed/Moved/Released/KeyDown |
| `Wif.Gis/Editing/ToolManager.cs` | 新增 | 管理活动工具、工具切换、撤销/重做栈（Command 模式） |
| `Wif.Gis/Editing/DrawingTool.cs` | 新增 | 绘制工具：点击添加顶点，双击/Enter 完成，Escape 取消，支持 Point/LineString/Polygon |
| `Wif.Gis/Editing/EditingTool.cs` | 新增 | 编辑工具：顶点手柄拖拽修改形状，整体移动 |
| `Wif.Gis/Editing/SelectionTool.cs` | 新增 | 选择工具：点击选中（命中检测），Shift+点击多选，拖拽框选 |
| `Wif.App/Views/AttributeTablePanel.axaml` | 新增 | 属性表面板：DataGrid 显示要素属性，单元格可编辑 |
| `Wif.App/ViewModels/AttributeTableViewModel.cs` | 新增 | 加载选中图层要素，双击行定位地图，编辑回写 |
| `Wif.App/Views/GisView.axaml` | 修改 | 添加绘图工具栏（顶部） + 底部可折叠属性表 |
| `Wif.App/ViewModels/GisViewModel.cs` | 修改 | 工具切换命令、要素选中事件、撤销/重做命令 |

### 工具系统设计

- **IMapTool**: `OnPointerPressed` / `OnPointerMoved` / `OnPointerReleased` / `OnKeyDown`，每个工具独立实现交互逻辑
- **ToolManager**: 维护当前工具实例，快捷键切换（S 选择 / P 画点 / L 画线 / G 画面 / E 编辑），撤销/重做用 Command 模式
- **DrawingTool**: 点击添加顶点，鼠标移动显示橡皮筋线预览，双击或 Enter 完成绘制
- **EditingTool**: 选中要素后显示白色方形顶点手柄，拖拽手柄修改几何形状，拖拽要素本体整体移动
- **SelectionTool**: 点击命中检测选中要素（高亮），Shift+点击追加选中，拖拽矩形框选

### 属性表

- 底部可折叠面板，显示当前图层所有要素
- DataGrid 列动态生成自 PropertiesJson
- 选中行同步高亮地图上对应要素
- 双击行缩放定位到该要素

## P3 — 空间分析

### 目标

将现有桩函数替换为真正的空间分析实现，增加量测工具和坐标系管理。

### 文件清单

| 文件 | 类型 | 职责 |
|------|------|------|
| `Wif.Gis/Analysis/SpatialAnalysisService.cs` | 新增 | 统一分析入口：缓冲区/交集/并集/差集，返回 AnalysisResult |
| `Wif.Gis/Analysis/MeasurementService.cs` | 新增 | 量测：距离/面积，支持大地线（Vincenty）和投影平面两种模式 |
| `Wif.Gis/Analysis/AnalysisResult.cs` | 新增 | 结果模型：结果图层 + 统计摘要（面积/数量/耗时） |
| `Wif.Gis/Projection/CrsManager.cs` | 新增 | CRS 管理：内置常用坐标系，EPSG 代码查找，自定义 WKT |
| `Wif.Gis/Projection/CoordinateTransformer.cs` | 新增 | 坐标转换：ProjNET 在不同 CRS 间转换几何体 |
| `Wif.Gis/Editing/MeasureTool.cs` | 新增 | 量测工具：地图交互式测距/测面，实时标注结果 |
| `Wif.App/Views/AnalysisDialog.axaml` | 新增 | 分析参数对话框：选择输入图层、设定参数、确认执行 |
| `Wif.App/ViewModels/AnalysisDialogViewModel.cs` | 新增 | 参数验证、调用 SpatialAnalysisService、结果写入新图层 |
| `Wif.Domain/Entities/GisLayer.cs` | 修改 | 新增 CrsCode 属性（默认 "EPSG:4326"） |
| `Wif.Gis/Wif.Gis.csproj` | 修改 | 添加 ProjNET NuGet 包 |
| `Wif.App/ViewModels/GisViewModel.cs` | 修改 | 分析命令改为弹对话框，新增量测工具切换、CRS 选择 |

### 支持的分析操作

| 操作 | 输入 | 输出 | 说明 |
|------|------|------|------|
| 缓冲区 | 1 个图层 + 距离 | 新图层 | 圆形/方形端点，可合并重叠 |
| 交集 | 2 个图层 | 新图层 | 两层重叠区域提取 |
| 并集 | 2 个图层 | 新图层 | 两层合并 |
| 差集 | 2 个图层 | 新图层 | A 减去 B |
| 距离量测 | 地图点击 | 地图标注 | 逐段距离 + 总距离，支持大地线 |
| 面积量测 | 地图点击 | 地图标注 | 面积 + 周长，支持大地线 |

### 坐标系管理

- **内置 CRS**: EPSG:4326 (WGS84)、EPSG:3857 (Web Mercator)、EPSG:4490 (CGCS2000)、常用 UTM 带
- **图层 CRS**: 每个 GisLayer 记录 CrsCode，导入时自动检测或手动指定
- **动态投影**: 显示和分析时自动转换到统一 CRS
- **量测精度**: 大地线模式（Vincenty 公式）保证量测精度

### 分析对话框

缓冲区分析为例：
- 选择输入图层（下拉框）
- 输入缓冲距离 + 单位（米/千米/度）
- 选择端点样式（圆形/方形/平头）
- 是否合并重叠区域
- 指定结果图层名
- 执行后：结果图层自动添加到图层列表，地图上高亮显示，状态栏显示统计

## P4 — 深度集成

### 目标

WorkItem 关联地理位置，GIS 视图显示工作项空间分布，支持双向导航。

### 数据模型变更

```
WorkItem (修改)
  + Longitude : double?      -- 经度
  + Latitude  : double?      -- 纬度
  + GisFeatureId : Ulid?     -- 关联 GIS 要素（可选）

GisFeature (修改)
  + WorkItemId : Ulid?       -- 反向关联工作项（可选）
```

关联是可选的：工作项可以没有坐标，GIS 要素可以不关联工作项。简单位置用 Longitude/Latitude，复杂空间关系用 GisFeatureId。

### 文件清单

| 文件 | 类型 | 职责 |
|------|------|------|
| `Wif.Domain/Entities/WorkItem.cs` | 修改 | 新增 Longitude/Latitude/GisFeatureId 字段 |
| `Wif.Domain/Entities/GisFeature.cs` | 修改 | 新增 WorkItemId 反向导航 |
| `Wif.Data/Migrations/WorkItemGeoMigration.cs` | 新增 | ALTER TABLE 添加列 + 索引 |
| `Wif.Gis/Services/WorkItemMapService.cs` | 新增 | 查询有坐标的工作项，生成地图要素图层，聚合/散点模式 |
| `Wif.Gis/Services/GeocodingService.cs` | 新增 | 地理编码：地址 → 坐标（可选，Nominatim 或天地图 API） |
| `Wif.Gis/Rendering/ClusterRenderer.cs` | 新增 | 聚合渲染：按缩放级别聚合点要素为气泡 |
| `Wif.App/Views/WorkItemMapPopup.axaml` | 新增 | 地图上工作项信息卡片（标题/状态/优先级/操作） |
| `Wif.App/ViewModels/GisViewModel.cs` | 修改 | 工作项图层管理、点击导航、聚合/散点切换 |
| `Wif.App/Views/WorkItemDetailView.axaml` | 修改 | 新增位置信息区：经纬度/小地图预览/"在地图上查看"/"从地图选点" |
| `Wif.App/ViewModels/WorkItemDetailViewModel.cs` | 修改 | 位置编辑、地图跳转命令 |

### 交互流程

1. **地图 → 工作项**: 点击标记 → 弹出信息卡片 → "查看详情"跳转 WorkItemDetailView
2. **工作项 → 地图**: 详情页"在地图上查看" → 切换 GIS 视图并定位
3. **设置位置**: 详情页"从地图选点" → GIS 视图拾取模式 → 点击地图 → 坐标回填
4. **关联要素**: GIS 视图选中要素 → 右键菜单"关联到工作项" → 工作项选择器
5. **聚合模式**: 低缩放级别自动聚合为气泡（显示数量），放大后展开
6. **筛选**: 按优先级/状态/负责人筛选，地图和侧边栏列表同步

### 工作项标记样式

- 紧急: 红色圆点 + "!"
- 一般: 橙色圆点 + "M"
- 低: 蓝色圆点 + "L"
- 已完成: 绿色圆点 + "✓"

## 新增 NuGet 依赖

| 包 | 阶段 | 用途 |
|----|------|------|
| `Mapsui.Avalonia` | P1 | Avalonia 地图控件 |
| `ProjNET` | P3 | 坐标投影转换 |

## Agent Team 并行实现

### 5 个 Agent，3 波执行

| Agent | 职责 | 负责文件数 | Wave |
|-------|------|-----------|------|
| **gis-map** | Mapsui 集成、MapService、瓦片源工厂、样式映射 | 5 | Wave 1 |
| **gis-editing** | 工具系统 IMapTool/ToolManager + 绘制/编辑/选择工具 + 属性表 | 7 | Wave 1 |
| **gis-analysis** | 空间分析服务 + 量测 + CRS 管理 + 投影转换 + 分析对话框 | 10 | Wave 1 |
| **gis-integration** | WorkItem 地理关联 + 迁移 + 聚合渲染 + 地理编码 + 信息卡片 | 9 | Wave 1 |
| **gis-assembler** | GisView/GisViewModel 总装 + DI 注册 + EF 配置 + 编译验证 | 7 | Wave 3 |

### 执行顺序

- **Wave 1**: gis-map / gis-editing / gis-analysis / gis-integration 并行，各自开发独立文件
- **Wave 2**: 各 Agent 完成自己负责的独立 UI 视图（仍可并行）
- **Wave 3**: gis-assembler 等所有人完成后，统一修改 GisView + GisViewModel + DI 注册 + 编译验证

### 文件冲突策略

- `GisLayer.cs`: gis-map 加 SourceType，gis-analysis 加 CrsCode → assembler 合并
- `Wif.Gis.csproj`: gis-map 加 Mapsui，gis-analysis 加 ProjNET → assembler 合并
- `GisView.axaml` + `GisViewModel.cs`: 仅 assembler 修改，避免并行冲突

## 非功能需求

- **性能**: 图层渲染支持 10,000+ 要素不卡顿（Mapsui 自带 LOD 和瓦片缓存）
- **离线**: 矢量数据完全离线，底图支持瓦片缓存
- **本地化**: 所有 UI 文本和状态消息使用中文

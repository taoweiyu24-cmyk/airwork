# Platform Phase 1 — 能力矩阵

> 此文档以代码证据为基准，记录平台各流程段能力的真实状态。
> 生成日期：2026-04-03

### 状态定义

- **已完成**：入口、后端行为、持久化/回显闭环齐全
- **半完成**：有真实逻辑，但缺关键交互、集成或验证
- **占位**：只有命令入口、状态提示或说明性实现
- **缺失**：文档定义但代码不存在
- **失真**：文档状态与代码证据不一致

---

## 1. 主流程能力

| 流程段 | 能力 | 状态 | 代码证据 | UI 证据 | 备注 |
|--------|------|------|----------|---------|------|
| 输入流 | 邮件同步 | 半完成 | `src/Wif.Mail/MailService.cs` | `src/Wif.App/Views/InboxView.axaml` | 账户接入、同步、读取、附件处理已具备，缺统一状态中心 |
| 输入流 | 邮件创建工作项 | 半完成 | `src/Wif.App/ViewModels/InboxViewModel.cs` | `src/Wif.App/Views/InboxView.axaml` | 可从邮件生成工作项，缺统一入口策略 |
| 输入流 | 文件导入 | 已完成 | `src/Wif.Core/Services/ImportService.cs` | `src/Wif.App/Views/ImportView.axaml` | 支持 PDF/DOCX/XLSX/PPTX，文档抽取后创建工作项 |
| 输入流 | 手工创建工作项 | 已完成 | `src/Wif.Core/Services/WorkItemService.cs` | `src/Wif.App/Views/WorkItemsView.axaml` | CRUD 完整，状态流转可用 |
| 分析 | AI 分析 | 半完成 | `src/Wif.Core/Services/AnalysisService.cs` | `src/Wif.App/Views/WorkItemDetailView.axaml` | 详情页可触发分析，缺入口级批量处理与策略体系 |
| 分析 | AI Provider 管理 | 半完成 | `src/Wif.AI/AiService.cs` | `src/Wif.App/Views/SettingsView.axaml` | Profile 管理可用，缺多策略配置 |
| 分析 | Egress Policy | 半完成 | `src/Wif.AI/ContextPackBuilder.cs` | 无独立 UI | 后端逻辑完整，Settings 页部分暴露 |
| 检索 | 基础搜索 | 已完成 | `src/Wif.Core/Services/SearchService.cs` | `src/Wif.App/MainWindow.axaml` | Repository 级搜索，前台有搜索框 |
| 检索 | 混合搜索 | 半完成 | `src/Wif.Search/HybridSearchService.cs` | `src/Wif.App/MainWindow.axaml` | FTS5 + DuckDB 向量搜索架构就绪，前台未全面启用 |
| 工作项 | 状态流转 | 已完成 | `src/Wif.Core/Services/WorkItemService.cs` | `src/Wif.App/Views/WorkItemsView.axaml` | 状态变更、批量操作已实现 |
| 工作项 | 标签管理 | 已完成 | `src/Wif.Core/Services/WorkItemService.cs` | `src/Wif.App/Views/WorkItemDetailView.axaml` | 标签 CRUD 完整 |
| 工作项 | 父子关系 | 半完成 | `src/Wif.App/ViewModels/WorkItemDetailViewModel.cs` | `src/Wif.App/Views/WorkItemDetailView.axaml` | UI 支持设置父项，缺树视图与关系导航 |
| 工作项 | 导出 | 已完成 | `src/Wif.Core/Services/ExportService.cs` | `src/Wif.App/Views/WorkItemDetailView.axaml` | 支持导出为 PDF/DOCX/XLSX/PPTX |
| 联系人 | 联系人管理 | 已完成 | `src/Wif.Core/Services/ContactService.cs` | `src/Wif.App/Views/ContactsView.axaml` | CRUD、搜索、邮件自动创建联系人 |
| 邮件 | 发送邮件 | 半完成 | `src/Wif.Mail/MailService.cs` | `src/Wif.App/Views/ComposeView.axaml` | SMTP 发送可用，缺富文本编辑与附件完整支持 |
| 邮件 | OAuth2 授权 | 半完成 | `src/Wif.Mail/OAuth2/` | `src/Wif.App/Views/SettingsView.axaml` | Google 和 Microsoft Provider 已实现，缺故障恢复 |
| 邮件 | IMAP Idle 推送 | 半完成 | `src/Wif.Mail/ImapIdleMonitor.cs` | 后台服务 | 推送通知存在，缺与前台状态中心联动 |
| 设置 | AI 配置 | 已完成 | `src/Wif.App/ViewModels/SettingsViewModel.cs` | `src/Wif.App/Views/SettingsView.axaml` | AI Profile CRUD、默认模型配置完整 |
| 设置 | 邮件账户配置 | 已完成 | `src/Wif.App/ViewModels/SettingsViewModel.cs` | `src/Wif.App/Views/SettingsView.axaml` | 账户添加、删除、OAuth2 配置完整 |

## 2. 文档处理能力

| 流程段 | 能力 | 状态 | 代码证据 | UI 证据 | 备注 |
|--------|------|------|----------|---------|------|
| 文档处理 | PDF 处理 | 已完成 | `src/Wif.Docs/Processors/PdfProcessor.cs` | `src/Wif.App/Views/ImportView.axaml` | 文本提取与 schema 提取 |
| 文档处理 | DOCX 处理 | 已完成 | `src/Wif.Docs/Processors/DocxProcessor.cs` | `src/Wif.App/Views/ImportView.axaml` | 文本提取与 schema 提取 |
| 文档处理 | XLSX 处理 | 已完成 | `src/Wif.Docs/Processors/XlsxProcessor.cs` | `src/Wif.App/Views/ImportView.axaml` | 文本提取与 schema 提取 |
| 文档处理 | PPTX 处理 | 已完成 | `src/Wif.Docs/Processors/PptxProcessor.cs` | `src/Wif.App/Views/ImportView.axaml` | 文本提取与 schema 提取 |
| 文档处理 | PDF 导出 | 已完成 | `src/Wif.Docs/Exporters/PdfExporter.cs` | `src/Wif.App/Views/ImportView.axaml` | 工作项内容导出 |
| 文档处理 | DOCX 导出 | 已完成 | `src/Wif.Docs/Exporters/DocxExporter.cs` | `src/Wif.App/Views/ImportView.axaml` | 工作项内容导出 |
| 文档处理 | XLSX 导出 | 已完成 | `src/Wif.Docs/Exporters/XlsxExporter.cs` | `src/Wif.App/Views/ImportView.axaml` | 工作项内容导出 |
| 文档处理 | PPTX 导出 | 已完成 | `src/Wif.Docs/Exporters/PptxExporter.cs` | `src/Wif.App/Views/ImportView.axaml` | 工作项内容导出 |

## 3. GIS 能力

### 3.1 图层管理

| 流程段 | 能力 | 状态 | 代码证据 | UI 证据 | 备注 |
|--------|------|------|----------|---------|------|
| 图层管理 | 新建空图层 | 已完成 | `GisViewModel.AddLayerAsync` | `src/Wif.App/Views/GisView.axaml` | 创建 + 持久化 + 地图渲染 |
| 图层管理 | 添加矢量图层 | 已完成 | `GisViewModel.AddVectorLayerAsync` | `src/Wif.App/Views/GisView.axaml` | 文件选择 + GDAL 导入 + 持久化 + 渲染 |
| 图层管理 | 添加栅格图层 | 已完成 | `GisViewModel.AddRasterLayerAsync` | `src/Wif.App/Views/GisView.axaml` | 文件选择 + GDAL 加载 + 持久化 + 渲染 |
| 图层管理 | 添加 WMS/WMTS | 半完成 | `GisViewModel.AddWmsLayerAsync` | `src/Wif.App/Views/GisView.axaml` | 通过剪贴板 URL 添加，无对话框 |
| 图层管理 | 添加 XYZ 瓦片 | 半完成 | `GisViewModel.AddXyzLayerAsync` | `src/Wif.App/Views/GisView.axaml` | 通过剪贴板 URL 添加，无对话框 |
| 图层管理 | 删除图层 | 已完成 | `GisViewModel.RemoveLayerAsync` | `src/Wif.App/Views/GisView.axaml` | 删除 + 地图移除 + UI 刷新 |
| 图层管理 | 图层排序 | 已完成 | `GisViewModel.MoveLayerUp/Down` | `src/Wif.App/Views/GisView.axaml` | UI 排序 |
| 图层管理 | 可见性切换 | 已完成 | `GisViewModel.ToggleLayerVisibility` | `src/Wif.App/Views/GisView.axaml` | 联动 MapService |
| 图层管理 | 切换编辑模式 | 已完成 | `GisViewModel.ToggleEditing` | `src/Wif.App/Views/GisView.axaml` | 激活/关闭工具管理器 |
| 图层管理 | 保存编辑 | 已完成 | `GisViewModel.SaveEditsAsync` | `src/Wif.App/Views/GisView.axaml` | 更新图层时间戳并持久化 |
| 图层管理 | 图层属性 | 半完成 | `GisViewModel.LayerPropertiesAsync` | `src/Wif.App/Views/GisView.axaml` | 纯文本信息输出到 AnalysisResult，无属性编辑对话框 |

### 3.2 导入导出

| 流程段 | 能力 | 状态 | 代码证据 | UI 证据 | 备注 |
|--------|------|------|----------|---------|------|
| 导入导出 | 导入 GeoJSON | 已完成 | `GisViewModel.ImportGeoJsonAsync` | `src/Wif.App/Views/GisView.axaml` | 文件选择 + 解析 + 持久化 + 渲染 |
| 导入导出 | 导入 Shapefile | 已完成 | `GisViewModel.ImportShapefileAsync` | `src/Wif.App/Views/GisView.axaml` | GDAL 读取 + 持久化 + 渲染 |
| 导入导出 | 统一导入 | 已完成 | `GisViewModel.ImportFileAsync` | `src/Wif.App/Views/GisView.axaml` | 支持 SHP/GeoJSON/GPKG/KML/GML/GeoTIFF |
| 导入导出 | 导出 GeoJSON | 已完成 | `GisViewModel.ExportGeoJsonAsync` | `src/Wif.App/Views/GisView.axaml` | 文件保存对话框 + 序列化 + 写入 |

### 3.3 矢量空间分析

| 流程段 | 能力 | 状态 | 代码证据 | UI 证据 | 备注 |
|--------|------|------|----------|---------|------|
| 矢量空间分析 | 缓冲区 | 已完成 | `GisViewModel.VectorBufferAsync` | `src/Wif.App/Views/GisView.axaml` | GeometryService.Buffer + 新图层持久化 + 渲染 |
| 矢量空间分析 | 相交 | 已完成 | `GisViewModel.VectorIntersectAsync` | `src/Wif.App/Views/GisView.axaml` | GeometryService.Intersection + 新图层持久化 |
| 矢量空间分析 | 合并 | 已完成 | `GisViewModel.VectorUnionAsync` | `src/Wif.App/Views/GisView.axaml` | GeometryService.Union + 新图层持久化 |
| 矢量空间分析 | 差异 | 已完成 | `GisViewModel.VectorDifferenceAsync` | `src/Wif.App/Views/GisView.axaml` | GeometryService.Difference + 新图层持久化 |
| 矢量空间分析 | 裁剪 | 已完成 | `GisViewModel.VectorClipAsync` | `src/Wif.App/Views/GisView.axaml` | GeometryService.Clip + 新图层持久化 |
| 矢量空间分析 | 融合 | 已完成 | `GisViewModel.VectorDissolveAsync` | `src/Wif.App/Views/GisView.axaml` | GeometryService.UnaryUnion + 新图层持久化 |
| 矢量空间分析 | 面积计算 | 半完成 | `GisViewModel.CalculateAreaAsync` | `src/Wif.App/Views/GisView.axaml` | 真实计算，但结果仅文本输出到 AnalysisResult |
| 矢量空间分析 | 距离计算 | 半完成 | `GisViewModel.CalculateDistanceAsync` | `src/Wif.App/Views/GisView.axaml` | 真实计算，但结果仅文本输出，只取前两个要素 |

### 3.4 栅格分析

| 流程段 | 能力 | 状态 | 代码证据 | UI 证据 | 备注 |
|--------|------|------|----------|---------|------|
| 栅格分析 | 栅格信息 | 已完成 | `GisViewModel.RasterInfoAsync` | `src/Wif.App/Views/GisView.axaml` | 读取 GDAL 元数据并展示 |
| 栅格分析 | 栅格计算器 | 半完成 | `GisViewModel.RasterCalculatorAsync` | `src/Wif.App/Views/GisView.axaml` | 实际为单点高程查询，非栅格代数运算器 |
| 栅格分析 | 栅格重投影 | 占位 | `GisViewModel.RasterReprojectAsync` | - | 仅显示坐标转换信息，提示需用 gdalwarp 命令行 |
| 栅格分析 | 栅格裁剪 | 占位 | `GisViewModel.RasterClipAsync` | - | 仅显示范围信息，提示需用 gdal_translate 命令行 |

### 3.5 编辑工具

| 流程段 | 能力 | 状态 | 代码证据 | UI 证据 | 备注 |
|--------|------|------|----------|---------|------|
| 编辑工具 | 选择工具 | 已完成 | `SelectionTool` + `ToolManager` | `src/Wif.App/Views/GisView.axaml` | 要素选择与高亮 |
| 编辑工具 | 编辑节点 | 已完成 | `EditingTool` + `ToolManager` | `src/Wif.App/Views/GisView.axaml` | 几何节点拖拽编辑 + 持久化 |
| 编辑工具 | 画点/线/面 | 已完成 | `DrawingTool` + `ToolManager` | `src/Wif.App/Views/GisView.axaml` | 绘制 + 持久化 + 撤销支持 |
| 编辑工具 | 移动要素 | 已完成 | `GisViewModel.MoveFeature` | `src/Wif.App/Views/GisView.axaml` | 激活编辑工具 |
| 编辑工具 | 拆分要素 | 已完成 | `GisViewModel.SplitFeatureAsync` | `src/Wif.App/Views/GisView.axaml` | 几何拆分 + 持久化 |
| 编辑工具 | 合并要素 | 已完成 | `GisViewModel.MergeFeaturesAsync` | `src/Wif.App/Views/GisView.axaml` | 几何合并 + 持久化 |
| 编辑工具 | 撤销/重做 | 已完成 | `ToolManager.UndoAsync/RedoAsync` | `src/Wif.App/Views/GisView.axaml` | 命令栈支持 |
| 编辑工具 | 复制/粘贴/删除 | 已完成 | `GisViewModel.EditCopy/Paste/DeleteAsync` | `src/Wif.App/Views/GisView.axaml` | 要素级操作 + 持久化 |
| 编辑工具 | 全选/取消全选 | 已完成 | `GisViewModel.SelectAll/DeselectAll` | `src/Wif.App/Views/GisView.axaml` | 要素 ID 集合管理 |

### 3.6 测量与属性

| 流程段 | 能力 | 状态 | 代码证据 | UI 证据 | 备注 |
|--------|------|------|----------|---------|------|
| 测量与属性 | 测距 | 已完成 | `MeasureTool` (Distance mode) | `src/Wif.App/Views/GisView.axaml` | 交互式测量 + 状态反馈 |
| 测量与属性 | 测面 | 已完成 | `MeasureTool` (Area mode) | `src/Wif.App/Views/GisView.axaml` | 交互式测量 + 状态反馈 |
| 测量与属性 | 识别要素 | 半完成 | `GisViewModel.IdentifyFeatureAsync` | `src/Wif.App/Views/GisView.axaml` | 最近要素文本信息输出，无地图高亮联动 |
| 测量与属性 | 属性表 | 已完成 | `AttributeTableViewModel` | `src/Wif.App/Views/GisView.axaml` | 可折叠 DataGrid，加载图层要素 |
| 测量与属性 | 字段计算器 | 半完成 | `GisViewModel.OpenFieldCalculatorAsync` | `src/Wif.App/Views/GisView.axaml` | 实为字段统计汇总，非字段表达式计算器 |
| 测量与属性 | 标注切换 | 占位 | `GisViewModel.ToggleLabels` | - | 仅设置 bool 标志，`Wif.Gis` 无标注渲染消费逻辑 |

### 3.7 导航与视图

| 流程段 | 能力 | 状态 | 代码证据 | UI 证据 | 备注 |
|--------|------|------|----------|---------|------|
| 导航与视图 | 放大/缩小 | 已完成 | `Map.Navigator.ZoomIn/Out` | `src/Wif.App/Views/GisView.axaml` | 联动视口历史 |
| 导航与视图 | 全图范围 | 已完成 | `MapService.ZoomToAll` | `src/Wif.App/Views/GisView.axaml` | |
| 导航与视图 | 缩放到图层 | 已完成 | `MapService.ZoomToLayer` | `src/Wif.App/Views/GisView.axaml` | |
| 导航与视图 | 缩放到选中 | 已完成 | `GisViewModel.ZoomToSelected` | `src/Wif.App/Views/GisView.axaml` | 复用 ZoomToLayer |
| 导航与视图 | 前一/后一视图 | 已完成 | 视口 Undo/Redo 栈 | `src/Wif.App/Views/GisView.axaml` | |
| 导航与视图 | 平移 | 已完成 | `GisViewModel.PanTool` | `src/Wif.App/Views/GisView.axaml` | |
| 导航与视图 | 底图选择 | 半完成 | `OnSelectedBasemapIndexChanged` | `src/Wif.App/Views/GisView.axaml` | OSM 可用，天地图需 API Key，无对话框 |
| 导航与视图 | 刷新地图 | 已完成 | `Map.Refresh()` | `src/Wif.App/Views/GisView.axaml` | |

### 3.8 面板

| 流程段 | 能力 | 状态 | 代码证据 | UI 证据 | 备注 |
|--------|------|------|----------|---------|------|
| 面板 | 浏览器面板 | 已完成 | `BrowserNodes` + 目录扫描 | `src/Wif.App/Views/GisView.axaml` | 目录添加/移除/过滤/刷新完整 |
| 面板 | 图层面板 | 已完成 | `Layers` 集合 + 面板 UI | `src/Wif.App/Views/GisView.axaml` | 图层 CRUD + 可见性 + 排序 |
| 面板 | AI 助手面板 | 半完成 | `AiGisAssistantViewModel` | `src/Wif.App/Views/GisView.axaml` | 面板切换 + 上下文更新，orchestrator 待接入 |

### 3.9 工程管理

| 流程段 | 能力 | 状态 | 代码证据 | UI 证据 | 备注 |
|--------|------|------|----------|---------|------|
| 工程管理 | 新建工程 | 占位 | `StatusMessage = "功能开发中"` | - | |
| 工程管理 | 打开工程 | 占位 | `StatusMessage = "功能开发中"` | - | |
| 工程管理 | 保存工程 | 占位 | `StatusMessage = "功能开发中"` | - | |
| 工程管理 | 工程另存为 | 占位 | `StatusMessage = "功能开发中"` | - | |

### 3.10 数据处理

| 流程段 | 能力 | 状态 | 代码证据 | UI 证据 | 备注 |
|--------|------|------|----------|---------|------|
| 数据处理 | 处理工具箱 | 占位 | `StatusMessage = "功能开发中"` | - | |
| 数据处理 | 处理历史 | 占位 | `StatusMessage = "功能开发中"` | - | |
| 数据处理 | 批处理 | 占位 | `StatusMessage = "功能开发中"` | - | |

### 3.11 状态栏

| 流程段 | 能力 | 状态 | 代码证据 | UI 证据 | 备注 |
|--------|------|------|----------|---------|------|
| 状态栏 | 状态消息 | 已完成 | `StatusMessage` 属性 | `src/Wif.App/Views/GisView.axaml` | 所有命令都设置状态消息 |
| 状态栏 | 坐标显示 | 已完成 | `CursorPosition` + SphericalMercator 转换 | `src/Wif.App/Views/GisView.axaml` | 鼠标移动实时更新 |
| 状态栏 | 比例尺 | 已完成 | `MapScale` | `src/Wif.App/Views/GisView.axaml` | Viewport 计算 |
| 状态栏 | CRS 显示 | 已完成 | `MapCrs` | `src/Wif.App/Views/GisView.axaml` | |
| 状态栏 | 放大倍数 | 已完成 | `MapMagnification` | `src/Wif.App/Views/GisView.axaml` | |
| 状态栏 | 旋转角度 | 已完成 | `MapRotation` | `src/Wif.App/Views/GisView.axaml` | |

### 3.12 AI-GIS 集成

| 流程段 | 能力 | 状态 | 代码证据 | UI 证据 | 备注 |
|--------|------|------|----------|---------|------|
| AI-GIS 集成 | AI-GIS 编排器 | 半完成 | `src/Wif.Core/Services/AiGisOrchestrator.cs` | `src/Wif.App/Views/GisView.axaml` | 编排逻辑存在，前台连接待完成 |
| AI-GIS 集成 | AI-GIS 计划审查 | 半完成 | `src/Wif.App/Views/AiGisPlanReviewDialog.axaml` | `src/Wif.App/Views/GisView.axaml` | 对话框 UI 存在 |
| AI-GIS 集成 | AI-GIS 执行历史 | 半完成 | `src/Wif.App/Views/AiGisHistoryPanel.axaml` | `src/Wif.App/Views/GisView.axaml` | 面板 UI 存在 |
| AI-GIS 集成 | AI-GIS 回滚 | 半完成 | `src/Wif.Core/Services/AiGisRollbackService.cs` | `src/Wif.App/Views/GisView.axaml` | 后端逻辑存在 |

---

## 4. 状态汇总

| 状态 | 数量 | 说明 |
|------|------|------|
| 已完成 | 63 | 入口 + 后端 + 持久化/回显闭环齐全 |
| 半完成 | 24 | 有真实逻辑，缺关键交互或集成 |
| 占位 | 10 | 仅命令入口或状态提示 |
| 缺失 | 0 | — |
| 失真 | 0 | 已在本次审计中修正（见 `docs/gis-ui-interface-map.md`） |

## 5. 关键发现

1. **矢量空间分析已完成度高**：Buffer/Intersect/Union/Difference/Clip/Dissolve 均完成了几何运算 + 新图层创建 + 持久化 + 地图渲染的完整闭环。

2. **栅格分析主要是占位**：栅格重投影和栅格裁剪仅显示信息文本并提示需用命令行工具；栅格计算器实际为单点查询。

3. **工程管理全部占位**：新建/打开/保存/另存为均为 `功能开发中` 状态提示。

4. **部分"已实现"功能实为文本信息展示**：图层属性、字段计算器、识别要素等功能有真实后端调用，但结果以纯文本输出到 `AnalysisResult`，未形成完整产品交互（如对话框、地图高亮、属性编辑）。

5. **标注功能占位**：`ToggleLabels` 仅设置 bool 标志，`Wif.Gis` 中无对应标注渲染逻辑。

6. **主流程闭环不足**：平台核心"邮件/文件 -> AI 分析 -> 工作项 -> 检索/复盘"链路各段存在，但未形成统一入口和连贯流程。

---

> 参考文档：`docs/superpowers/specs/2026-04-03-platform-audit-roadmap-design.md`
> 关联文档：`docs/gis-ui-interface-map.md`

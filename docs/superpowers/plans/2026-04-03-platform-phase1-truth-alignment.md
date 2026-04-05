# Platform Phase 1 Truth Alignment and Workflow Shell Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the current module-centric desktop app into a trustworthy, workflow-first platform shell for the core path `input -> AI analysis -> work item -> follow-up -> search/review`, while publishing a code-backed capability baseline.

**Architecture:** Keep the existing Avalonia shell and service registrations, but add a thin workflow navigation model above the current modules instead of rewriting the app in one pass. Phase 1 also introduces app-layer tests, promotes hybrid search to a first-class platform capability, upgrades WorkItems into the primary processing workspace, and synchronizes implementation-facing docs with actual code status.

**Tech Stack:** Avalonia UI, CommunityToolkit.Mvvm, xUnit, existing `Wif.Core` services, existing `Wif.Search` hybrid search, Markdown docs in `docs/superpowers`

---

## Scope

This is the first implementation plan in a sequence. It intentionally excludes:

- collaboration primitives such as assignee / watcher / SLA
- GIS feature expansion
- new backend domain entities beyond what is required for shell/workflow orchestration
- deep visual polish for every page

This plan produces a stable Phase 1 platform foundation. Later phases can build collaboration, analytics, and GIS integration on top of it.

## File Map

| Action | File | Responsibility |
|--------|------|----------------|
| Create | `tests/Wif.App.Tests/Wif.App.Tests.csproj` | New app-layer test project for shell and viewmodel behavior |
| Create | `tests/Wif.App.Tests/MainWindowViewModelTests.cs` | Workflow navigation and global search tests |
| Create | `tests/Wif.App.Tests/WorkItemsViewModelTests.cs` | WorkItems workspace state and quick-filter tests |
| Modify | `Wif.sln` | Add `Wif.App.Tests` to the solution |
| Create | `src/Wif.App/ViewModels/ShellWorkspace.cs` | Workflow-first shell state enum / labels / mapping helpers |
| Modify | `src/Wif.App/ViewModels/MainWindowViewModel.cs` | Replace raw module switching semantics with workflow workspace semantics |
| Modify | `src/Wif.App/MainWindow.axaml` | Rebuild left navigation into workflow sections and add global search entry |
| Modify | `src/Wif.App/MainWindow.axaml.cs` | Wire the new shell events to existing views without breaking WebView2 handling |
| Modify | `src/Wif.Core/Services/SearchService.cs` | Make hybrid search the default shell-facing search path |
| Modify | `tests/Wif.Core.Tests/SearchServiceTests.cs` | Lock hybrid-search fallback behavior with tests |
| Modify | `src/Wif.App/ViewModels/WorkItemsViewModel.cs` | Add workflow quick-filters, shell handoff hooks, and search mode support |
| Modify | `src/Wif.App/Views/WorkItemsView.axaml` | Reframe WorkItems as the primary processing workspace |
| Modify | `src/Wif.App/Views/WorkItemsView.axaml.cs` | Keep interaction hooks aligned with the redesigned workspace |
| Modify | `src/Wif.App/ViewModels/WorkItemDetailViewModel.cs` | Add summary/action-first computed state for detail workspace |
| Modify | `src/Wif.App/Views/WorkItemDetailView.axaml` | Replace stacked detail page with summary + action + context layout |
| Create | `docs/superpowers/platform-capability-matrix.md` | Code-backed capability matrix for platform Phase 1 |
| Modify | `docs/gis-ui-interface-map.md` | Downgrade incorrect “已实现” entries to actual status categories |
| Modify | `docs/superpowers/specs/2026-04-03-platform-audit-roadmap-design.md` | Link the delivered Phase 1 plan and update execution notes if needed |

---

### Task 1: Add App-Layer Test Coverage for Shell and Workspace State

**Files:**
- Create: `tests/Wif.App.Tests/Wif.App.Tests.csproj`
- Create: `tests/Wif.App.Tests/MainWindowViewModelTests.cs`
- Create: `tests/Wif.App.Tests/WorkItemsViewModelTests.cs`
- Modify: `Wif.sln`

- [ ] **Step 1: Create the new app test project file**

Add `tests/Wif.App.Tests/Wif.App.Tests.csproj`:

```xml
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net9.0</TargetFramework>
    <ImplicitUsings>enable</ImplicitUsings>
    <Nullable>enable</Nullable>
    <IsPackable>false</IsPackable>
  </PropertyGroup>

  <ItemGroup>
    <PackageReference Include="Microsoft.NET.Test.Sdk" Version="17.12.0" />
    <PackageReference Include="xunit" Version="2.9.2" />
    <PackageReference Include="xunit.runner.visualstudio" Version="3.0.0" />
    <PackageReference Include="coverlet.collector" Version="6.0.4">
      <PrivateAssets>all</PrivateAssets>
    </PackageReference>
  </ItemGroup>

  <ItemGroup>
    <ProjectReference Include="..\..\src\Wif.App\Wif.App.csproj" />
    <ProjectReference Include="..\..\src\Wif.Core\Wif.Core.csproj" />
    <ProjectReference Include="..\..\src\Wif.Domain\Wif.Domain.csproj" />
  </ItemGroup>
</Project>
```

- [ ] **Step 2: Add the test project to `Wif.sln`**

Add a new project entry matching the existing test folder structure and include build mappings in `GlobalSection(ProjectConfigurationPlatforms)`.

- [ ] **Step 3: Write the first failing shell navigation test**

Create `tests/Wif.App.Tests/MainWindowViewModelTests.cs`:

```csharp
using Wif.App.ViewModels;
using Xunit;

public class MainWindowViewModelTests
{
    [Fact]
    public void DefaultWorkspace_IsInflow()
    {
        var vm = TestMainWindowViewModelFactory.Create();
        Assert.Equal(ShellWorkspace.Inflow, vm.ActiveWorkspace);
    }
}
```

- [ ] **Step 4: Write the first failing WorkItems quick-filter test**

Create `tests/Wif.App.Tests/WorkItemsViewModelTests.cs`:

```csharp
using Wif.App.ViewModels;
using Xunit;

public class WorkItemsViewModelTests
{
    [Fact]
    public void QuickFilter_Default_IsAllWork()
    {
        var vm = TestWorkItemsViewModelFactory.Create();
        Assert.Equal(WorkItemsQuickFilter.AllWork, vm.QuickFilter);
    }
}
```

- [ ] **Step 5: Run the new app tests to verify they fail**

Run: `dotnet test tests/Wif.App.Tests/Wif.App.Tests.csproj -v minimal`
Expected: FAIL because `ShellWorkspace`, `ActiveWorkspace`, `WorkItemsQuickFilter`, and test factories do not exist yet

- [ ] **Step 6: Add minimal test factories**

In each test file, add a tiny local factory that builds `MainWindowViewModel` / `WorkItemsViewModel` using fake dependencies or existing in-memory stubs.

- [ ] **Step 7: Run the tests again until only missing production symbols remain**

Run: `dotnet test tests/Wif.App.Tests/Wif.App.Tests.csproj -v minimal`
Expected: FAIL only on missing production members, not broken test infrastructure

- [ ] **Step 8: Commit the isolated test-project setup**

```bash
git add Wif.sln tests/Wif.App.Tests
git commit -m "test: add app-layer tests for shell and workspace state"
```

---

### Task 2: Replace Module-Centric Shell State with Workflow Workspaces

**Files:**
- Create: `src/Wif.App/ViewModels/ShellWorkspace.cs`
- Modify: `src/Wif.App/ViewModels/MainWindowViewModel.cs`
- Modify: `src/Wif.App/MainWindow.axaml`
- Modify: `src/Wif.App/MainWindow.axaml.cs`
- Test: `tests/Wif.App.Tests/MainWindowViewModelTests.cs`

- [ ] **Step 1: Add the workflow workspace enum and helper metadata**

Create `src/Wif.App/ViewModels/ShellWorkspace.cs`:

```csharp
namespace Wif.App.ViewModels;

public enum ShellWorkspace
{
    Inflow,
    Workbench,
    Review,
    Spatial,
    Settings
}

public static class ShellWorkspaceLabels
{
    public static string ToDisplayName(this ShellWorkspace workspace) => workspace switch
    {
        ShellWorkspace.Inflow => "输入流",
        ShellWorkspace.Workbench => "工作台",
        ShellWorkspace.Review => "复盘",
        ShellWorkspace.Spatial => "空间分析",
        ShellWorkspace.Settings => "设置",
        _ => workspace.ToString()
    };
}
```

- [ ] **Step 2: Update `MainWindowViewModel` to expose workflow state**

Replace the raw string module state with:

```csharp
[ObservableProperty]
[NotifyPropertyChangedFor(nameof(IsMailModeActive))]
private ShellWorkspace _activeWorkspace = ShellWorkspace.Inflow;

public bool IsMailModeActive => ActiveWorkspace == ShellWorkspace.Inflow;
```

Add strongly typed navigation commands:

```csharp
[RelayCommand] private void NavigateInflow() => ActiveWorkspace = ShellWorkspace.Inflow;
[RelayCommand] private void NavigateWorkbench() => ActiveWorkspace = ShellWorkspace.Workbench;
[RelayCommand] private void NavigateReview() => ActiveWorkspace = ShellWorkspace.Review;
[RelayCommand] private void NavigateSpatial() => ActiveWorkspace = ShellWorkspace.Spatial;
[RelayCommand] private void NavigateSettingsWorkspace() => ActiveWorkspace = ShellWorkspace.Settings;
```

- [ ] **Step 3: Add a shell-level global search event**

In `MainWindowViewModel.cs` add:

```csharp
[ObservableProperty]
private string _globalSearchQuery = string.Empty;

public event Action<string>? GlobalSearchRequested;

[RelayCommand]
private void RunGlobalSearch()
{
    var query = GlobalSearchQuery.Trim();
    if (query.Length == 0) return;
    GlobalSearchRequested?.Invoke(query);
}
```

- [ ] **Step 4: Extend the shell tests**

Add tests:

```csharp
[Fact]
public void NavigateWorkbench_SetsActiveWorkspace()
{
    var vm = TestMainWindowViewModelFactory.Create();
    vm.NavigateWorkbenchCommand.Execute(null);
    Assert.Equal(ShellWorkspace.Workbench, vm.ActiveWorkspace);
}

[Fact]
public void RunGlobalSearch_RaisesEventWithTrimmedQuery()
{
    var vm = TestMainWindowViewModelFactory.Create();
    string? observed = null;
    vm.GlobalSearchRequested += query => observed = query;
    vm.GlobalSearchQuery = "  permit  ";

    vm.RunGlobalSearchCommand.Execute(null);

    Assert.Equal("permit", observed);
}
```

- [ ] **Step 5: Run the shell tests to verify they fail**

Run: `dotnet test tests/Wif.App.Tests/Wif.App.Tests.csproj --filter MainWindowViewModelTests -v minimal`
Expected: FAIL because production code is not yet updated

- [ ] **Step 6: Update `MainWindow.axaml` to use workflow labels**

Replace the current primary module buttons with workflow-oriented buttons:

```xml
<Button x:Name="WorkspaceInflow" Content="收" Tag="Inflow" ToolTip.Tip="输入流" />
<Button x:Name="WorkspaceWorkbench" Content="工" Tag="Workbench" ToolTip.Tip="工作台" />
<Button x:Name="WorkspaceReview" Content="复" Tag="Review" ToolTip.Tip="复盘" />
<Button x:Name="WorkspaceSpatial" Content="图" Tag="Spatial" ToolTip.Tip="空间分析" />
<Button x:Name="WorkspaceSettings" Content="设" Tag="Settings" ToolTip.Tip="设置" />
```

Add a shell search box above content:

```xml
<TextBox x:Name="GlobalSearchBox"
         Text="{Binding GlobalSearchQuery, Mode=TwoWay}"
         Watermark="搜索邮件、工作项、附件..." />
```

- [ ] **Step 7: Update `MainWindow.axaml.cs` to map workspaces to existing views**

Implement a workspace-to-view mapping:

```csharp
private void NavigateToWorkspace(ShellWorkspace workspace)
{
    _vm.ActiveWorkspace = workspace;
    UpdateWorkspaceButtonStates();

    switch (workspace)
    {
        case ShellWorkspace.Inflow:
            ShowStaticView(InboxContent);
            break;
        case ShellWorkspace.Workbench:
        case ShellWorkspace.Review:
            ShowStaticView(WorkItemsContent);
            break;
        case ShellWorkspace.Spatial:
            ShowStaticView(GisContent);
            break;
        case ShellWorkspace.Settings:
            ShowStaticView(SettingsContent);
            break;
    }
}
```

- [ ] **Step 8: Wire global search to WorkItems**

In the window constructor:

```csharp
_vm.GlobalSearchRequested += query =>
{
    NavigateToWorkspace(ShellWorkspace.Workbench);
    if (WorkItemsContent.DataContext is WorkItemsViewModel workItemsVm)
        workItemsVm.RunShellSearch(query);
};
```

- [ ] **Step 9: Run the shell tests to verify they pass**

Run: `dotnet test tests/Wif.App.Tests/Wif.App.Tests.csproj --filter MainWindowViewModelTests -v minimal`
Expected: PASS

- [ ] **Step 10: Build the app**

Run: `dotnet build src/Wif.App/Wif.App.csproj`
Expected: Build succeeded

- [ ] **Step 11: Commit**

```bash
git add src/Wif.App/ViewModels/ShellWorkspace.cs src/Wif.App/ViewModels/MainWindowViewModel.cs src/Wif.App/MainWindow.axaml src/Wif.App/MainWindow.axaml.cs tests/Wif.App.Tests/MainWindowViewModelTests.cs
git commit -m "feat: add workflow-first shell navigation"
```

---

### Task 3: Promote Hybrid Search to a First-Class Platform Capability

**Files:**
- Modify: `src/Wif.Core/Services/SearchService.cs`
- Modify: `tests/Wif.Core.Tests/SearchServiceTests.cs`
- Modify: `src/Wif.App/ViewModels/WorkItemsViewModel.cs`
- Test: `tests/Wif.App.Tests/WorkItemsViewModelTests.cs`

- [ ] **Step 1: Add failing hybrid-search tests in core**

Extend `tests/Wif.Core.Tests/SearchServiceTests.cs` with:

```csharp
[Fact]
public async Task HybridSearchAsync_FallsBackToRepositorySearch_WhenHybridServiceMissing()
{
    var repo = new FakeWorkItemRepository();
    repo.SearchResults = [new WorkItem { Title = "permit review" }];
    var service = new SearchService(repo, hybridSearch: null);

    var results = await service.HybridSearchAsync("permit");

    Assert.Single(results);
    Assert.Equal("permit review", results[0].Title);
}
```

- [ ] **Step 2: Add a failing app-level shell-search test**

Extend `tests/Wif.App.Tests/WorkItemsViewModelTests.cs`:

```csharp
[Fact]
public void RunShellSearch_SetsQuickFilterToSearch()
{
    var vm = TestWorkItemsViewModelFactory.Create();

    vm.RunShellSearch("critical permit");

    Assert.Equal(WorkItemsQuickFilter.SearchResults, vm.QuickFilter);
    Assert.Equal("critical permit", vm.SearchQuery);
}
```

- [ ] **Step 3: Run the targeted tests and confirm they fail**

Run:

```bash
dotnet test tests/Wif.Core.Tests/Wif.Core.Tests.csproj --filter SearchServiceTests -v minimal
dotnet test tests/Wif.App.Tests/Wif.App.Tests.csproj --filter WorkItemsViewModelTests -v minimal
```

Expected: FAIL on missing `RunShellSearch` / `WorkItemsQuickFilter.SearchResults` or incorrect behavior

- [ ] **Step 4: Make hybrid search the default shell-facing method**

Update `SearchService.cs`:

```csharp
public Task<IReadOnlyList<WorkItem>> SearchAsync(
    string query, int limit = 20, CancellationToken ct = default)
    => HybridSearchAsync(query, SearchMode.Hybrid, limit, ct);
```

Keep repository fallback inside `HybridSearchAsync`.

- [ ] **Step 5: Add quick-filter enum support to `WorkItemsViewModel`**

Add:

```csharp
public enum WorkItemsQuickFilter
{
    AllWork,
    RecentInputs,
    NeedsAttention,
    SearchResults
}

[ObservableProperty]
private WorkItemsQuickFilter _quickFilter = WorkItemsQuickFilter.AllWork;
```

- [ ] **Step 6: Add shell-search handoff method**

Add to `WorkItemsViewModel.cs`:

```csharp
public void RunShellSearch(string query)
{
    SearchQuery = query;
    QuickFilter = WorkItemsQuickFilter.SearchResults;
    _ = SearchCommand.ExecuteAsync(null);
}
```

- [ ] **Step 7: Run the targeted tests again**

Run:

```bash
dotnet test tests/Wif.Core.Tests/Wif.Core.Tests.csproj --filter SearchServiceTests -v minimal
dotnet test tests/Wif.App.Tests/Wif.App.Tests.csproj --filter WorkItemsViewModelTests -v minimal
```

Expected: PASS

- [ ] **Step 8: Commit**

```bash
git add src/Wif.Core/Services/SearchService.cs src/Wif.App/ViewModels/WorkItemsViewModel.cs tests/Wif.Core.Tests/SearchServiceTests.cs tests/Wif.App.Tests/WorkItemsViewModelTests.cs
git commit -m "feat: promote hybrid search to shell-level workflow"
```

---

### Task 4: Reframe WorkItems as the Primary Processing Workspace

**Files:**
- Modify: `src/Wif.App/ViewModels/WorkItemsViewModel.cs`
- Modify: `src/Wif.App/Views/WorkItemsView.axaml`
- Modify: `src/Wif.App/Views/WorkItemsView.axaml.cs`
- Test: `tests/Wif.App.Tests/WorkItemsViewModelTests.cs`

- [ ] **Step 1: Add failing quick-filter tests**

Add:

```csharp
[Fact]
public async Task SetQuickFilterAsync_NeedsAttention_SetsWaitingFilter()
{
    var vm = TestWorkItemsViewModelFactory.CreateWithData();
    await vm.SetQuickFilterAsync(WorkItemsQuickFilter.NeedsAttention);
    Assert.Equal(WorkItemsQuickFilter.NeedsAttention, vm.QuickFilter);
    Assert.Equal(WorkItemStatus.Waiting, vm.SelectedFilter);
}
```

- [ ] **Step 2: Run the app tests to verify failure**

Run: `dotnet test tests/Wif.App.Tests/Wif.App.Tests.csproj --filter WorkItemsViewModelTests -v minimal`
Expected: FAIL because `SetQuickFilterAsync` does not exist

- [ ] **Step 3: Add quick-filter command logic**

In `WorkItemsViewModel.cs` add:

```csharp
[RelayCommand]
public async Task SetQuickFilterAsync(WorkItemsQuickFilter filter)
{
    QuickFilter = filter;

    switch (filter)
    {
        case WorkItemsQuickFilter.RecentInputs:
            SelectedFilter = WorkItemStatus.Inbox;
            break;
        case WorkItemsQuickFilter.NeedsAttention:
            SelectedFilter = WorkItemStatus.Waiting;
            break;
        case WorkItemsQuickFilter.AllWork:
        case WorkItemsQuickFilter.SearchResults:
        default:
            SelectedFilter = null;
            break;
    }

    await LoadAsync();
}
```

- [ ] **Step 4: Add computed section labels**

Add:

```csharp
public string WorkspaceTitle => QuickFilter switch
{
    WorkItemsQuickFilter.RecentInputs => "最近输入",
    WorkItemsQuickFilter.NeedsAttention => "待处理",
    WorkItemsQuickFilter.SearchResults => "搜索结果",
    _ => "工作台"
};
```

- [ ] **Step 5: Redesign the top of `WorkItemsView.axaml` into a workbench**

Add a quick-filter strip above the stat cards:

```xml
<StackPanel Orientation="Horizontal" Spacing="8">
  <Button Content="全部工作" Command="{Binding SetQuickFilterCommand}" CommandParameter="{x:Static vm:WorkItemsQuickFilter.AllWork}" />
  <Button Content="最近输入" Command="{Binding SetQuickFilterCommand}" CommandParameter="{x:Static vm:WorkItemsQuickFilter.RecentInputs}" />
  <Button Content="待处理" Command="{Binding SetQuickFilterCommand}" CommandParameter="{x:Static vm:WorkItemsQuickFilter.NeedsAttention}" />
</StackPanel>
```

Replace the page title with:

```xml
<TextBlock Text="{Binding WorkspaceTitle}" FontSize="24" FontWeight="Bold" />
```

- [ ] **Step 6: Keep existing batch operations and card interactions intact**

Do not remove:

- `x:Name="SearchBox"`
- `x:Name="ItemsList"`
- `DoubleTapped="OnCardDoubleTapped"`

- [ ] **Step 7: Run app tests**

Run: `dotnet test tests/Wif.App.Tests/Wif.App.Tests.csproj --filter WorkItemsViewModelTests -v minimal`
Expected: PASS

- [ ] **Step 8: Build the app**

Run: `dotnet build src/Wif.App/Wif.App.csproj`
Expected: Build succeeded

- [ ] **Step 9: Commit**

```bash
git add src/Wif.App/ViewModels/WorkItemsViewModel.cs src/Wif.App/Views/WorkItemsView.axaml src/Wif.App/Views/WorkItemsView.axaml.cs tests/Wif.App.Tests/WorkItemsViewModelTests.cs
git commit -m "feat: turn WorkItems into primary processing workspace"
```

---

### Task 5: Turn WorkItem Detail into an Action-First Workspace

**Files:**
- Modify: `src/Wif.App/ViewModels/WorkItemDetailViewModel.cs`
- Modify: `src/Wif.App/Views/WorkItemDetailView.axaml`

- [ ] **Step 1: Add computed summary properties in the viewmodel**

Add:

```csharp
public bool HasLocation => CurrentItem?.Latitude is not null && CurrentItem?.Longitude is not null;
public bool HasLinkedFeature => CurrentItem?.GisFeatureId is not null;
public bool HasProposals => Proposals.Count > 0;
public string NextActionHint => CurrentItem?.Status switch
{
    WorkItemStatus.Inbox => "建议先执行 AI 分析并确认是否生成后续动作",
    WorkItemStatus.Waiting => "建议明确阻塞原因并设置下一步处理动作",
    WorkItemStatus.Active => "建议推进处理并补齐上下文",
    WorkItemStatus.Done => "建议执行复盘或归档",
    _ => "查看上下文并选择下一步动作"
};
```

- [ ] **Step 2: Add an action-priority test by reasoning, then implement minimally**

Because there is no existing detail-view test project, verify the computed state through buildable properties first instead of UI automation.

- [ ] **Step 3: Replace the top of `WorkItemDetailView.axaml`**

New top structure:

```xml
<Grid ColumnDefinitions="2*,*" RowDefinitions="Auto,Auto,Auto">
  <Border Grid.Column="0" Padding="16" CornerRadius="10">
    <!-- title, status, source, created-at, next action hint -->
  </Border>
  <Border Grid.Column="1" Padding="16" CornerRadius="10">
    <!-- primary actions: Analyze, approve/commit proposals, GIS actions -->
  </Border>
</Grid>
```

Move low-priority sections below:

- raw content
- export buttons
- tags
- parent/child structure

- [ ] **Step 4: Keep existing commands and bindings unchanged where possible**

The redesign must still use:

- `AnalyzeCommand`
- proposal commands
- GIS commands
- tag commands
- status change commands

- [ ] **Step 5: Build and manually verify**

Run: `dotnet build src/Wif.App/Wif.App.csproj`
Expected: Build succeeded

Manual verification:

- open a work item
- confirm next-action hint is visible above secondary context
- confirm AI and GIS actions remain reachable without scrolling through raw content

- [ ] **Step 6: Commit**

```bash
git add src/Wif.App/ViewModels/WorkItemDetailViewModel.cs src/Wif.App/Views/WorkItemDetailView.axaml
git commit -m "feat: redesign work item detail as action-first workspace"
```

---

### Task 6: Publish a Code-Backed Capability Matrix and Fix GIS Status Drift

**Files:**
- Create: `docs/superpowers/platform-capability-matrix.md`
- Modify: `docs/gis-ui-interface-map.md`
- Modify: `docs/superpowers/specs/2026-04-03-platform-audit-roadmap-design.md`

- [ ] **Step 1: Create the capability matrix document**

Create `docs/superpowers/platform-capability-matrix.md` with sections:

- workflow segment
- capability name
- status (`已完成 / 半完成 / 占位 / 缺失 / 失真`)
- code evidence
- UI evidence
- notes / next action

Start with this table:

```md
| 流程段 | 能力 | 状态 | 代码证据 | UI 证据 | 备注 |
|--------|------|------|----------|--------|------|
| 输入流 | 邮件同步 | 半完成 | `src/Wif.App/ViewModels/InboxViewModel.cs` | `src/Wif.App/Views/InboxView.axaml` | 有同步与读取，无统一状态中心 |
| 分析 | AI 分析 | 半完成 | `src/Wif.Core/Services/AnalysisService.cs` | `src/Wif.App/Views/WorkItemDetailView.axaml` | 详情页可触发，缺入口级处理 |
| 检索 | 混合搜索 | 半完成 | `src/Wif.Core/Services/SearchService.cs` | `src/Wif.App/MainWindow.axaml` | Phase 1 提升为全局搜索入口 |
```

- [ ] **Step 2: Correct `docs/gis-ui-interface-map.md`**

Replace every “已实现” entry that only maps to:

- `StatusMessage = "功能开发中"`
- `待集成`
- 说明性文本输出
- 非持久化示意逻辑

with either `占位` or `半完成`.

- [ ] **Step 3: Add a “文档状态定义” section to `docs/gis-ui-interface-map.md`**

Use:

```md
### 状态定义

- 已完成：入口、后端行为、持久化/回显闭环齐全
- 半完成：有真实逻辑，但缺关键交互、集成或验证
- 占位：只有命令入口、状态提示或说明性实现
- 缺失：文档定义但代码不存在
- 失真：文档状态与代码证据不一致
```

- [ ] **Step 4: Link the capability matrix from the Phase 1 spec**

Add a short note in `docs/superpowers/specs/2026-04-03-platform-audit-roadmap-design.md` pointing to the matrix as the execution-time truth source.

- [ ] **Step 5: Run a docs verification pass**

Run:

```bash
rg -n "功能开发中|待集成|计划中" src/Wif.App/ViewModels/GisViewModel.cs docs/gis-ui-interface-map.md
```

Expected:

- remaining matches in the doc should either be intentional or explicitly labeled `占位 / 计划中`
- no match should imply “已实现” for a code path that is only a stub

- [ ] **Step 6: Commit**

```bash
git add docs/superpowers/platform-capability-matrix.md docs/gis-ui-interface-map.md docs/superpowers/specs/2026-04-03-platform-audit-roadmap-design.md
git commit -m "docs: publish capability matrix and correct GIS status map"
```

---

### Task 7: Phase 1 Verification and Release Readiness

**Files:**
- Verify: `src/Wif.App/*`
- Verify: `tests/Wif.Core.Tests/*`
- Verify: `tests/Wif.App.Tests/*`
- Verify: `docs/superpowers/*`

- [ ] **Step 1: Run targeted core and app tests**

Run:

```bash
dotnet test tests/Wif.Core.Tests/Wif.Core.Tests.csproj --filter SearchServiceTests -v minimal
dotnet test tests/Wif.App.Tests/Wif.App.Tests.csproj -v minimal
```

Expected: PASS

- [ ] **Step 2: Run the full solution tests**

Run: `dotnet test Wif.sln -v minimal`
Expected: PASS

- [ ] **Step 3: Build the desktop app**

Run: `dotnet build src/Wif.App/Wif.App.csproj -v minimal`
Expected: Build succeeded

- [ ] **Step 4: Manual smoke test**

Verify:

1. app opens without breaking WebView2 mail rendering
2. left shell uses workflow labels rather than module names
3. global search routes to WorkItems and returns results
4. WorkItems quick-filters switch workspace state correctly
5. WorkItem detail shows summary/action-first layout
6. GIS still opens and no shell navigation regression is introduced

- [ ] **Step 5: Final commit**

```bash
git add .
git commit -m "feat: deliver platform phase 1 workflow shell foundation"
```

- [ ] **Step 6: Update execution status in docs**

Add a short “Phase 1 completed / in progress” note to:

- `docs/superpowers/specs/2026-04-03-platform-audit-roadmap-design.md`
- `docs/superpowers/platform-capability-matrix.md`

---

## Notes for the Implementer

- Keep GIS visible but subordinate in the shell. Do not expand GIS scope in this phase.
- Prefer adding small, focused helper types over further enlarging `MainWindowViewModel` or `WorkItemsViewModel`.
- If `tests/Wif.App.Tests` proves awkward because of Avalonia references, keep test coverage focused on pure viewmodel logic and avoid UI runtime dependencies.
- If git metadata is unavailable in the local workspace snapshot, still preserve the commit boundaries described above as logical checkpoints.

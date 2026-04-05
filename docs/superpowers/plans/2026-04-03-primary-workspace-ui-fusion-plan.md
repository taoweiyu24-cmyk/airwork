# Primary Workspace UI Fusion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Convert the current mixed desktop UI into a unified workflow-first shell across the primary user journey: shell, inflow, workbench, work item detail, and settings.

**Architecture:** Keep the existing Avalonia view structure and current services, but add a shared workspace design language and rebuild the highest-value surfaces around one product model: `input -> work item -> follow-up`. This plan deliberately excludes deep GIS redesign so the team can stabilize the platform shell and primary workspaces first.

**Tech Stack:** Avalonia UI, CommunityToolkit.Mvvm, xUnit, existing `Wif.App` viewmodels/views, Markdown specs in `docs/superpowers`

---

## Scope

This plan covers:

- shared workspace styling tokens
- `MainWindow` workflow shell
- `InboxView` as inflow workspace
- `WorkItemsView` as primary workbench
- `WorkItemDetailView` as action-first detail workspace
- `SettingsView` split into product vs system mental model

This plan excludes:

- GIS visual redesign beyond shell naming consistency
- new collaboration domain features such as assignee, SLA, comments
- deep backend refactors unrelated to UI fusion

## File Map

| Action | File | Responsibility |
|--------|------|----------------|
| Create | `src/Wif.App/Styles/WorkspaceTheme.axaml` | Shared workspace cards, section headers, pill, toolbar, and shell styles |
| Modify | `src/Wif.App/App.axaml` | Merge the shared workspace theme into app-level styles |
| Create | `tests/Wif.App.Tests/Wif.App.Tests.csproj` | App-layer UI/viewmodel test project, if not already created |
| Create | `tests/Wif.App.Tests/ShellWorkspacePresentationTests.cs` | Verify shell labels, workspace state, and search handoff behavior |
| Create | `tests/Wif.App.Tests/PrimaryWorkspaceViewModelTests.cs` | Verify inflow/workbench/detail computed state without UI runtime |
| Modify | `Wif.sln` | Add `Wif.App.Tests` if missing |
| Modify | `src/Wif.App/MainWindow.axaml` | Workflow shell layout, global search, top workspace framing |
| Modify | `src/Wif.App/MainWindow.axaml.cs` | Shell navigation, workspace header syncing, search routing |
| Modify | `src/Wif.App/ViewModels/MainWindowViewModel.cs` | Shell workspace presentation state |
| Modify | `src/Wif.App/Views/InboxView.axaml` | Inflow workspace layout and processing affordances |
| Modify | `src/Wif.App/ViewModels/InboxViewModel.cs` | Inflow quick filters, state chips, and work item conversion signals |
| Modify | `src/Wif.App/Views/WorkItemsView.axaml` | Workbench-first layout and processing groups |
| Modify | `src/Wif.App/ViewModels/WorkItemsViewModel.cs` | Workspace titles, quick filters, and shell search behavior |
| Modify | `src/Wif.App/Views/WorkItemDetailView.axaml` | Action-first detail layout |
| Modify | `src/Wif.App/ViewModels/WorkItemDetailViewModel.cs` | Summary/action computed properties for detail |
| Modify | `src/Wif.App/Views/SettingsView.axaml` | Product/system settings split and stronger hierarchy |
| Modify | `src/Wif.App/ViewModels/SettingsViewModel.cs` | Settings section grouping metadata |
| Modify | `docs/superpowers/specs/2026-04-03-ui-benchmark-fusion-design.md` | Mark which UI fusion tasks are covered by this plan |

---

### Task 1: Add Shared Workspace Theme Tokens

**Files:**
- Create: `src/Wif.App/Styles/WorkspaceTheme.axaml`
- Modify: `src/Wif.App/App.axaml`

- [ ] **Step 1: Create the shared style resource file**

Add `src/Wif.App/Styles/WorkspaceTheme.axaml` with reusable styles:

```xml
<Styles xmlns="https://github.com/avaloniaui"
        xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
  <Style Selector="Border.workspace-surface">
    <Setter Property="Background" Value="{DynamicResource SystemControlBackgroundBaseLowBrush}" />
    <Setter Property="CornerRadius" Value="12" />
    <Setter Property="Padding" Value="16" />
    <Setter Property="BorderBrush" Value="{DynamicResource SystemControlForegroundBaseLowBrush}" />
    <Setter Property="BorderThickness" Value="1" />
  </Style>

  <Style Selector="TextBlock.workspace-title">
    <Setter Property="FontSize" Value="24" />
    <Setter Property="FontWeight" Value="Bold" />
  </Style>

  <Style Selector="TextBlock.workspace-subtitle">
    <Setter Property="FontSize" Value="12" />
    <Setter Property="Opacity" Value="0.6" />
  </Style>

  <Style Selector="Border.workspace-pill">
    <Setter Property="CornerRadius" Value="999" />
    <Setter Property="Padding" Value="10,4" />
    <Setter Property="Background" Value="{DynamicResource SystemControlBackgroundChromeMediumBrush}" />
  </Style>

  <Style Selector="Button.workspace-nav">
    <Setter Property="Width" Value="36" />
    <Setter Property="Height" Value="36" />
    <Setter Property="Padding" Value="8,4" />
  </Style>
</Styles>
```

- [ ] **Step 2: Merge the style file into `App.axaml`**

Update the app styles:

```xml
<Application.Styles>
  <FluentTheme />
  <StyleInclude Source="/Styles/WorkspaceTheme.axaml" />
  <Style Selector="Button.ActiveModule">
    <Setter Property="Background" Value="{DynamicResource SystemAccentColor}" />
    <Setter Property="Foreground" Value="White" />
  </Style>
</Application.Styles>
```

- [ ] **Step 3: Build to verify resource loading**

Run: `dotnet build src/Wif.App/Wif.App.csproj -v minimal`
Expected: Build succeeded

- [ ] **Step 4: Commit**

```bash
git add src/Wif.App/Styles/WorkspaceTheme.axaml src/Wif.App/App.axaml
git commit -m "feat: add shared workspace theme tokens"
```

---

### Task 2: Add App-Layer Tests for Shell and Primary Workspace Presentation

**Files:**
- Create: `tests/Wif.App.Tests/Wif.App.Tests.csproj`
- Create: `tests/Wif.App.Tests/ShellWorkspacePresentationTests.cs`
- Create: `tests/Wif.App.Tests/PrimaryWorkspaceViewModelTests.cs`
- Modify: `Wif.sln`

- [ ] **Step 1: Create `Wif.App.Tests` if it does not already exist**

Use the same test project structure as other test projects:

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
    <ProjectReference Include="..\\..\\src\\Wif.App\\Wif.App.csproj" />
    <ProjectReference Include="..\\..\\src\\Wif.Core\\Wif.Core.csproj" />
    <ProjectReference Include="..\\..\\src\\Wif.Domain\\Wif.Domain.csproj" />
  </ItemGroup>
</Project>
```

- [ ] **Step 2: Add the test project to `Wif.sln` if missing**

Follow the existing solution structure under the `tests` solution folder.

- [ ] **Step 3: Write the first failing shell presentation test**

Create `tests/Wif.App.Tests/ShellWorkspacePresentationTests.cs`:

```csharp
using Wif.App.ViewModels;
using Xunit;

public class ShellWorkspacePresentationTests
{
    [Fact]
    public void ActiveWorkspace_DefaultsToInflow()
    {
        var vm = TestShellFactory.CreateMainWindowViewModel();
        Assert.Equal(ShellWorkspace.Inflow, vm.ActiveWorkspace);
    }
}
```

- [ ] **Step 4: Write the first failing primary workspace test**

Create `tests/Wif.App.Tests/PrimaryWorkspaceViewModelTests.cs`:

```csharp
using Wif.App.ViewModels;
using Xunit;

public class PrimaryWorkspaceViewModelTests
{
    [Fact]
    public void WorkspaceTitle_DefaultsToWorkbench()
    {
        var vm = TestShellFactory.CreateWorkItemsViewModel();
        Assert.Equal("工作台", vm.WorkspaceTitle);
    }
}
```

- [ ] **Step 5: Run the app tests to verify they fail**

Run: `dotnet test tests/Wif.App.Tests/Wif.App.Tests.csproj -v minimal`
Expected: FAIL because the UI presentation properties/factories are not fully implemented yet

- [ ] **Step 6: Add minimal local test factories**

Add small helper factories in the test files that create the viewmodels with fake dependencies.

- [ ] **Step 7: Re-run tests until only production-symbol failures remain**

Run: `dotnet test tests/Wif.App.Tests/Wif.App.Tests.csproj -v minimal`
Expected: FAIL only on missing production members

- [ ] **Step 8: Commit**

```bash
git add Wif.sln tests/Wif.App.Tests
git commit -m "test: add shell and primary workspace UI tests"
```

---

### Task 3: Rebuild MainWindow as a Workflow Shell

**Files:**
- Modify: `src/Wif.App/MainWindow.axaml`
- Modify: `src/Wif.App/MainWindow.axaml.cs`
- Modify: `src/Wif.App/ViewModels/MainWindowViewModel.cs`
- Test: `tests/Wif.App.Tests/ShellWorkspacePresentationTests.cs`

- [ ] **Step 1: Add failing shell interaction tests**

Extend `ShellWorkspacePresentationTests.cs`:

```csharp
[Fact]
public void WorkspaceDisplayName_TracksActiveWorkspace()
{
    var vm = TestShellFactory.CreateMainWindowViewModel();
    vm.ActiveWorkspace = ShellWorkspace.Workbench;
    Assert.Equal("工作台", vm.ActiveWorkspaceTitle);
}

[Fact]
public void RunGlobalSearch_UsesTrimmedText()
{
    var vm = TestShellFactory.CreateMainWindowViewModel();
    string? captured = null;
    vm.GlobalSearchRequested += value => captured = value;
    vm.GlobalSearchQuery = "  permit review ";

    vm.RunGlobalSearchCommand.Execute(null);

    Assert.Equal("permit review", captured);
}
```

- [ ] **Step 2: Run the shell tests to verify they fail**

Run: `dotnet test tests/Wif.App.Tests/Wif.App.Tests.csproj --filter ShellWorkspacePresentationTests -v minimal`
Expected: FAIL on missing `ActiveWorkspaceTitle` or search handoff behavior

- [ ] **Step 3: Add shell presentation properties**

In `MainWindowViewModel.cs`, add:

```csharp
[ObservableProperty]
private string _globalSearchQuery = string.Empty;

public string ActiveWorkspaceTitle => ActiveWorkspace.ToDisplayName();
public string ActiveWorkspaceSubtitle => ActiveWorkspace switch
{
    ShellWorkspace.Inflow => "吸收邮件与文件输入，转化为可处理工作项",
    ShellWorkspace.Workbench => "集中推进进行中的工作项",
    ShellWorkspace.Review => "检索、复盘与结果回顾",
    ShellWorkspace.Spatial => "空间分析与地图扩展工作台",
    ShellWorkspace.Settings => "系统配置与产品偏好",
    _ => string.Empty
};
```

- [ ] **Step 4: Replace the content header area in `MainWindow.axaml`**

Add a top workspace header above the content stack:

```xml
<Border Grid.Column="2" DockPanel.Dock="Top" Classes="workspace-surface" Margin="16,16,16,0">
  <Grid ColumnDefinitions="*,320">
    <StackPanel>
      <TextBlock Classes="workspace-title" Text="{Binding ActiveWorkspaceTitle}" />
      <TextBlock Classes="workspace-subtitle" Text="{Binding ActiveWorkspaceSubtitle}" />
    </StackPanel>
    <TextBox Grid.Column="1"
             x:Name="GlobalSearchBox"
             Text="{Binding GlobalSearchQuery, Mode=TwoWay}"
             Watermark="搜索邮件、工作项、附件..." />
  </Grid>
</Border>
```

- [ ] **Step 5: Rename primary nav affordances from module language to workflow language**

Update the primary buttons to represent:

- 输入流
- 工作台
- 复盘
- 空间分析
- 设置

Keep existing underlying views for now.

- [ ] **Step 6: Wire shell search to WorkItems from code-behind**

In `MainWindow.axaml.cs`, subscribe:

```csharp
_vm.GlobalSearchRequested += query =>
{
    NavigateToWorkItems();
    if (WorkItemsContent.DataContext is WorkItemsViewModel workItemsVm)
        workItemsVm.RunShellSearch(query);
};
```

- [ ] **Step 7: Run shell tests again**

Run: `dotnet test tests/Wif.App.Tests/Wif.App.Tests.csproj --filter ShellWorkspacePresentationTests -v minimal`
Expected: PASS

- [ ] **Step 8: Build the app**

Run: `dotnet build src/Wif.App/Wif.App.csproj -v minimal`
Expected: Build succeeded

- [ ] **Step 9: Commit**

```bash
git add src/Wif.App/MainWindow.axaml src/Wif.App/MainWindow.axaml.cs src/Wif.App/ViewModels/MainWindowViewModel.cs tests/Wif.App.Tests/ShellWorkspacePresentationTests.cs
git commit -m "feat: rebuild main window as workflow shell"
```

---

### Task 4: Turn Inbox into an Inflow Workspace

**Files:**
- Modify: `src/Wif.App/Views/InboxView.axaml`
- Modify: `src/Wif.App/ViewModels/InboxViewModel.cs`
- Test: `tests/Wif.App.Tests/PrimaryWorkspaceViewModelTests.cs`

- [ ] **Step 1: Add failing inflow-state tests**

Extend `PrimaryWorkspaceViewModelTests.cs`:

```csharp
[Fact]
public void InflowMode_DefaultsToAllInputs()
{
    var vm = TestShellFactory.CreateInboxViewModel();
    Assert.Equal("全部输入", vm.InflowFilterLabel);
}
```

- [ ] **Step 2: Run the primary workspace tests to verify failure**

Run: `dotnet test tests/Wif.App.Tests/Wif.App.Tests.csproj --filter PrimaryWorkspaceViewModelTests -v minimal`
Expected: FAIL because inflow presentation state is missing

- [ ] **Step 3: Add inflow filter presentation state**

In `InboxViewModel.cs` add:

```csharp
public enum InflowFilter
{
    AllInputs,
    NeedsProcessing,
    ConvertedToWorkItems
}

[ObservableProperty]
private InflowFilter _activeInflowFilter = InflowFilter.AllInputs;

public string InflowFilterLabel => ActiveInflowFilter switch
{
    InflowFilter.NeedsProcessing => "待处理",
    InflowFilter.ConvertedToWorkItems => "已转工作项",
    _ => "全部输入"
};
```

- [ ] **Step 4: Rebuild the header area in `InboxView.axaml`**

Replace the single folder-name header with:

```xml
<StackPanel>
  <TextBlock Classes="workspace-title" Text="输入流" />
  <TextBlock Classes="workspace-subtitle" Text="吸收邮件输入并转化为工作项" />
  <StackPanel Orientation="Horizontal" Spacing="8">
    <Button Content="全部输入" />
    <Button Content="待处理" />
    <Button Content="已转工作项" />
  </StackPanel>
</StackPanel>
```

- [ ] **Step 5: Promote primary actions in the reading pane**

In the toolbar area, visually prioritize:

- `创建工作项`
- `回复`
- `全部回复`

Move destructive or secondary actions after them:

- `标记未读`
- `删除`

- [ ] **Step 6: Add a processing summary card above message metadata**

Insert a top card in the reading pane using `workspace-surface` showing:

- current mail source
- whether attachments exist
- next recommended action

- [ ] **Step 7: Re-run the primary workspace tests**

Run: `dotnet test tests/Wif.App.Tests/Wif.App.Tests.csproj --filter PrimaryWorkspaceViewModelTests -v minimal`
Expected: PASS

- [ ] **Step 8: Build and smoke-test the inbox**

Run: `dotnet build src/Wif.App/Wif.App.csproj -v minimal`
Expected: Build succeeded

Manual check:

- message list still selects and loads body
- attachment actions still work
- create-work-item remains reachable without scrolling

- [ ] **Step 9: Commit**

```bash
git add src/Wif.App/Views/InboxView.axaml src/Wif.App/ViewModels/InboxViewModel.cs tests/Wif.App.Tests/PrimaryWorkspaceViewModelTests.cs
git commit -m "feat: redesign inbox as inflow workspace"
```

---

### Task 5: Strengthen WorkItems as the Primary Workbench

**Files:**
- Modify: `src/Wif.App/Views/WorkItemsView.axaml`
- Modify: `src/Wif.App/ViewModels/WorkItemsViewModel.cs`
- Test: `tests/Wif.App.Tests/PrimaryWorkspaceViewModelTests.cs`

- [ ] **Step 1: Add failing workbench quick-filter tests**

Extend `PrimaryWorkspaceViewModelTests.cs`:

```csharp
[Fact]
public async Task SetQuickFilterAsync_NeedsAttention_UpdatesWorkspaceTitle()
{
    var vm = TestShellFactory.CreateWorkItemsViewModel();

    await vm.SetQuickFilterAsync(WorkItemsQuickFilter.NeedsAttention);

    Assert.Equal("待处理", vm.WorkspaceTitle);
}
```

- [ ] **Step 2: Run the tests to verify failure**

Run: `dotnet test tests/Wif.App.Tests/Wif.App.Tests.csproj --filter PrimaryWorkspaceViewModelTests -v minimal`
Expected: FAIL if workspace title/quick filter logic is not aligned

- [ ] **Step 3: Add workbench subtitle and summary properties**

In `WorkItemsViewModel.cs` add:

```csharp
public string WorkspaceSubtitle => QuickFilter switch
{
    WorkItemsQuickFilter.RecentInputs => "最近沉淀的新输入项",
    WorkItemsQuickFilter.NeedsAttention => "需要优先处理的工作项",
    WorkItemsQuickFilter.SearchResults => "来自全局搜索的工作项结果",
    _ => "集中推进你的工作项"
};
```

- [ ] **Step 4: Update the top section of `WorkItemsView.axaml`**

Ensure the top section has:

- workspace title
- workspace subtitle
- quick filters row
- stats row
- search/sort/batch toolbar

in that order

- [ ] **Step 5: Add a “processing groups” row above the card grid**

Add three summary chips:

- 待我处理
- 最近输入
- 已完成

These can be presentation-only in Phase 1 if they reflect existing counts.

- [ ] **Step 6: Keep batch operations unchanged**

Do not remove or rename:

- `ToggleBatchModeCommand`
- `ToggleSelectAllCommand`
- batch status commands
- `CreateNewItemCommand`

- [ ] **Step 7: Re-run app tests**

Run: `dotnet test tests/Wif.App.Tests/Wif.App.Tests.csproj --filter PrimaryWorkspaceViewModelTests -v minimal`
Expected: PASS

- [ ] **Step 8: Build the app**

Run: `dotnet build src/Wif.App/Wif.App.csproj -v minimal`
Expected: Build succeeded

- [ ] **Step 9: Commit**

```bash
git add src/Wif.App/Views/WorkItemsView.axaml src/Wif.App/ViewModels/WorkItemsViewModel.cs tests/Wif.App.Tests/PrimaryWorkspaceViewModelTests.cs
git commit -m "feat: strengthen workitems as primary workbench"
```

---

### Task 6: Redesign WorkItem Detail as an Action-First Workspace

**Files:**
- Modify: `src/Wif.App/Views/WorkItemDetailView.axaml`
- Modify: `src/Wif.App/ViewModels/WorkItemDetailViewModel.cs`

- [ ] **Step 1: Add action-first computed properties**

In `WorkItemDetailViewModel.cs` add:

```csharp
public string SummaryTitle => CurrentItem?.Title ?? "(加载中...)";
public bool HasAiOutput => !string.IsNullOrWhiteSpace(AiAnalysisResult) || Proposals.Count > 0;
public bool HasRawContent => !string.IsNullOrWhiteSpace(RawContent) && RawContent != "(无内容)";
public string PrimaryActionLabel => CurrentItem?.Status switch
{
    WorkItemStatus.Inbox => "开始分析",
    WorkItemStatus.Waiting => "恢复推进",
    WorkItemStatus.Active => "继续处理",
    WorkItemStatus.Done => "查看复盘",
    _ => "查看详情"
};
```

- [ ] **Step 2: Build to verify the viewmodel remains clean**

Run: `dotnet build src/Wif.App/Wif.App.csproj -v minimal`
Expected: Build succeeded

- [ ] **Step 3: Replace the vertical top stack with a two-column hero section**

At the top of `WorkItemDetailView.axaml`, create:

```xml
<Grid ColumnDefinitions="2*,*" ColumnSpacing="16">
  <Border Grid.Column="0" Classes="workspace-surface">
    <!-- summary, status, source, created-at, next action hint -->
  </Border>
  <Border Grid.Column="1" Classes="workspace-surface">
    <!-- AI analyze, GIS plan, proposal actions, status transitions -->
  </Border>
</Grid>
```

- [ ] **Step 4: Move low-priority content below the hero section**

Keep these below the fold:

- raw mail content
- export buttons
- tags
- parent / child relations

- [ ] **Step 5: Keep existing command bindings intact**

Do not replace:

- `AnalyzeCommand`
- proposal commands
- GIS commands
- export commands
- tag commands
- status transition click handlers

- [ ] **Step 6: Build and manually verify**

Run: `dotnet build src/Wif.App/Wif.App.csproj -v minimal`
Expected: Build succeeded

Manual check:

- open a work item
- action area is visible before raw content
- analyze / GIS / status controls are still functional

- [ ] **Step 7: Commit**

```bash
git add src/Wif.App/Views/WorkItemDetailView.axaml src/Wif.App/ViewModels/WorkItemDetailViewModel.cs
git commit -m "feat: redesign work item detail as action-first workspace"
```

---

### Task 7: Reframe Settings into Product vs System Structure

**Files:**
- Modify: `src/Wif.App/Views/SettingsView.axaml`
- Modify: `src/Wif.App/ViewModels/SettingsViewModel.cs`

- [ ] **Step 1: Add settings grouping metadata**

In `SettingsViewModel.cs`, add grouped section descriptors:

```csharp
public IReadOnlyList<string> ProductSettingsSections => ["外观"];
public IReadOnlyList<string> SystemSettingsSections => ["邮箱账户", "AI 配置", "AI 内容策略", "模块管理", "数据存储"];
```

- [ ] **Step 2: Replace the single wrap layout with two grouped sections**

In `SettingsView.axaml`, rebuild the page into:

```xml
<StackPanel Spacing="24">
  <StackPanel>
    <TextBlock Classes="workspace-title" Text="设置" />
    <TextBlock Classes="workspace-subtitle" Text="产品偏好与系统配置" />
  </StackPanel>

  <StackPanel Spacing="12">
    <TextBlock Text="产品设置" FontSize="16" FontWeight="SemiBold" />
    <!-- appearance card -->
  </StackPanel>

  <StackPanel Spacing="12">
    <TextBlock Text="系统设置" FontSize="16" FontWeight="SemiBold" />
    <!-- remaining cards -->
  </StackPanel>
</StackPanel>
```

- [ ] **Step 3: Build and visually verify**

Run: `dotnet build src/Wif.App/Wif.App.csproj -v minimal`
Expected: Build succeeded

Manual check:

- users can distinguish product preferences from admin/system configuration
- all existing settings cards still open their dialogs

- [ ] **Step 4: Commit**

```bash
git add src/Wif.App/Views/SettingsView.axaml src/Wif.App/ViewModels/SettingsViewModel.cs
git commit -m "feat: split settings into product and system groups"
```

---

### Task 8: Verification and Plan Status Update

**Files:**
- Verify: `src/Wif.App/*`
- Verify: `tests/Wif.App.Tests/*`
- Modify: `docs/superpowers/specs/2026-04-03-ui-benchmark-fusion-design.md`

- [ ] **Step 1: Run app-layer tests**

Run: `dotnet test tests/Wif.App.Tests/Wif.App.Tests.csproj -v minimal`
Expected: PASS

- [ ] **Step 2: Build the app**

Run: `dotnet build src/Wif.App/Wif.App.csproj -v minimal`
Expected: Build succeeded

- [ ] **Step 3: Manual smoke test the primary journey**

Verify:

1. shell navigation labels are workflow-first
2. global search routes into workbench
3. inbox feels like an inflow workspace
4. work items page feels like the main workbench
5. work item detail exposes actions before raw context
6. settings hierarchy is clearer

- [ ] **Step 4: Update the UI fusion spec with execution note**

Add a short note to `docs/superpowers/specs/2026-04-03-ui-benchmark-fusion-design.md`:

```md
> Phase 1 UI fusion execution is tracked in `docs/superpowers/plans/2026-04-03-primary-workspace-ui-fusion-plan.md`.
```

- [ ] **Step 5: Final commit**

```bash
git add docs/superpowers/specs/2026-04-03-ui-benchmark-fusion-design.md
git commit -m "docs: link ui fusion spec to implementation plan"
```

---

## Notes for the Implementer

- Do not redesign GIS in this plan. Only keep shell naming and top-level consistency intact.
- Avoid introducing new UI runtime dependencies or browser-based UI tooling.
- Favor shared styles over one-off page-specific style duplication.
- If `tests/Wif.App.Tests` already exists from another plan, extend it instead of recreating it.
- If git metadata is unavailable in the local workspace snapshot, preserve the commit boundaries as logical checkpoints.

# WorkItems Dashboard UI Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Redesign the WorkItems page from a two-column list layout to a modern dashboard with summary stat cards, sort/filter toolbar, and a responsive card grid — matching the reference "Taskflow Manager" UI.

**Architecture:** Replace the current `Grid ColumnDefinitions="260,*"` layout with a vertical `DockPanel`/`ScrollViewer` containing three stacked sections: (1) stat summary row, (2) sort/filter toolbar, (3) `WrapPanel` card grid. The ViewModel gains computed stat properties (`TotalCount`, `ActiveCount`, `CompletedCount`, `IssueCount`), a sort mode, and the existing search/filter/batch functionality is preserved but relocated into the new layout.

**Tech Stack:** Avalonia UI (net9.0-windows), CommunityToolkit.Mvvm, XAML data binding, DynamicResource theming

---

## File Map

| Action | File | Responsibility |
|--------|------|----------------|
| Rewrite | `src/Wif.App/Views/WorkItemsView.axaml` | Dashboard layout: stat cards, toolbar, card grid |
| Modify | `src/Wif.App/ViewModels/WorkItemsViewModel.cs` | Add stat properties, sort enum/command, refactor load |
| Modify | `src/Wif.App/Views/WorkItemsView.axaml.cs` | Keep double-tap, FocusSearchBox; adjust element names |

---

### Task 1: Add Sort Enum and Stat Properties to ViewModel

**Files:**
- Modify: `src/Wif.App/ViewModels/WorkItemsViewModel.cs`

- [ ] **Step 1: Add SortMode enum at top of file**

Add this enum inside the namespace, before `WorkItemViewModel`:

```csharp
public enum WorkItemSortMode
{
    CreatedDesc,
    CreatedAsc,
    TitleAsc,
    StatusAsc
}
```

- [ ] **Step 2: Add stat and sort properties to `WorkItemsViewModel`**

Add these observable properties after the existing `_isAllSelected` field:

```csharp
[ObservableProperty]
private int _totalCount;

[ObservableProperty]
private int _activeCount;

[ObservableProperty]
private int _completedCount;

[ObservableProperty]
private int _issueCount;

[ObservableProperty]
private WorkItemSortMode _sortMode = WorkItemSortMode.CreatedDesc;
```

- [ ] **Step 3: Add `ComputeStats` helper method**

Add this private method after `OnIsAllSelectedChanged`:

```csharp
private void ComputeStats(IReadOnlyList<WorkItem> allItems)
{
    TotalCount = allItems.Count;
    ActiveCount = allItems.Count(i => i.Status == WorkItemStatus.Active);
    CompletedCount = allItems.Count(i => i.Status == WorkItemStatus.Done);
    // "Issue" = items in Waiting status (blocked/needs attention)
    IssueCount = allItems.Count(i => i.Status == WorkItemStatus.Waiting);
}
```

- [ ] **Step 4: Add `ApplySort` helper method**

Add after `ComputeStats`:

```csharp
private static IEnumerable<WorkItem> ApplySort(IEnumerable<WorkItem> items, WorkItemSortMode mode) =>
    mode switch
    {
        WorkItemSortMode.CreatedAsc => items.OrderBy(i => i.CreatedAt),
        WorkItemSortMode.TitleAsc => items.OrderBy(i => i.Title),
        WorkItemSortMode.StatusAsc => items.OrderBy(i => i.Status),
        _ => items.OrderByDescending(i => i.CreatedAt),
    };
```

- [ ] **Step 5: Add SortCommand**

Add after the existing `ToggleSelectAll` command:

```csharp
[RelayCommand]
private async Task SetSortAsync(WorkItemSortMode mode)
{
    SortMode = mode;
    await LoadAsync();
}
```

- [ ] **Step 6: Update `LoadAsync` to compute stats and apply sort**

Replace the body of the `try` block inside `LoadAsync` with:

```csharp
// Fetch all items (no status filter) for stats, then filter
var allItems = await _workItemService.ListAsync(status: null, take: 500, ct: CancellationToken.None);
ComputeStats(allItems);

// Apply status filter
var filtered = SelectedFilter.HasValue
    ? allItems.Where(i => i.Status == SelectedFilter.Value)
    : allItems;

// Apply sort
var sorted = ApplySort(filtered, SortMode);

Items.Clear();
foreach (var item in sorted)
{
    var vm = new WorkItemViewModel(item);
    vm.PropertyChanged += (_, e) =>
    {
        if (e.PropertyName == nameof(WorkItemViewModel.IsSelected))
        {
            OnPropertyChanged(nameof(HasSelection));
            OnPropertyChanged(nameof(SelectedCount));
        }
    };
    Items.Add(vm);
}
IsAllSelected = false;
```

- [ ] **Step 7: Verify build**

Run: `dotnet build src/Wif.App/Wif.App.csproj`
Expected: Build succeeded

- [ ] **Step 8: Commit**

```bash
git add src/Wif.App/ViewModels/WorkItemsViewModel.cs
git commit -m "feat: add stat properties and sort mode to WorkItemsViewModel"
```

---

### Task 2: Rewrite WorkItemsView.axaml — Dashboard Layout

**Files:**
- Rewrite: `src/Wif.App/Views/WorkItemsView.axaml`

This is the main UI rewrite. The new layout is a single-column scrollable page with three sections.

- [ ] **Step 1: Write the complete new AXAML**

Replace the entire contents of `WorkItemsView.axaml` with:

```xml
<UserControl xmlns="https://github.com/avaloniaui"
             xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml"
             xmlns:vm="clr-namespace:Wif.App.ViewModels"
             xmlns:enums="clr-namespace:Wif.Domain.Enums;assembly=Wif.Domain"
             x:Class="Wif.App.Views.WorkItemsView"
             x:CompileBindings="False">

  <UserControl.Styles>
    <!-- Stat card base style -->
    <Style Selector="Border.stat-card">
      <Setter Property="Background" Value="{DynamicResource SystemControlBackgroundBaseLowBrush}" />
      <Setter Property="CornerRadius" Value="12" />
      <Setter Property="Padding" Value="20,16" />
      <Setter Property="MinWidth" Value="160" />
      <Setter Property="BorderThickness" Value="1" />
      <Setter Property="BorderBrush" Value="{DynamicResource SystemControlForegroundBaseLowBrush}" />
    </Style>

    <!-- Issue card red border -->
    <Style Selector="Border.stat-card-issue">
      <Setter Property="Background" Value="{DynamicResource SystemControlBackgroundBaseLowBrush}" />
      <Setter Property="CornerRadius" Value="12" />
      <Setter Property="Padding" Value="20,16" />
      <Setter Property="MinWidth" Value="160" />
      <Setter Property="BorderThickness" Value="2" />
      <Setter Property="BorderBrush" Value="#E53E3E" />
    </Style>

    <!-- Work item card style -->
    <Style Selector="Border.work-card">
      <Setter Property="Background" Value="{DynamicResource SystemControlBackgroundBaseLowBrush}" />
      <Setter Property="CornerRadius" Value="12" />
      <Setter Property="Padding" Value="16" />
      <Setter Property="Width" Value="280" />
      <Setter Property="Height" Value="180" />
      <Setter Property="BorderThickness" Value="1" />
      <Setter Property="BorderBrush" Value="{DynamicResource SystemControlForegroundBaseLowBrush}" />
    </Style>

    <!-- New item placeholder card -->
    <Style Selector="Border.new-card">
      <Setter Property="Background" Value="Transparent" />
      <Setter Property="CornerRadius" Value="12" />
      <Setter Property="Padding" Value="16" />
      <Setter Property="Width" Value="280" />
      <Setter Property="Height" Value="180" />
      <Setter Property="BorderThickness" Value="2" />
      <Setter Property="BorderBrush" Value="{DynamicResource SystemControlForegroundBaseLowBrush}" />
      <Setter Property="BorderDashArray" Value="6,3" />
    </Style>
  </UserControl.Styles>

  <DockPanel>
    <!-- Loading bar at very top -->
    <ProgressBar DockPanel.Dock="Top"
                 IsIndeterminate="True"
                 IsVisible="{Binding IsLoading}"
                 Height="3" />

    <!-- Error/Status message -->
    <TextBlock DockPanel.Dock="Top"
               Text="{Binding ErrorMessage}"
               IsVisible="{Binding ErrorMessage, Converter={x:Static StringConverters.IsNotNullOrEmpty}}"
               Foreground="#E53E3E"
               Margin="32,8,32,0"
               TextWrapping="Wrap"
               FontSize="13" />

    <!-- Scrollable main content -->
    <ScrollViewer HorizontalScrollBarVisibility="Disabled"
                  VerticalScrollBarVisibility="Auto">
      <StackPanel Margin="32,24" Spacing="24">

        <!-- Page title -->
        <TextBlock Text="我的工作项" FontSize="24" FontWeight="Bold" />

        <!-- ═══════════ Section 1: Stat Summary Cards ═══════════ -->
        <WrapPanel Orientation="Horizontal" HorizontalAlignment="Left">
          <!-- Total -->
          <Border Classes="stat-card" Margin="0,0,12,0">
            <Grid ColumnDefinitions="*,Auto">
              <StackPanel Grid.Column="0" Spacing="4">
                <TextBlock Text="{Binding TotalCount}"
                           FontSize="28" FontWeight="Bold" />
                <TextBlock Text="全部工作项"
                           FontSize="12" Opacity="0.6" />
              </StackPanel>
              <Border Grid.Column="1"
                      Background="#EBF5FF" CornerRadius="8"
                      Padding="6" VerticalAlignment="Top">
                <TextBlock Text="&#x2630;" FontSize="16" Foreground="#3B82F6" />
              </Border>
            </Grid>
          </Border>

          <!-- Active -->
          <Border Classes="stat-card" Margin="0,0,12,0">
            <Grid ColumnDefinitions="*,Auto">
              <StackPanel Grid.Column="0" Spacing="4">
                <TextBlock Text="{Binding ActiveCount}"
                           FontSize="28" FontWeight="Bold" />
                <TextBlock Text="进行中"
                           FontSize="12" Opacity="0.6" />
              </StackPanel>
              <Border Grid.Column="1"
                      Background="#ECFDF5" CornerRadius="8"
                      Padding="6" VerticalAlignment="Top">
                <TextBlock Text="&#x25B6;" FontSize="16" Foreground="#10B981" />
              </Border>
            </Grid>
          </Border>

          <!-- Completed -->
          <Border Classes="stat-card" Margin="0,0,12,0">
            <Grid ColumnDefinitions="*,Auto">
              <StackPanel Grid.Column="0" Spacing="4">
                <TextBlock Text="{Binding CompletedCount}"
                           FontSize="28" FontWeight="Bold" />
                <TextBlock Text="已完成"
                           FontSize="12" Opacity="0.6" />
              </StackPanel>
              <Border Grid.Column="1"
                      Background="#ECFDF5" CornerRadius="8"
                      Padding="6" VerticalAlignment="Top">
                <TextBlock Text="&#x2714;" FontSize="16" Foreground="#10B981" />
              </Border>
            </Grid>
          </Border>

          <!-- Issue -->
          <Border Classes="stat-card-issue">
            <Grid ColumnDefinitions="*,Auto">
              <StackPanel Grid.Column="0" Spacing="4">
                <TextBlock Text="{Binding IssueCount}"
                           FontSize="28" FontWeight="Bold" />
                <TextBlock Text="待处理"
                           FontSize="12" Opacity="0.6" />
              </StackPanel>
              <Border Grid.Column="1"
                      Background="#FEF2F2" CornerRadius="8"
                      Padding="6" VerticalAlignment="Top">
                <TextBlock Text="&#x26A0;" FontSize="16" Foreground="#E53E3E" />
              </Border>
            </Grid>
          </Border>
        </WrapPanel>

        <!-- ═══════════ Section 2: Sort / Filter / Search Toolbar ═══════════ -->
        <WrapPanel Orientation="Horizontal" HorizontalAlignment="Left">
          <!-- Sort dropdown -->
          <DropDownButton CornerRadius="8" Padding="10,6" Margin="0,0,8,0">
            <DropDownButton.Content>
              <StackPanel Orientation="Horizontal" Spacing="6">
                <TextBlock Text="排序:" Opacity="0.7" />
                <TextBlock Text="{Binding SortMode}" />
              </StackPanel>
            </DropDownButton.Content>
            <DropDownButton.Flyout>
              <MenuFlyout Placement="Bottom">
                <MenuItem Header="最新创建"
                          Command="{Binding SetSortCommand}"
                          CommandParameter="{x:Static vm:WorkItemSortMode.CreatedDesc}" />
                <MenuItem Header="最早创建"
                          Command="{Binding SetSortCommand}"
                          CommandParameter="{x:Static vm:WorkItemSortMode.CreatedAsc}" />
                <MenuItem Header="标题 A-Z"
                          Command="{Binding SetSortCommand}"
                          CommandParameter="{x:Static vm:WorkItemSortMode.TitleAsc}" />
                <MenuItem Header="按状态"
                          Command="{Binding SetSortCommand}"
                          CommandParameter="{x:Static vm:WorkItemSortMode.StatusAsc}" />
              </MenuFlyout>
            </DropDownButton.Flyout>
          </DropDownButton>

          <!-- Filter dropdown -->
          <DropDownButton CornerRadius="8" Padding="10,6" Margin="0,0,8,0">
            <DropDownButton.Content>
              <StackPanel Orientation="Horizontal" Spacing="6">
                <TextBlock Text="筛选:" Opacity="0.7" />
                <TextBlock Text="{Binding SelectedFilter, TargetNullValue='全部'}" />
              </StackPanel>
            </DropDownButton.Content>
            <DropDownButton.Flyout>
              <MenuFlyout Placement="Bottom">
                <MenuItem Header="全部"
                          Command="{Binding FilterCommand}"
                          CommandParameter="{x:Null}" />
                <MenuItem Header="收件箱"
                          Command="{Binding FilterCommand}"
                          CommandParameter="{x:Static enums:WorkItemStatus.Inbox}" />
                <MenuItem Header="进行中"
                          Command="{Binding FilterCommand}"
                          CommandParameter="{x:Static enums:WorkItemStatus.Active}" />
                <MenuItem Header="等待中"
                          Command="{Binding FilterCommand}"
                          CommandParameter="{x:Static enums:WorkItemStatus.Waiting}" />
                <MenuItem Header="已完成"
                          Command="{Binding FilterCommand}"
                          CommandParameter="{x:Static enums:WorkItemStatus.Done}" />
                <MenuItem Header="已归档"
                          Command="{Binding FilterCommand}"
                          CommandParameter="{x:Static enums:WorkItemStatus.Archived}" />
              </MenuFlyout>
            </DropDownButton.Flyout>
          </DropDownButton>

          <!-- Search box -->
          <TextBox x:Name="SearchBox"
                   Text="{Binding SearchQuery, Mode=TwoWay}"
                   Watermark="搜索工作项..."
                   Width="220"
                   CornerRadius="8"
                   Margin="0,0,8,0" />

          <Button Content="搜索"
                  Command="{Binding SearchCommand}"
                  CornerRadius="8"
                  Padding="12,6"
                  Margin="0,0,16,0" />

          <!-- Batch mode toggle -->
          <Button Content="{Binding BatchModeButtonText}"
                  Command="{Binding ToggleBatchModeCommand}"
                  CornerRadius="8"
                  Padding="12,6"
                  Margin="0,0,8,0" />

          <!-- Select All (visible in batch mode) -->
          <Button Content="{Binding SelectAllButtonText}"
                  Command="{Binding ToggleSelectAllCommand}"
                  IsVisible="{Binding IsBatchMode}"
                  CornerRadius="8"
                  Padding="12,6" />
        </WrapPanel>

        <!-- Batch operations bar (visible when batch mode active) -->
        <Border Background="{DynamicResource SystemControlBackgroundBaseMediumLowBrush}"
                CornerRadius="10"
                Padding="16,10"
                IsVisible="{Binding IsBatchMode}">
          <StackPanel Orientation="Horizontal" Spacing="12" HorizontalAlignment="Left">
            <TextBlock Text="{Binding SelectedCount, StringFormat='已选择 {0} 项'}"
                       VerticalAlignment="Center"
                       FontWeight="SemiBold" />
            <Button Content="设为等待中"
                    Command="{Binding BatchSetWaitingCommand}"
                    CornerRadius="8" Padding="10,4" />
            <Button Content="设为已完成"
                    Command="{Binding BatchSetDoneCommand}"
                    CornerRadius="8" Padding="10,4" />
            <Button Content="设为已归档"
                    Command="{Binding BatchSetArchivedCommand}"
                    CornerRadius="8" Padding="10,4" />
            <Button Content="取消"
                    Command="{Binding ToggleBatchModeCommand}"
                    CornerRadius="8" Padding="10,4" />
          </StackPanel>
        </Border>

        <!-- Item count -->
        <StackPanel Orientation="Horizontal" Spacing="4" Opacity="0.6">
          <TextBlock Text="共" FontSize="13" />
          <TextBlock Text="{Binding Items.Count}" FontSize="13" FontWeight="SemiBold" />
          <TextBlock Text="项" FontSize="13" />
        </StackPanel>

        <!-- ═══════════ Section 3: Card Grid ═══════════ -->
        <ItemsControl x:Name="ItemsList"
                      ItemsSource="{Binding Items}">
          <ItemsControl.ItemsPanel>
            <ItemsPanelTemplate>
              <WrapPanel Orientation="Horizontal" />
            </ItemsPanelTemplate>
          </ItemsControl.ItemsPanel>
          <ItemsControl.ItemTemplate>
            <DataTemplate>
              <Border Classes="work-card" Margin="0,0,12,12"
                      DoubleTapped="OnCardDoubleTapped">
                <Grid RowDefinitions="Auto,*,Auto">
                  <!-- Row 0: Icon + Title + Checkbox/Menu -->
                  <Grid Grid.Row="0" ColumnDefinitions="Auto,*,Auto">
                    <!-- Source type icon circle -->
                    <Border Grid.Column="0"
                            Width="32" Height="32"
                            CornerRadius="16"
                            Background="#3B82F6"
                            Margin="0,0,10,0"
                            VerticalAlignment="Top">
                      <TextBlock Text="{Binding Source.Type, Converter={x:Static vm:SourceTypeIconConverter.Instance}}"
                                 Foreground="White"
                                 FontSize="14"
                                 HorizontalAlignment="Center"
                                 VerticalAlignment="Center" />
                    </Border>

                    <!-- Title -->
                    <TextBlock Grid.Column="1"
                               Text="{Binding Title}"
                               FontWeight="SemiBold"
                               FontSize="14"
                               TextTrimming="CharacterEllipsis"
                               MaxLines="2"
                               TextWrapping="Wrap"
                               VerticalAlignment="Top" />

                    <!-- Batch checkbox OR three-dot menu -->
                    <Panel Grid.Column="2" VerticalAlignment="Top">
                      <CheckBox IsChecked="{Binding IsSelected, Mode=TwoWay}"
                                IsVisible="{Binding DataContext.IsBatchMode, RelativeSource={RelativeSource AncestorType=UserControl}}"
                                Margin="4,0,0,0" />
                    </Panel>
                  </Grid>

                  <!-- Row 1: Status badge -->
                  <StackPanel Grid.Row="1"
                              Orientation="Horizontal"
                              Spacing="8"
                              VerticalAlignment="Top"
                              Margin="0,8,0,0">
                    <!-- Status badge -->
                    <Border CornerRadius="4"
                            Padding="8,2"
                            Background="{Binding Status, Converter={x:Static vm:StatusColorConverter.Instance}}">
                      <TextBlock Text="{Binding Status, Converter={x:Static vm:StatusTextConverter.Instance}}"
                                 FontSize="11"
                                 Foreground="White"
                                 FontWeight="SemiBold" />
                    </Border>
                  </StackPanel>

                  <!-- Row 2: Bottom info (date + source) -->
                  <StackPanel Grid.Row="2"
                              Orientation="Horizontal"
                              Spacing="12">
                    <TextBlock Text="{Binding CreatedAt, StringFormat='{}{0:yyyy-MM-dd HH:mm}'}"
                               FontSize="11"
                               Opacity="0.5" />
                    <TextBlock Text="{Binding Source.Type}"
                               FontSize="11"
                               Opacity="0.5" />
                  </StackPanel>
                </Grid>
              </Border>
            </DataTemplate>
          </ItemsControl.ItemTemplate>
        </ItemsControl>

        <!-- "+ New" placeholder card -->
        <WrapPanel Orientation="Horizontal">
          <Button Command="{Binding CreateNewItemCommand}"
                  Background="Transparent"
                  BorderThickness="0"
                  Padding="0"
                  Cursor="Hand">
            <Border Classes="new-card">
              <StackPanel HorizontalAlignment="Center"
                          VerticalAlignment="Center"
                          Spacing="8">
                <TextBlock Text="+"
                           FontSize="32"
                           HorizontalAlignment="Center"
                           Opacity="0.5" />
                <TextBlock Text="新建工作项"
                           FontSize="14"
                           HorizontalAlignment="Center"
                           Opacity="0.5" />
              </StackPanel>
            </Border>
          </Button>
        </WrapPanel>

      </StackPanel>
    </ScrollViewer>
  </DockPanel>
</UserControl>
```

- [ ] **Step 2: Verify build**

Run: `dotnet build src/Wif.App/Wif.App.csproj`
Expected: Build will FAIL — missing converters (`SourceTypeIconConverter`, `StatusColorConverter`, `StatusTextConverter`). This is expected; we create them in Task 3.

- [ ] **Step 3: Commit the AXAML (WIP)**

```bash
git add src/Wif.App/Views/WorkItemsView.axaml
git commit -m "feat(wip): rewrite WorkItemsView to dashboard card grid layout"
```

---

### Task 3: Create Value Converters for Card Display

**Files:**
- Create: `src/Wif.App/ViewModels/WorkItemConverters.cs`

The AXAML references three static converter instances. We define them in a single file.

- [ ] **Step 1: Create `WorkItemConverters.cs`**

```csharp
using System.Globalization;
using Avalonia.Data.Converters;
using Avalonia.Media;
using Wif.Domain.Enums;

namespace Wif.App.ViewModels;

/// <summary>
/// Converts SourceType to a single-char icon for the card circle.
/// </summary>
public sealed class SourceTypeIconConverter : IValueConverter
{
    public static readonly SourceTypeIconConverter Instance = new();

    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture) =>
        value switch
        {
            SourceType.Email => "\u2709",   // ✉
            SourceType.File => "\uD83D\uDCC4",    // 📄
            SourceType.Api => "\uD83D\uDD17",     // 🔗
            SourceType.Manual => "\u270F",  // ✏
            _ => "?"
        };

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
        => throw new NotSupportedException();
}

/// <summary>
/// Converts WorkItemStatus to a background brush for the status badge.
/// </summary>
public sealed class StatusColorConverter : IValueConverter
{
    public static readonly StatusColorConverter Instance = new();

    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture) =>
        value switch
        {
            WorkItemStatus.Inbox => new SolidColorBrush(Color.Parse("#3B82F6")),
            WorkItemStatus.Active => new SolidColorBrush(Color.Parse("#3B82F6")),
            WorkItemStatus.Waiting => new SolidColorBrush(Color.Parse("#F59E0B")),
            WorkItemStatus.Done => new SolidColorBrush(Color.Parse("#10B981")),
            WorkItemStatus.Archived => new SolidColorBrush(Color.Parse("#6B7280")),
            _ => new SolidColorBrush(Color.Parse("#6B7280"))
        };

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
        => throw new NotSupportedException();
}

/// <summary>
/// Converts WorkItemStatus to Chinese display text for the status badge.
/// </summary>
public sealed class StatusTextConverter : IValueConverter
{
    public static readonly StatusTextConverter Instance = new();

    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture) =>
        value switch
        {
            WorkItemStatus.Inbox => "收件箱",
            WorkItemStatus.Active => "进行中",
            WorkItemStatus.Waiting => "等待中",
            WorkItemStatus.Done => "已完成",
            WorkItemStatus.Archived => "已归档",
            _ => "未知"
        };

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
        => throw new NotSupportedException();
}
```

- [ ] **Step 2: Verify build**

Run: `dotnet build src/Wif.App/Wif.App.csproj`
Expected: Build succeeded

- [ ] **Step 3: Commit**

```bash
git add src/Wif.App/ViewModels/WorkItemConverters.cs
git commit -m "feat: add status/source-type converters for work item cards"
```

---

### Task 4: Update Code-Behind for New Layout

**Files:**
- Modify: `src/Wif.App/Views/WorkItemsView.axaml.cs`

The old code-behind references `ItemsList` (a ListBox with `SelectedItem`). The new layout uses an `ItemsControl` (no selection), and card double-tap is handled per-card via `DoubleTapped` on the Border.

- [ ] **Step 1: Replace the code-behind**

Replace the entire file with:

```csharp
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Threading;
using Avalonia.VisualTree;
using Microsoft.Extensions.DependencyInjection;
using Wif.App.ViewModels;

namespace Wif.App.Views;

public partial class WorkItemsView : UserControl
{
    public WorkItemsView()
    {
        InitializeComponent();
        DataContext = App.Services.GetRequiredService<WorkItemsViewModel>();
        Loaded += async (_, _) =>
        {
            try
            {
                if (DataContext is WorkItemsViewModel vm)
                    await vm.LoadCommand.ExecuteAsync(null);
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"[ERROR] WorkItemsView Loaded failed: {ex}");
            }
        };
    }

    private void OnCardDoubleTapped(object? sender, TappedEventArgs e)
    {
        if (sender is not Border { DataContext: WorkItemViewModel vm })
            return;

        var mainWindow = this.FindAncestorOfType<MainWindow>();
        mainWindow?.NavigateToWorkItemDetail(vm.Id);
    }

    public void FocusSearchBox()
    {
        Dispatcher.UIThread.Post(() =>
        {
            SearchBox.Focus();
            SearchBox.SelectAll();
        });
    }
}
```

- [ ] **Step 2: Verify build**

Run: `dotnet build src/Wif.App/Wif.App.csproj`
Expected: Build succeeded

- [ ] **Step 3: Commit**

```bash
git add src/Wif.App/Views/WorkItemsView.axaml.cs
git commit -m "feat: update WorkItemsView code-behind for card grid layout"
```

---

### Task 5: Visual Polish and Verification

**Files:**
- Modify: `src/Wif.App/Views/WorkItemsView.axaml` (minor tweaks if needed)
- Modify: `src/Wif.App/ViewModels/WorkItemsViewModel.cs` (minor tweaks if needed)

- [ ] **Step 1: Run the application**

Run: `dotnet run --project src/Wif.App/Wif.App.csproj`

Verify:
1. Dashboard page loads with stat cards row at top
2. Sort and filter dropdowns work
3. Work item cards display in a wrap grid
4. Double-click a card opens detail view
5. Batch mode toggle works (checkboxes appear on cards)
6. "+ 新建工作项" placeholder card creates a new item
7. Search box filters items
8. Stats update after filtering

- [ ] **Step 2: Fix any visual issues found during testing**

Common things to adjust:
- Card width/height if content overflows
- Stat card spacing on narrow windows
- Icon rendering (Unicode fallback if system font doesn't have the glyph)

- [ ] **Step 3: Run tests**

Run: `dotnet test Wif.sln`
Expected: All existing tests pass (UI changes don't break domain/service tests)

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "feat: complete work items dashboard card grid UI redesign"
```

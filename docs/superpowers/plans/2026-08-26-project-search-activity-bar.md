# Project Search Activity Bar Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move Project Search from the status bar into the left Activity Bar after Git, while pinning Collab Panel to the bottom of the rail.

**Architecture:** Extend workspace panel metadata with top/bottom Activity Bar placement and let vertical `PanelButtons` merge panel controls with externally registered, priority-ordered views. Keep Project Search rendering in the `search` crate, register it from `zed`, and preserve all horizontal dock behavior.

**Tech Stack:** Rust, GPUI, Zed workspace/search/collab UI crates, cargo tests, project Clippy script.

---

### Task 1: Add reusable vertical Activity Bar ordering

**Files:**
- Modify: `crates/workspace/src/dock.rs:36-190`
- Modify: `crates/workspace/src/dock.rs:380-440`
- Modify: `crates/workspace/src/dock.rs:1420-1680`
- Modify: `crates/workspace/src/workspace.rs:5900-6050`
- Test: `crates/workspace/src/dock.rs:1700-1760`

- [ ] **Step 1: Write failing ordering tests**

Add tests for a generic arrangement helper before defining it:

```rust
#[test]
fn vertical_activity_bar_orders_top_items_and_separates_bottom_items() {
    let entries = vec![
        ActivityBarEntry::new(6, ActivityBarPlacement::Top, "outline"),
        ActivityBarEntry::new(5, ActivityBarPlacement::Bottom, "collab"),
        ActivityBarEntry::new(4, ActivityBarPlacement::Top, "search"),
        ActivityBarEntry::new(1, ActivityBarPlacement::Top, "project"),
        ActivityBarEntry::new(3, ActivityBarPlacement::Top, "git"),
    ];

    let (top, bottom) = arrange_activity_bar_entries(entries);

    assert_eq!(top, vec!["project", "git", "search", "outline"]);
    assert_eq!(bottom, vec!["collab"]);
}

#[test]
fn horizontal_panel_buttons_keep_default_top_placement() {
    assert_eq!(TestPanel::activity_bar_placement(), ActivityBarPlacement::Top);
}
```

- [ ] **Step 2: Run the tests and verify RED**

Run:

```bash
cargo test -p workspace vertical_activity_bar_orders_top_items_and_separates_bottom_items
```

Expected: compilation fails because `ActivityBarEntry`, `ActivityBarPlacement`, and `arrange_activity_bar_entries` do not exist.

- [ ] **Step 3: Add placement metadata and the tested arrangement helper**

Add workspace-owned metadata and forward it through `PanelHandle`:

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ActivityBarPlacement {
    #[default]
    Top,
    Bottom,
}

pub trait Panel: Focusable + EventEmitter<PanelEvent> + Render + Sized {
    fn activity_bar_placement() -> ActivityBarPlacement {
        ActivityBarPlacement::Top
    }
}

pub trait PanelHandle: Send + Sync {
    fn activity_bar_placement(&self) -> ActivityBarPlacement;
}

impl<T> PanelHandle for Entity<T>
where
    T: Panel,
{
    fn activity_bar_placement(&self) -> ActivityBarPlacement {
        T::activity_bar_placement()
    }
}
```

Implement the generic helper used by production rendering and tests:

```rust
struct ActivityBarEntry<T> {
    priority: u32,
    placement: ActivityBarPlacement,
    content: T,
}

impl<T> ActivityBarEntry<T> {
    fn new(priority: u32, placement: ActivityBarPlacement, content: T) -> Self {
        Self { priority, placement, content }
    }
}

fn arrange_activity_bar_entries<T>(
    mut entries: Vec<ActivityBarEntry<T>>,
) -> (Vec<T>, Vec<T>) {
    entries.sort_by_key(|entry| entry.priority);
    let mut top = Vec::new();
    let mut bottom = Vec::new();
    for entry in entries {
        match entry.placement {
            ActivityBarPlacement::Top => top.push(entry.content),
            ActivityBarPlacement::Bottom => bottom.push(entry.content),
        }
    }
    (top, bottom)
}
```

- [ ] **Step 4: Let vertical PanelButtons accept external priority-ordered views**

Store type-erased external items only on the vertical instance:

```rust
struct ExternalActivityBarItem {
    priority: u32,
    view: AnyView,
}

pub struct PanelButtons {
    dock: Entity<Dock>,
    orientation: PanelButtonsOrientation,
    external_activity_bar_items: Vec<ExternalActivityBarItem>,
    _settings_subscription: Subscription,
}

// Initialize `external_activity_bar_items` with `Vec::new()` in
// `PanelButtons::with_orientation`.

pub fn add_activity_bar_item<V: Render>(
    &mut self,
    priority: u32,
    item: Entity<V>,
    cx: &mut Context<Self>,
) {
    self.external_activity_bar_items.push(ExternalActivityBarItem {
        priority,
        view: item.into(),
    });
    cx.notify();
}
```

For horizontal orientation, render the existing panel button vector unchanged. For vertical orientation, convert panel buttons and external views into `ActivityBarEntry<AnyElement>`, call `arrange_activity_bar_entries`, and render two centered `v_flex` groups inside a `.justify_between()` full-height container. Keep `.py_1()` and `.gap_1()` on the two groups so button sizing and spacing remain unchanged.

- [ ] **Step 5: Expose registration through Workspace**

Add a crate-boundary-safe method:

```rust
pub fn add_left_dock_activity_bar_item<V: Render>(
    &mut self,
    priority: u32,
    item: Entity<V>,
    cx: &mut Context<Self>,
) {
    self.left_dock_buttons.update(cx, |buttons, cx| {
        buttons.add_activity_bar_item(priority, item, cx);
    });
}
```

- [ ] **Step 6: Run focused and full workspace tests**

Run:

```bash
cargo test -p workspace vertical_activity_bar
cargo test -p workspace panel_button_orientation
cargo test -p workspace
```

Expected: the focused tests pass and all workspace tests pass.

- [ ] **Step 7: Commit Task 1**

```bash
git add crates/workspace/src/dock.rs crates/workspace/src/workspace.rs
git commit -m "Add activity bar item ordering"
```

### Task 2: Pin Collab Panel to the bottom section

**Files:**
- Modify: `crates/collab_ui/src/collab_panel.rs:3934-4015`
- Test: `crates/collab_ui/src/collab_panel.rs`

- [ ] **Step 1: Write a failing Collab placement test**

Add a test that exercises the trait metadata without constructing the panel:

```rust
#[test]
fn collab_panel_uses_bottom_activity_bar_placement() {
    assert_eq!(
        <CollabPanel as Panel>::activity_bar_placement(),
        workspace::ActivityBarPlacement::Bottom,
    );
}
```

- [ ] **Step 2: Run the test and verify RED**

Run:

```bash
cargo test -p collab_ui collab_panel_uses_bottom_activity_bar_placement
```

Expected: assertion fails because `CollabPanel` still inherits `Top`.

- [ ] **Step 3: Override Collab Panel placement**

Add to `impl Panel for CollabPanel`:

```rust
fn activity_bar_placement() -> workspace::ActivityBarPlacement {
    workspace::ActivityBarPlacement::Bottom
}
```

- [ ] **Step 4: Run focused and crate tests**

Run:

```bash
cargo test -p collab_ui collab_panel_uses_bottom_activity_bar_placement
cargo test -p collab_ui
```

Expected: both commands pass.

- [ ] **Step 5: Commit Task 2**

```bash
git add crates/collab_ui/src/collab_panel.rs
git commit -m "Pin collaboration panel in activity bar"
```

### Task 3: Move Project Search from status bar to Activity Bar

**Files:**
- Modify: `crates/search/src/search_status_button.rs:1-80`
- Modify: `crates/zed/src/zed.rs:590-645`
- Test: `crates/search/src/search_status_button.rs`

- [ ] **Step 1: Write failing presentation metric tests**

Extract presentation-specific visuals and test both modes first:

```rust
#[test]
fn activity_bar_search_uses_vertical_button_metrics() {
    let visuals = SearchButtonPresentation::ActivityBar.visuals();
    assert_eq!(visuals.icon_size, IconSize::Custom(ui::rems_from_px(18_f32)));
    assert_eq!(visuals.button_size, Some(ButtonSize::Large));
    assert_eq!(visuals.width, Some(px(34.)));
}

#[test]
fn status_bar_search_keeps_existing_metrics() {
    let visuals = SearchButtonPresentation::StatusBar.visuals();
    assert_eq!(visuals.icon_size, IconSize::Small);
    assert_eq!(visuals.button_size, None);
    assert_eq!(visuals.width, None);
}
```

- [ ] **Step 2: Run the test and verify RED**

Run:

```bash
cargo test -p search search_button
```

Expected: compilation fails because the presentation and visual helper types do not exist.

- [ ] **Step 3: Implement the Activity Bar presentation**

Add presentation state and a weak workspace handle:

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum SearchButtonPresentation {
    #[default]
    StatusBar,
    ActivityBar,
}

#[derive(Clone, Copy)]
struct SearchButtonVisuals {
    icon_size: IconSize,
    button_size: Option<ButtonSize>,
    width: Option<Pixels>,
}

impl SearchButtonPresentation {
    fn visuals(self) -> SearchButtonVisuals {
        match self {
            Self::StatusBar => SearchButtonVisuals {
                icon_size: IconSize::Small,
                button_size: None,
                width: None,
            },
            Self::ActivityBar => SearchButtonVisuals {
                icon_size: IconSize::Custom(ui::rems_from_px(18_f32)),
                button_size: Some(ButtonSize::Large),
                width: Some(px(34.)),
            },
        }
    }
}

pub struct SearchButton {
    pane_item_focus_handle: Option<FocusHandle>,
    presentation: SearchButtonPresentation,
    workspace: Option<WeakEntity<workspace::Workspace>>,
}

pub fn activity_bar(workspace: WeakEntity<workspace::Workspace>) -> Self {
    Self {
        pane_item_focus_handle: None,
        presentation: SearchButtonPresentation::ActivityBar,
        workspace: Some(workspace),
    }
}
```

Apply the established Activity Bar metrics to the `IconButton`. In the tooltip closure, prefer resolving `workspace.active_pane().active_item()` and its focus handle at tooltip display time, then fall back to `pane_item_focus_handle`. Continue dispatching `workspace::DeploySearch` and checking `EditorSettings::get_global(cx).search.button`.

- [ ] **Step 4: Register Search after Git and remove status registration**

In `zed.rs`, construct the Activity Bar presentation and register priority `4`:

```rust
let search_button = cx.new(|_| {
    search::search_status_button::SearchButton::activity_bar(workspace_handle.downgrade())
});
workspace.add_left_dock_activity_bar_item(4, search_button, cx);
```

Delete this status-bar registration:

```rust
status_bar.add_left_item(search_button, window, cx);
```

Do not alter the remaining left status items.

- [ ] **Step 5: Run affected tests and build the application layer**

Run:

```bash
cargo test -p search search_button
cargo test -p workspace
cargo check -p zed
```

Expected: all commands pass, and `zed.rs` has exactly one `search_button` registration in the Activity Bar path.

- [ ] **Step 6: Commit Task 3**

```bash
git add crates/search/src/search_status_button.rs crates/zed/src/zed.rs
git commit -m "Move project search into activity bar"
```

### Task 4: Final verification and local visual check

**Files:**
- Verify: `README.md`
- Verify: all files changed since `ecdce42116`

- [ ] **Step 1: Verify the README review guard and clean diff**

Run:

```bash
sed -n '1,2p' README.md
git diff --check ecdce42116...HEAD
git status --short
```

Expected: the mandatory two-line review guard is present, diff check succeeds, and there are no uncommitted tracked files.

- [ ] **Step 2: Run full affected test suites**

```bash
cargo test -p workspace
cargo test -p search
cargo test -p collab_ui
```

Expected: all tests pass with zero failures.

- [ ] **Step 3: Run build, Clippy, and formatting checks**

```bash
cargo build -p workspace
cargo check -p zed
./script/clippy -p workspace -p search -p collab_ui -p zed
cargo fmt --all -- --check
```

Expected: every command exits successfully.

- [ ] **Step 4: Launch the local app and inspect the rail**

Run:

```bash
cargo run --features gpui_platform/runtime_shaders -- /Users/bytedance/Documents/life_crm
```

Verify visually:

- Project, Git, and Project Search appear in that order in the top group.
- Other ordinary panel buttons follow Search in priority order.
- Collab Panel is anchored at the bottom of the 44px rail.
- Project Search no longer appears in the status bar.
- Search opens the existing Project Search UI.
- Selected panel styling and rail colors remain unchanged.

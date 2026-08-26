# Left Dock Visual Refinement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the vertical left Activity Bar easier to target and visually distinct while retaining a compact footprint and a neutral selected state.

**Architecture:** Extend the existing orientation-specific presentation logic in `PanelButtons` with a small immutable visual-metrics value. Apply the larger metrics only to vertical buttons, then update the workspace's single Activity Bar width and theme surface so all existing dock state, panel registration, and interaction logic remain unchanged.

**Tech Stack:** Rust, GPUI, Zed `workspace` crate, existing `IconButton` and theme tokens, GPUI tests.

---

## File structure

- Modify `crates/workspace/src/dock.rs` to define and apply orientation-specific icon, button, and width metrics and to test that horizontal buttons remain unchanged.
- Modify `crates/workspace/src/workspace.rs` to widen the Activity Bar to 44 pixels, use the theme's general background surface, and update the rendered-bounds regression test.
- Verify `docs/superpowers/specs/2026-08-26-left-dock-activity-bar-design.md` as the acceptance specification.
- Do not modify individual panel crates, theme schemas, settings schemas, or status-bar behavior.

## Task 0: Confirm the repository guard and baseline

**Files:**

- Verify: `README.md`
- Verify: `crates/workspace/src/dock.rs`
- Verify: `crates/workspace/src/workspace.rs`

- [ ] **Step 1: Confirm the mandatory review notice is present**

Run:

```bash
sed -n '1,2p' README.md
```

Expected:

```text
> [!IMPORTANT]
> Remove this line to confirm you've reviewed this PR before submitting.
```

Do not remove or rewrite these lines.

- [ ] **Step 2: Confirm the worktree contains no unrelated tracked edits**

Run:

```bash
git status --short
```

Expected: only the ignored or untracked `.superpowers/` visual-companion session may appear. Do not stage it. There must be no modified source file before implementation starts.

- [ ] **Step 3: Run the current focused tests as a baseline**

Run:

```bash
cargo test -p workspace panel_button_orientation_selects -- --nocapture
cargo test -p workspace test_workspace_content_bounds_exclude_left_dock_activity_bar -- --nocapture
```

Expected: the existing orientation tests and the current 40-pixel bounds test pass.

## Task 1: Enlarge only the vertical panel buttons

**Files:**

- Modify: `crates/workspace/src/dock.rs:387-430`
- Modify: `crates/workspace/src/dock.rs:1430-1595`
- Test: `crates/workspace/src/dock.rs:1680-1715`

- [ ] **Step 1: Add a failing orientation-metrics test**

Add this test after `panel_button_orientation_selects_neutral_selected_style`:

```rust
#[test]
fn panel_button_orientation_selects_compact_enlarged_vertical_metrics() {
    let horizontal = PanelButtonsOrientation::Horizontal.button_visuals();
    assert!(horizontal.icon_size == ui::IconSize::Small);
    assert!(horizontal.button_size.is_none());
    assert_eq!(horizontal.width, None);

    let vertical = PanelButtonsOrientation::Vertical.button_visuals();
    assert!(vertical.icon_size == ui::IconSize::Custom(gpui::rems_from_px(18.)));
    assert!(vertical.button_size == Some(ui::ButtonSize::Large));
    assert_eq!(vertical.width, Some(px(34.)));
}
```

- [ ] **Step 2: Run the focused test and verify the missing helper is reported**

Run:

```bash
cargo test -p workspace panel_button_orientation_selects_compact_enlarged_vertical_metrics -- --nocapture
```

Expected: compilation fails because `button_visuals` does not exist yet. Fix environment failures before continuing.

- [ ] **Step 3: Define the orientation-specific visual metrics**

Add this type next to `PanelButtonsOrientation`:

```rust
#[derive(Clone, Copy)]
struct PanelButtonVisuals {
    icon_size: ui::IconSize,
    button_size: Option<ui::ButtonSize>,
    width: Option<Pixels>,
}
```

Add this method to `impl PanelButtonsOrientation`:

```rust
fn button_visuals(self) -> PanelButtonVisuals {
    match self {
        Self::Horizontal => PanelButtonVisuals {
            icon_size: ui::IconSize::Small,
            button_size: None,
            width: None,
        },
        Self::Vertical => PanelButtonVisuals {
            icon_size: ui::IconSize::Custom(gpui::rems_from_px(18.)),
            button_size: Some(ui::ButtonSize::Large),
            width: Some(px(34.)),
        },
    }
}
```

The horizontal values deliberately preserve the current 14-pixel icon and default button dimensions. The vertical values produce an 18-pixel icon, a 32-pixel large-button height, and a 34-pixel width.

- [ ] **Step 4: Apply the metrics to the shared button builder**

In `Render for PanelButtons`, calculate the immutable visuals next to `selected_button_style`:

```rust
let selected_button_style = self.orientation.selected_button_style();
let button_visuals = self.orientation.button_visuals();
```

Replace the existing fixed icon-size call in the `IconButton` chain with:

```rust
.icon_size(button_visuals.icon_size)
.when_some(button_visuals.button_size, |this, size| this.size(size))
.when_some(button_visuals.width, |this, width| this.width(width))
```

Keep `.toggle_state(is_active_button)` and the orientation-specific `ButtonStyle::Filled` call unchanged. Do not add a custom color or selection marker.

- [ ] **Step 5: Run focused and regression tests**

Run:

```bash
cargo test -p workspace panel_button_orientation_selects -- --nocapture
cargo fmt --all -- --check
git diff --check
```

Expected: all orientation tests pass, formatting is clean, and no whitespace errors are reported.

- [ ] **Step 6: Commit the vertical-button refinement**

```bash
git add crates/workspace/src/dock.rs
git commit -m "Refine activity bar button styling"
```

## Task 2: Widen and separate the Activity Bar surface

**Files:**

- Modify: `crates/workspace/src/workspace.rs:172-176`
- Modify: `crates/workspace/src/workspace.rs:9355-9372`
- Test: `crates/workspace/src/workspace.rs:17715-17740`

- [ ] **Step 1: Change the bounds test first**

In `test_workspace_content_bounds_exclude_left_dock_activity_bar`, replace the current width expectation:

```rust
assert_eq!(activity_bar_bounds.size.width, px(44.));
```

- [ ] **Step 2: Run the focused bounds test and verify the red state**

Run:

```bash
cargo test -p workspace test_workspace_content_bounds_exclude_left_dock_activity_bar -- --nocapture
```

Expected: the assertion fails with an actual width of 40 pixels and an expected width of 44 pixels.

- [ ] **Step 3: Update the shared Activity Bar width**

Change the existing constant to:

```rust
const LEFT_DOCK_ACTIVITY_BAR_WIDTH: Pixels = px(44.);
```

Do not add a second width literal. The same constant must continue to drive both the rendered rail and the inset workspace canvas so dock resizing and flexible sizing remain in the content coordinate system.

- [ ] **Step 4: Use the distinct theme surface**

In the `left-dock-activity-bar` element chain, replace:

```rust
.bg(colors.panel_background)
```

with:

```rust
.bg(colors.background)
```

Retain the existing right border and `colors.border`. Adjacent dock content continues to use `panel_background`, yielding a theme-aware one-step surface distinction in light and dark themes.

- [ ] **Step 5: Run focused layout and dock-sizing regressions**

Run:

```bash
cargo test -p workspace test_workspace_content_bounds_exclude_left_dock_activity_bar -- --nocapture
cargo test -p workspace test_flexible_dock_sizing -- --nocapture
cargo fmt --all -- --check
git diff --check
```

Expected: the 44-pixel bounds test and the existing flexible dock-sizing regression pass, formatting is clean, and no whitespace errors are reported.

- [ ] **Step 6: Commit the Activity Bar surface refinement**

```bash
git add crates/workspace/src/workspace.rs
git commit -m "Refine activity bar surface"
```

## Task 3: Complete automated and local visual verification

**Files:**

- Verify: `crates/workspace/src/dock.rs`
- Verify: `crates/workspace/src/workspace.rs`
- Verify: `docs/superpowers/specs/2026-08-26-left-dock-activity-bar-design.md`

- [ ] **Step 1: Run the complete workspace verification set**

Run:

```bash
cargo build -p workspace
cargo test -p workspace
./script/clippy -p workspace
cargo fmt --all -- --check
git diff --check origin/main...HEAD
```

Expected: build, all workspace tests, repository-prescribed Clippy, formatting, and whitespace checks exit successfully.

- [ ] **Step 2: Launch the local build using runtime shaders**

The installed Xcode 26.6 cannot currently download Metal Toolchain 17F109, so use the repository's runtime-shader feature that already launched successfully in this worktree:

```bash
cargo run --features gpui_platform/runtime_shaders -- /Users/bytedance/Documents/life_crm
```

Expected: the development build launches and keeps running without requiring the optional command-line Metal compiler.

- [ ] **Step 3: Verify the visual acceptance points**

Confirm in the running app:

```text
[ ] Activity Bar is 44 pixels wide and remains fixed at the far left.
[ ] Vertical icons are visibly larger while the rail remains compact.
[ ] Each vertical button has an approximately 34-by-32-pixel target.
[ ] The active button has a clearly visible neutral background and default icon color.
[ ] There is no blue selected icon, blue stripe, or colored selection marker.
[ ] The Activity Bar surface is subtly distinct from adjacent panel content in the dark theme.
[ ] The status bar does not regain duplicate left-dock buttons.
[ ] Git badges, tooltips, context menus, Up/Down focus, and dock toggling remain usable.
```

Capture a screenshot for user review and keep the local process running unless the user asks to stop it.

- [ ] **Step 4: Inspect final scope and history**

Run:

```bash
git status --short --branch
git diff origin/main...HEAD --stat
git log --oneline origin/main..HEAD
```

Expected: functional source changes remain limited to `crates/workspace/src/dock.rs` and `crates/workspace/src/workspace.rs`; `.superpowers/` is not staged or committed.

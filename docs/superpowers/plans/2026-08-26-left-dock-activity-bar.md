# Left Dock Activity Bar Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move left-dock panel buttons from the lower-left status bar into a persistent 40-pixel vertical Activity Bar, while retaining existing dock behavior and a neutral selected state with no blue marker.

**Architecture:** Reuse the existing `PanelButtons` entity with a small horizontal/vertical orientation enum. `Workspace` owns the vertical left-dock instance and mounts it once around the existing bottom-dock layout match; right- and bottom-dock instances remain horizontal status-bar items. `Dock` remains the sole state and persistence owner.

**Tech Stack:** Rust, GPUI, Zed `workspace` crate, existing `ui::IconButton`/`RightClickMenu` components, GPUI tests.

---

## File structure

- Modify `crates/workspace/src/dock.rs` to add panel-button orientation, vertical layout, inward-opening context-menu anchors, and focused unit tests.
- Modify `crates/workspace/src/workspace.rs` to retain and render the left-dock button entity, remove its status-bar registration, and add a workspace construction test.
- Keep `crates/workspace/src/status_bar.rs` and all individual panel crates unchanged.
- Use `docs/superpowers/specs/2026-08-26-left-dock-activity-bar-design.md` as the acceptance specification.

## Task 0: Verify the implementation environment and repository guard

**Files:**

- Verify: `README.md`
- Verify: `rust-toolchain.toml`

- [ ] **Step 1: Confirm the required review notice is still present**

Run:

```bash
sed -n '1,2p' README.md
```

Expected output:

```text
> [!IMPORTANT]
> Remove this line to confirm you've reviewed this PR before submitting.
```

Do not remove or rewrite these lines.

- [ ] **Step 2: Confirm the pinned Rust toolchain and native build prerequisite are available**

Run:

```bash
rustc --version
cargo --version
cmake --version
```

Expected: all three commands succeed; Rust resolves the version pinned by `rust-toolchain.toml` (currently `1.97.1`). If a command is missing, install the repository prerequisites before editing source files, then rerun these commands.

- [ ] **Step 3: Confirm the implementation starts from a clean worktree**

Run:

```bash
git status --short
```

Expected: a clean worktree. If unrelated user changes are present, preserve them and keep them out of this feature's commits.

## Task 1: Add an oriented `PanelButtons` component

**Files:**

- Modify: `crates/workspace/src/dock.rs:390-394`
- Modify: `crates/workspace/src/dock.rs:1379-1590`
- Test: `crates/workspace/src/dock.rs` immediately before the existing `test-support` module

- [ ] **Step 1: Add a failing unit test for orientation-specific context-menu placement**

Add this test module before `#[cfg(any(test, feature = "test-support"))]`:

```rust
#[cfg(test)]
mod panel_buttons_tests {
    use super::*;

    #[test]
    fn panel_button_orientation_selects_context_menu_anchors() {
        assert_eq!(
            PanelButtonsOrientation::Horizontal.menu_anchors(DockPosition::Left),
            (Anchor::BottomLeft, Anchor::TopLeft)
        );
        assert_eq!(
            PanelButtonsOrientation::Horizontal.menu_anchors(DockPosition::Bottom),
            (Anchor::BottomRight, Anchor::TopRight)
        );
        assert_eq!(
            PanelButtonsOrientation::Horizontal.menu_anchors(DockPosition::Right),
            (Anchor::BottomRight, Anchor::TopRight)
        );
        assert_eq!(
            PanelButtonsOrientation::Vertical.menu_anchors(DockPosition::Left),
            (Anchor::TopLeft, Anchor::TopRight)
        );
    }
}
```

- [ ] **Step 2: Run the focused test and verify it fails for the missing type**

Run:

```bash
cargo test -p workspace panel_button_orientation_selects_context_menu_anchors -- --nocapture
```

Expected: compilation fails because `PanelButtonsOrientation` does not exist yet. A failure caused by environment setup must be fixed before continuing.

- [ ] **Step 3: Add orientation state and constructors**

Replace the current `PanelButtons` declaration with:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PanelButtonsOrientation {
    Horizontal,
    Vertical,
}

impl PanelButtonsOrientation {
    fn menu_anchors(self, dock_position: DockPosition) -> (Anchor, Anchor) {
        match self {
            Self::Vertical => (Anchor::TopLeft, Anchor::TopRight),
            Self::Horizontal => match dock_position {
                DockPosition::Left => (Anchor::BottomLeft, Anchor::TopLeft),
                DockPosition::Bottom | DockPosition::Right => {
                    (Anchor::BottomRight, Anchor::TopRight)
                }
            },
        }
    }
}

pub struct PanelButtons {
    dock: Entity<Dock>,
    orientation: PanelButtonsOrientation,
    _settings_subscription: Subscription,
}
```

Replace the current constructor with:

```rust
impl PanelButtons {
    pub fn new(dock: Entity<Dock>, cx: &mut Context<Self>) -> Self {
        Self::with_orientation(dock, PanelButtonsOrientation::Horizontal, cx)
    }

    pub fn vertical(dock: Entity<Dock>, cx: &mut Context<Self>) -> Self {
        Self::with_orientation(dock, PanelButtonsOrientation::Vertical, cx)
    }

    fn with_orientation(
        dock: Entity<Dock>,
        orientation: PanelButtonsOrientation,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.observe(&dock, |_, _, cx| cx.notify()).detach();
        let settings_subscription = cx.observe_global::<SettingsStore>(|_, cx| cx.notify());
        Self {
            dock,
            orientation,
            _settings_subscription: settings_subscription,
        }
    }

    #[cfg(test)]
    pub(crate) fn orientation(&self) -> PanelButtonsOrientation {
        self.orientation
    }
}
```

- [ ] **Step 4: Use the orientation for menu anchors and container layout**

In `Render for PanelButtons`, replace the dock-position anchor match with:

```rust
let (menu_anchor, menu_attach) = self.orientation.menu_anchors(dock.position);
```

After the existing `buttons.reverse()` and `has_buttons` calculation, replace the final horizontal container with:

```rust
let button_container = match self.orientation {
    PanelButtonsOrientation::Horizontal => h_flex(),
    PanelButtonsOrientation::Vertical => v_flex()
        .size_full()
        .items_center()
        .py_1()
        .overflow_y_scroll()
        .scrollbar_width(px(0.)),
};

button_container
    .gap_1()
    .when(
        self.orientation == PanelButtonsOrientation::Horizontal
            && has_buttons
            && (dock.position == DockPosition::Bottom
                || dock.position == DockPosition::Right),
        |this| this.child(Divider::vertical().color(DividerColor::Border)),
    )
    .children(buttons)
    .when(
        self.orientation == PanelButtonsOrientation::Horizontal
            && has_buttons
            && dock.position == DockPosition::Left,
        |this| this.child(Divider::vertical().color(DividerColor::Border)),
    )
```

Do not add a custom active-state stripe or color. Retain the existing `IconButton::toggle_state(is_active_button)` so selection uses the standard neutral theme state.

- [ ] **Step 5: Run the focused test and formatting check**

Run:

```bash
cargo test -p workspace panel_button_orientation_selects_context_menu_anchors -- --nocapture
cargo fmt --all -- --check
```

Expected: the focused test passes and formatting reports no differences. If formatting fails, run `cargo fmt --all`, inspect the diff, and rerun the check.

- [ ] **Step 6: Commit the oriented component**

```bash
git add crates/workspace/src/dock.rs
git commit -m "Add vertical panel button layout"
```

## Task 2: Mount the left-dock buttons in the workspace Activity Bar

**Files:**

- Modify: `crates/workspace/src/workspace.rs:52`
- Modify: `crates/workspace/src/workspace.rs:1373-1420`
- Modify: `crates/workspace/src/workspace.rs:1760-1780`
- Modify: `crates/workspace/src/workspace.rs:1850-1905`
- Modify: `crates/workspace/src/workspace.rs:9287-9505`
- Test: `crates/workspace/src/workspace.rs` in the existing `tests` module

- [ ] **Step 1: Add a failing workspace ownership test**

Replace this existing import line:

```rust
dock::{PanelEvent, test::TestPanel},
```

with:

```rust
dock::{PanelButtonsOrientation, PanelEvent, test::TestPanel},
```

Add this test near `test_status_bar_visibility`:

```rust
#[gpui::test]
async fn test_left_dock_uses_vertical_panel_buttons(cx: &mut TestAppContext) {
    init_test(cx);

    let fs = FakeFs::new(cx.executor());
    let project = Project::test(fs, [], cx).await;
    let (workspace, _cx) =
        cx.add_window_view(|window, cx| Workspace::test_new(project, window, cx));

    workspace.read_with(cx, |workspace, cx| {
        assert_eq!(
            workspace.left_dock_buttons.read(cx).orientation(),
            PanelButtonsOrientation::Vertical
        );
    });
}
```

- [ ] **Step 2: Run the focused test and verify it fails for missing workspace ownership**

Run:

```bash
cargo test -p workspace test_left_dock_uses_vertical_panel_buttons -- --nocapture
```

Expected: compilation fails because `Workspace` does not yet retain `left_dock_buttons`.

- [ ] **Step 3: Retain the vertical entity and remove its status-bar registration**

Add this field immediately after the three dock fields in `Workspace`:

```rust
left_dock_buttons: Entity<PanelButtons>,
```

Change the three button constructors to:

```rust
let left_dock_buttons = cx.new(|cx| PanelButtons::vertical(left_dock.clone(), cx));
let bottom_dock_buttons = cx.new(|cx| PanelButtons::new(bottom_dock.clone(), cx));
let right_dock_buttons = cx.new(|cx| PanelButtons::new(right_dock.clone(), cx));
```

Remove only this line from the `StatusBar::new` closure:

```rust
status_bar.add_left_item(left_dock_buttons, window, cx);
```

Keep both right-side registrations unchanged. Add `left_dock_buttons` to the `Workspace` struct literal directly after `left_dock`:

```rust
left_dock,
left_dock_buttons,
bottom_dock,
right_dock,
```

- [ ] **Step 4: Wrap the existing dock-layout match once**

At the current dock-layout site, replace this exact opening:

```rust
                            .child({
                                match bottom_dock_layout {
```

with:

```rust
.child(
    h_flex()
        .flex_1()
        .min_h_0()
        .w_full()
        .overflow_hidden()
        .child(
            div()
                .id("left-dock-activity-bar")
                .w(px(40.))
                .h_full()
                .flex_none()
                .border_r_1()
                .border_color(colors.border)
                .bg(colors.panel_background)
                .child(self.left_dock_buttons.clone()),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .h_full()
                .child({
                    match bottom_dock_layout {
```

Leave all four match arms unchanged. At the end of the `Contained` arm, replace this exact closing:

```rust
                                }
                            })
                            .children(self.zoomed.as_ref().and_then(|view| {
```

with:

```rust
                                }
                            }),
                        ),
                ),
            )
            .children(self.zoomed.as_ref().and_then(|view| {
```

Do not insert the Activity Bar separately inside any match arm. The wrapper must remain below the title bar, above the status bar, and outside the zoom overlay so the overlay behavior remains unchanged.

- [ ] **Step 5: Run the ownership test and existing dock behavior tests**

Run:

```bash
cargo test -p workspace test_left_dock_uses_vertical_panel_buttons -- --nocapture
cargo test -p workspace test_toggle_docks_and_panels -- --nocapture
cargo test -p workspace test_move_focused_panel_to_next_position -- --nocapture
cargo test -p workspace test_panels_stay_open_after_position_change_and_settings_update -- --nocapture
```

Expected: all tests pass. The existing behavior tests establish that the Activity Bar continues to use the same `Dock` state and actions rather than duplicating selection or persistence.

- [ ] **Step 6: Format, inspect the scoped diff, and commit**

Run:

```bash
cargo fmt --all
git diff --check
git diff -- crates/workspace/src/dock.rs crates/workspace/src/workspace.rs
```

Expected: no whitespace errors; source changes remain limited to the oriented component, workspace ownership/registration, the single outer layout wrapper, and tests.

Commit:

```bash
git add crates/workspace/src/workspace.rs
git commit -m "Move left dock buttons into an activity bar"
```

## Task 3: Complete automated and visual regression verification

**Files:**

- Verify: `crates/workspace/src/dock.rs`
- Verify: `crates/workspace/src/workspace.rs`
- Verify: `docs/superpowers/specs/2026-08-26-left-dock-activity-bar-design.md`

- [ ] **Step 1: Run all workspace tests**

Run:

```bash
cargo test -p workspace
```

Expected: the `workspace` test suite passes without ignored new failures.

- [ ] **Step 2: Run the repository-prescribed linter**

Run:

```bash
./script/clippy -p workspace
```

Expected: Clippy completes successfully. Use this script instead of invoking `cargo clippy` directly.

- [ ] **Step 3: Launch the customized build and verify the acceptance matrix**

Launch the normal local Zed development build for this checkout against a non-Zed project:

```bash
cargo run -- /Users/bytedance/Documents/life_crm
```

Expected: the development build launches successfully. Then verify each item manually:

```text
[ ] Activity Bar is fixed at the far left and spans from below the title bar to above the status bar.
[ ] Project, Search, Git, Outline, Agent, and any other left-dock buttons appear vertically in dock order.
[ ] Left-dock buttons no longer appear in the status bar; right- and bottom-dock buttons still do.
[ ] Inactive click opens/selects; clicking the active button closes the left dock.
[ ] Selected state is a neutral background only, with no blue stripe or colored marker.
[ ] Git/count badges, tooltips, keyboard focus, ARIA labels, and right-click menus still work.
[ ] Context menus open to the right, into the workspace.
[ ] Moving a panel among Left, Bottom, and Right moves its button to the matching container.
[ ] Hiding a panel button still removes it.
[ ] The rail remains visible while the left dock is closed.
[ ] Overflow scrolls vertically without a persistent visible scrollbar.
[ ] Full, LeftAligned, RightAligned, and Contained bottom-dock layouts all remain correct.
[ ] Light and dark themes and a narrow window remain usable.
```

- [ ] **Step 4: Inspect final scope and history**

Run:

```bash
git status --short
git diff origin/main...HEAD --stat
git log --oneline origin/main..HEAD
```

Expected: no uncommitted source changes; customization remains concentrated in the two workspace source files plus the design/plan documentation and mandatory README notice.

## Task 4: Prepare the upstream-compatible handoff

- [ ] **Step 1: Record the rebase hot spots**

In the eventual PR or fork-maintenance notes, call out only these likely conflict locations:

```text
- PanelButtons construction and rendering in crates/workspace/src/dock.rs
- Panel-button registration in Workspace::new
- The outer wrapper around the bottom_dock_layout match in Workspace::render
```

- [ ] **Step 2: Use a compliant PR description if publishing the branch**

Use an imperative, correctly capitalized PR title without a conventional-commit prefix or trailing punctuation. End the body exactly with a release note section such as:

```text
Release Notes:

- Improved left-dock navigation by adding a persistent vertical activity bar.
```

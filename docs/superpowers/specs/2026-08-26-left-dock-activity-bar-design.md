# Left Dock Activity Bar Design

## Context

Zed currently renders panel buttons for all three docks in the status bar. The buttons for panels assigned to the left dock therefore appear in the lower-left corner of the window. This makes frequently used panels such as Project, Search, Git, Outline, and Agent harder to reach than a persistent vertical activity bar.

The customized build should present left-dock panel buttons in a narrow vertical rail at the far left of the workspace, similar to a conventional editor activity bar. The change must reuse Zed's existing dock state and panel registration so it remains inexpensive to carry while syncing upstream changes.

## Goals

- Render every visible left-dock panel button in a persistent vertical activity bar.
- Remove the duplicate left-dock buttons from the status bar.
- Preserve existing panel activation, closing, badges, tooltips, context menus, keyboard navigation, and accessibility labels.
- Automatically reflect panels being added to or moved out of the left dock.
- Avoid changing individual panel implementations.
- Keep the workspace layout change isolated from the three bottom-dock layout variants.

## Non-goals

- Moving right-dock or bottom-dock buttons out of the status bar.
- Introducing a general-purpose user setting for activity-bar placement in the first version.
- Reordering panel buttons independently from the dock's existing panel order.
- Redesigning panel headers, panel contents, icons, or panel persistence.
- Adding a separate activity-bar plugin or extension API.

## Existing behavior

`Workspace::new` constructs one `PanelButtons` entity for each dock. It registers the left-dock entity as a left status item and the right- and bottom-dock entities as right status items. `PanelButtons` reads its `Dock` directly and builds buttons from the dock's `panel_entries`.

The existing button implementation already provides the required behavior:

- Active-state calculation from the dock's active panel and open state.
- Panel toggle actions and focus management.
- Per-panel icons, tooltips, count badges, and visibility settings.
- A context menu for moving a panel, changing flexible width, and hiding its button.

The activity bar changes only the presentation and mount point of this existing component.

## Considered approaches

### Extend `PanelButtons` with orientation

Add horizontal and vertical presentation modes to the existing component. Mount the left-dock instance in the workspace content and retain the horizontal instances for the right and bottom docks.

This is the selected approach because it keeps the existing behavior in one implementation, automatically supports new panels, and produces the smallest upstream maintenance surface.

### Add a separate `ActivityBar` component

Create a new component that separately iterates over left-dock panel entries. This gives the rail a distinct name but duplicates button construction, context menus, active-state logic, badges, and accessibility behavior. The two implementations could drift as upstream changes panel buttons.

### Modify individual panels

Have Project, Git, Search, and other panels register separate activity-bar buttons. This spreads the change across many crates, omits future panels by default, and creates the greatest merge burden. It is rejected.

## Architecture

### Oriented panel-button component

`PanelButtons` will accept an orientation selected at construction. Horizontal remains the default presentation for status-bar instances. Vertical presentation will:

- Use a vertical flex container.
- Stack buttons from top to bottom in existing dock order.
- Attach context menus to the right side of each button.
- Use a right-side divider for separation from the adjacent panel.
- Permit vertical overflow without changing the editor or dock size.

Button creation remains shared between orientations. The orientation changes only container layout, divider direction, spacing, and menu anchoring.

### Workspace ownership

`Workspace` will retain the vertical left-dock `PanelButtons` entity so it can render it in the main element tree. The left-dock instance will no longer be passed to `StatusBar::add_left_item`. The existing right- and bottom-dock instances will continue to be registered with the status bar without behavior changes.

### Layout insertion

The workspace content area will be wrapped in one horizontal container:

1. The activity bar is the fixed-width first child.
2. The existing workspace content is the flexible second child.

This wrapper sits below the title bar and above the status bar. It encloses the existing `bottom_dock_layout` match instead of being inserted separately into the `Full`, `LeftAligned`, and `RightAligned` branches. Consequently, the activity bar spans the main content and bottom-dock area consistently while the three existing layout implementations remain unchanged.

## Visual design

- Fixed width of 40 pixels, represented with the corresponding existing UI spacing token.
- Panel-compatible background color with a right border using the theme's existing border color.
- Existing small panel icons and standard `IconButton` interaction states.
- Top-aligned panel buttons with compact vertical spacing.
- Existing count badges, including Git change counts.
- Neutral selected background only. There is no blue accent bar or other colored selection marker.
- The rail remains visible when the left dock is closed.
- If the number of buttons exceeds the available height, the rail scrolls vertically without a persistent scrollbar consuming width.

The design intentionally uses current theme colors and component states instead of introducing activity-bar-specific colors.

## Interaction and state flow

The existing `Dock` remains the sole source of panel state. The activity bar does not persist a new selected panel or open state.

When a button is clicked:

- If its panel is not active, the existing panel toggle action opens the left dock and activates that panel.
- If its panel is already active and the left dock is open, the existing dock toggle action closes the left dock.
- Focus continues to move through the dock's focus handle before the action is dispatched.

When a panel is moved to the right or bottom dock through the existing context menu, the dock entry changes and the button automatically disappears from the left activity bar. It then appears in the corresponding status-bar button group. Moving a panel into the left dock performs the inverse transition.

Per-panel button visibility settings continue to filter the shared button list. No migration or additional persisted state is required.

## Accessibility

- Retain each icon button's existing tab index and ARIA label.
- Retain action-derived tooltips and keyboard shortcuts.
- Keep the activity bar in a predictable position before left-dock content in the workspace element tree.
- Ensure the vertical container does not trap keyboard focus when it scrolls.
- Preserve count badges as supplemental visual information rather than the only accessible label.

## Error handling

The feature introduces no fallible I/O or asynchronous operations. A panel without an icon or tooltip continues to follow the existing logged-error behavior in `PanelButtons`. Orientation must not add a second error path or silently substitute panel metadata.

## Testing

Implementation verification will cover:

- Left-dock buttons render in vertical dock order.
- Left-dock buttons are absent from the status bar.
- Right- and bottom-dock status-bar buttons remain unchanged.
- Clicking an inactive button opens and activates its panel.
- Clicking the active button closes the left dock.
- Moving panels among left, right, and bottom docks moves their buttons to the correct container.
- Hidden panel buttons remain hidden.
- Count badges remain visible in the vertical layout.
- Context menus open toward the workspace instead of outside the window.
- Keyboard focus and action dispatch continue to work.
- `Full`, `LeftAligned`, and `RightAligned` bottom-dock layouts all render correctly.
- Light and dark themes, narrow windows, closed left docks, and overflowing button lists remain usable.

Tests reuse existing GPUI dock test support. A focused component or workspace test covers the state transitions, followed by visual verification for spacing, theme behavior, and the three bottom-dock layouts.

## Upstream compatibility

The customization is intentionally concentrated in `crates/workspace/src/dock.rs` and `crates/workspace/src/workspace.rs`. It does not modify Project, Git, Search, Outline, Agent, or other panel crates. It also avoids a new settings schema and leaves existing bottom-dock layout branches intact.

When pulling upstream changes, the main review points are:

- Changes to `PanelButtons` rendering or panel-button metadata.
- Changes to status-bar registration during `Workspace` construction.
- Changes to the outer workspace render tree around the bottom-dock layout match.

New upstream panels that implement the existing `Panel` interface require no customization-specific work.

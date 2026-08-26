# Project Search Activity Bar Design

## Goal

Move Project Search from the status bar into the left Activity Bar and pin the Collab Panel button to the bottom of that bar.

With the current left-dock panels, the top section uses this order:

1. Project Panel
2. Git Panel
3. Project Search
4. Remaining ordinary panel buttons in their existing priority order

The Collab Panel button is rendered in a separate bottom section.

## Scope

- Change only the vertical button layout used by the left dock Activity Bar.
- Keep horizontal panel buttons for the right and bottom docks unchanged.
- Remove the Project Search button from the status bar so the action is not duplicated.
- Preserve the existing Project Search action, shortcut tooltip, accessibility label, and `editor.search.button` visibility setting.
- Preserve existing panel toggle actions, badges, context menus, and hide-button settings.
- Keep the existing 44px rail, 34x32px vertical buttons, 18px icons, neutral selected state, and theme colors.

## Architecture

### Activity Bar placement metadata

Add a workspace-owned placement enum for panel buttons with `Top` and `Bottom` variants. `Panel` defaults to `Top`, and `PanelHandle` forwards the placement metadata. `CollabPanel` overrides the default with `Bottom`.

This keeps the workspace implementation independent of `collab_ui` and avoids matching panel names or icons.

### External Activity Bar items

Allow the vertical `PanelButtons` instance to receive type-erased renderable items with an explicit numeric priority. Panel buttons and external items in the top section are merged by priority while preserving stable ordering for equal priorities.

Project Search is registered by the application layer with a priority immediately after Git Panel and before the remaining ordinary panels. This preserves crate boundaries: the `workspace` crate does not depend on the `search` crate.

### Project Search presentation

`SearchButton` gains an Activity Bar presentation in addition to its existing status-bar presentation. The Activity Bar presentation uses the established vertical metrics: 18px icon, 32px button height, and 34px width. It continues to dispatch `workspace::DeploySearch` and obey `EditorSettings::search.button`.

The Activity Bar presentation receives a weak workspace handle and resolves the current active item when its tooltip is shown. This preserves context-sensitive shortcut display without keeping the button registered in the status bar.

The application initialization registers this entity with the left Activity Bar instead of adding it to the status bar.

## Layout and interaction

The vertical container is divided into a top group and a bottom group with flexible space between them. Ordinary panel buttons and Project Search appear in the top group. Bottom-placed panel buttons, currently Collab Panel, appear in the bottom group.

All controls remain descendants of the same ARIA toolbar and tab group. Existing Up and Down handling therefore continues to move focus across the top group, Project Search, and the bottom group without leaving the Activity Bar.

If Project Search is hidden through `editor.search.button`, it is omitted without changing panel ordering. If Collab Panel is hidden or placed on the right dock, no empty bottom control is rendered.

## Testing

- Add a failing workspace test for top-item priority insertion and top/bottom partitioning before implementing the ordering logic.
- Add a failing search test that locks Activity Bar button metrics while preserving status-bar metrics.
- Add or update a workspace integration test to verify the external item is mounted in the Activity Bar and is absent from status-bar registration.
- Verify existing horizontal orientation, selected styling, Activity Bar bounds, toolbar focus behavior, and flexible dock sizing tests continue to pass.
- Run the full workspace and affected crate test suites, `./script/clippy`, formatting, and diff checks.

## Non-goals

- Reordering panels inside the docks themselves.
- Changing Project Search behavior or search results.
- Adding user-configurable Activity Bar ordering in this iteration.
- Moving right- or bottom-dock controls into a vertical rail.

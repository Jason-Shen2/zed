use editor::EditorSettings;
use gpui::{App, FocusHandle, Pixels, WeakEntity, px};
use settings::Settings as _;
use ui::{ButtonCommon, Clickable, Context, Render, Tooltip, Window, prelude::*};
use workspace::{HideStatusItem, ItemHandle, StatusItemView};

pub const SEARCH_ICON: IconName = IconName::MagnifyingGlass;

#[derive(Clone, Copy)]
enum SearchButtonPresentation {
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
    presentation: SearchButtonPresentation,
    workspace: Option<WeakEntity<workspace::Workspace>>,
    pane_item_focus_handle: Option<FocusHandle>,
}

impl SearchButton {
    pub fn new() -> Self {
        Self {
            presentation: SearchButtonPresentation::StatusBar,
            workspace: None,
            pane_item_focus_handle: None,
        }
    }

    pub fn activity_bar(workspace: WeakEntity<workspace::Workspace>) -> Self {
        Self {
            presentation: SearchButtonPresentation::ActivityBar,
            workspace: Some(workspace),
            pane_item_focus_handle: None,
        }
    }
}

impl Render for SearchButton {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl ui::IntoElement {
        let button = div().debug_selector(|| "project-search-indicator".to_owned());

        if !EditorSettings::get_global(cx).search.button {
            return button.hidden();
        }

        let visuals = self.presentation.visuals();
        let workspace = self.workspace.clone();
        let pane_item_focus_handle = self.pane_item_focus_handle.clone();
        button.child(
            IconButton::new("project-search-indicator", SEARCH_ICON)
                .icon_size(visuals.icon_size)
                .when_some(visuals.button_size, |this, size| this.size(size))
                .when_some(visuals.width, |this, width| this.width(width))
                .tab_index(0isize)
                .aria_label("Project Search")
                .tooltip(move |_window, cx| {
                    let focus_handle = workspace
                        .as_ref()
                        .and_then(WeakEntity::upgrade)
                        .and_then(|workspace| {
                            workspace
                                .read(cx)
                                .active_item(cx)
                                .map(|item| item.item_focus_handle(cx))
                        })
                        .or_else(|| pane_item_focus_handle.clone());
                    if let Some(focus_handle) = &focus_handle {
                        Tooltip::for_action_in(
                            "Project Search",
                            &workspace::DeploySearch::default(),
                            focus_handle,
                            cx,
                        )
                    } else {
                        Tooltip::for_action(
                            "Project Search",
                            &workspace::DeploySearch::default(),
                            cx,
                        )
                    }
                })
                .on_click(cx.listener(|_this, _, window, cx| {
                    window.dispatch_action(Box::new(workspace::DeploySearch::default()), cx);
                })),
        )
    }
}

impl StatusItemView for SearchButton {
    fn set_active_pane_item(
        &mut self,
        active_pane_item: Option<&dyn ItemHandle>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.pane_item_focus_handle = active_pane_item.map(|item| item.item_focus_handle(cx));
    }

    fn hide_setting(&self, _: &App) -> Option<HideStatusItem> {
        Some(HideStatusItem::new(|settings| {
            settings.editor.search.get_or_insert_default().button = Some(false);
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::px;

    #[test]
    fn search_button_status_bar_presentation_uses_status_bar_metrics() {
        let visuals = SearchButtonPresentation::StatusBar.visuals();

        assert!(visuals.icon_size == IconSize::Small);
        assert!(visuals.button_size.is_none());
        assert_eq!(visuals.width, None);
    }

    #[test]
    fn search_button_activity_bar_presentation_uses_activity_bar_metrics() {
        let visuals = SearchButtonPresentation::ActivityBar.visuals();

        assert!(visuals.icon_size == IconSize::Custom(ui::rems_from_px(18_f32)));
        assert!(visuals.button_size == Some(ButtonSize::Large));
        assert_eq!(visuals.width, Some(px(34.)));
    }
}

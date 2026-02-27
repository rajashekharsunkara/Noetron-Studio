//! noetron_toggle — top toggle bar: switches between No-Code and Full-Code modes.
//!
//! This is a `ToolbarItemView` that sits at the top of every Noetron pane.
//! When No-Code is active the editor is hidden and the `noetron_ui` panels are shown.
//! When Full-Code is active the normal Zed editor is shown.

use gpui::{App, Context, EventEmitter, Global, IntoElement, Render, Window, actions};
use ui::prelude::*;
use workspace::{ToolbarItemEvent, ToolbarItemLocation, ToolbarItemView};

actions!(noetron_toggle, [ToggleNoCode]);

// ── Global mode ───────────────────────────────────────────────────────────────

/// The current edit mode for the focused pane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NoetronMode {
    #[default]
    FullCode,
    NoCode,
}

/// Global (app-wide) mode state.  Each workspace reads this.
pub struct NoetronModeState {
    pub mode: NoetronMode,
}

impl Global for NoetronModeState {}

impl NoetronModeState {
    pub fn init(cx: &mut App) {
        cx.set_global(NoetronModeState { mode: NoetronMode::FullCode });
    }

    pub fn toggle(cx: &mut App) {
        cx.update_global::<NoetronModeState, _>(|state, _cx| {
            state.mode = match state.mode {
                NoetronMode::FullCode => NoetronMode::NoCode,
                NoetronMode::NoCode => NoetronMode::FullCode,
            };
        });
    }

    pub fn current(cx: &App) -> NoetronMode {
        cx.try_global::<NoetronModeState>()
            .map(|s| s.mode)
            .unwrap_or_default()
    }
}

// ── Toggle bar ────────────────────────────────────────────────────────────────

/// A toolbar item that renders the mode toggle strip.
pub struct NoetronToggleBar {
    has_aiproj: bool,
}

impl NoetronToggleBar {
    pub fn new() -> Self {
        Self { has_aiproj: false }
    }
}

impl Default for NoetronToggleBar {
    fn default() -> Self {
        Self::new()
    }
}

impl EventEmitter<ToolbarItemEvent> for NoetronToggleBar {}

impl Render for NoetronToggleBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.has_aiproj {
            return div().into_any_element();
        }

        let mode = NoetronModeState::current(cx);
        let is_nocode = mode == NoetronMode::NoCode;

        h_flex()
            .id("noetron-toggle-bar")
            .h_8()
            .px_2()
            .gap_1()
            .items_center()
            .child(
                // No-Code button
                Button::new("nocode-btn", "⬡ No-Code")
                    .style(if is_nocode {
                        ButtonStyle::Filled
                    } else {
                        ButtonStyle::Subtle
                    })
                    .size(ButtonSize::Compact)
                    .on_click(cx.listener(|_this, _ev, _window, cx| {
                        if NoetronModeState::current(cx) != NoetronMode::NoCode {
                            NoetronModeState::toggle(cx);
                            cx.notify();
                        }
                    })),
            )
            .child(
                // Full-Code button
                Button::new("fullcode-btn", "</> Full Code")
                    .style(if !is_nocode {
                        ButtonStyle::Filled
                    } else {
                        ButtonStyle::Subtle
                    })
                    .size(ButtonSize::Compact)
                    .on_click(cx.listener(|_this, _ev, _window, cx| {
                        if NoetronModeState::current(cx) != NoetronMode::FullCode {
                            NoetronModeState::toggle(cx);
                            cx.notify();
                        }
                    })),
            )
            .into_any_element()
    }
}

impl ToolbarItemView for NoetronToggleBar {
    fn set_active_pane_item(
        &mut self,
        active_pane_item: Option<&dyn workspace::ItemHandle>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> ToolbarItemLocation {
        // Show the bar only when the active item comes from a .aiproj project.
        // For now we show it whenever there is any active item.
        self.has_aiproj = active_pane_item.is_some();
        cx.notify();
        if self.has_aiproj {
            ToolbarItemLocation::PrimaryLeft
        } else {
            ToolbarItemLocation::Hidden
        }
    }
}

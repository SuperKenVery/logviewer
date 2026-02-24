use gpui::*;
use gpui::AsyncApp;
use std::borrow::Cow;
use anyhow::Result;
use std::path::PathBuf;
use crate::source::{start_source, LogSource, SourceEvent};
use super::model::AppState;
use super::main_view::MainView;
use super::main_view::{FocusFilter, FocusHide, FocusHighlight, FocusLineStart, ToggleListen, ToggleTime, ToggleWrap, ToggleFollowTail, Clear, FocusList};
use fancy_regex::Regex;
use std::sync::Arc;

struct Assets;

impl AssetSource for Assets {
    fn load(&self, _path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Ok(None)
    }

    fn list(&self, _path: &str) -> Result<Vec<SharedString>> {
        Ok(vec![])
    }
}

pub fn run(file: Option<PathBuf>, port: Option<u16>) {
    let platform = gpui_platform::current_platform(false);
    Application::with_platform(platform)
        .with_assets(Assets)
        .run(move |cx| {
            cx.bind_keys([
                KeyBinding::new("cmd-f", FocusFilter, None),
                KeyBinding::new("cmd-d", FocusHide, None),
                KeyBinding::new("cmd-h", FocusHighlight, None),
                KeyBinding::new("cmd-s", FocusLineStart, None),
                KeyBinding::new("cmd-l", ToggleListen, None),
                KeyBinding::new("cmd-t", ToggleTime, None),
                KeyBinding::new("cmd-w", ToggleWrap, None),
                KeyBinding::new("cmd-g", ToggleFollowTail, None),
                KeyBinding::new("cmd-k", Clear, None),
                KeyBinding::new("escape", FocusList, None),
            ]);

            let (tx, rx) = async_channel::unbounded::<SourceEvent>();
            let (sync_tx, sync_rx) = std::sync::mpsc::channel::<SourceEvent>();

            let app_state = AppState::new(cx, Some(sync_tx.clone()));

            let source = if let Some(p) = port {
                LogSource::Network(p)
            } else if let Some(f) = file.clone() {
                LogSource::File(f)
            } else {
                LogSource::Stdin
            };

            std::thread::spawn(move || {
                while let Ok(event) = sync_rx.recv() {
                    if tx.send_blocking(event).is_err() {
                        break;
                    }
                }
            });

            let regex = {
                let state = app_state.read(cx);
                if state.line_start_text.trim().is_empty() {
                    None
                } else {
                    Regex::new(&state.line_start_text).ok().map(Arc::new)
                }
            };

            if let Err(e) = start_source(source, sync_tx, regex) {
                app_state.update(cx, |state, _cx| {
                    state.connection_status = format!("Failed to start: {}", e);
                });
            }

            let state_clone = app_state.clone();
            cx.spawn(|cx: &mut AsyncApp| {
                let cx = cx.clone();
                async move {
                    while let Ok(event) = rx.recv().await {
                        let _ = cx.update(|cx| {
                            state_clone.update(cx, |state: &mut AppState, cx| {
                                match event {
                                    SourceEvent::Line(line) => state.add_line(line, cx),
                                    SourceEvent::SystemLine(line) => state.add_line(line, cx),
                                    SourceEvent::Connected(peer) => state.connection_status = format!("Connected: {}", peer),
                                    SourceEvent::Disconnected(peer) => state.connection_status = format!("Disconnected: {}", peer),
                                    SourceEvent::Error(err) => state.connection_status = format!("Error: {}", err),
                                }
                            });
                        });
                    }
                }
            }).detach();

            cx.open_window(WindowOptions::default(), |_, cx| {
                cx.new(|cx| MainView::new(app_state.clone(), cx))
            }).ok();
        });
}

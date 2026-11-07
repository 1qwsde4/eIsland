/*
 * eIsland - A sleek, Apple Dynamic Island inspired floating widget for Windows, built with Electron.
 * https://github.com/JNTMTMTM/eIsland
 *
 * Copyright (C) 2026 JNTMTMTM
 * Copyright (C) 2026 pyisland.com
 *
 * Original author: JNTMTMTM[](https://github.com/JNTMTMTM)
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 */

//! 可被快捷键和命令边界复用的窗口业务动作。

use super as windows;
use crate::{shared::Result, state::Desktop};
use serde_json::{json, Value};
use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;

pub(super) fn store_write(app: &AppHandle, key: &str, value: &Value) -> Result<()> {
    app.state::<Desktop>()
        .store
        .lock()
        .unwrap()
        .write(key, value)
}

pub(crate) fn set_shape(app: &AppHandle, mode: &str) -> Result<()> {
    if !matches!(mode, "notch" | "pill") {
        return Err("Invalid shape mode".into());
    }
    store_write(app, "island-shape-mode", &json!(mode))?;
    app.state::<Desktop>().inner.lock().unwrap().shape = mode.into();
    windows::reposition(app)?;
    let rect = app.state::<Desktop>().inner.lock().unwrap().visible;
    windows::emit(
        app,
        "main",
        "island:shape-mode:changed",
        json!([mode, rect.x, rect.y]),
    )?;
    windows::broadcast(app, "island:shape-mode", json!(mode))
}

/// Blendet die Insel für 10 oder 15 Sekunden aus und zeigt sie danach automatisch wieder.
pub(crate) fn hide_temporarily(app: &AppHandle, seconds: u64) -> Result<()> {
    if !matches!(seconds, 10 | 15) {
        return Err("Invalid hide duration".into());
    }
    windows::hide_main(app)?;
    let generation = app
        .state::<Desktop>()
        .inner
        .lock()
        .unwrap()
        .visibility_generation;
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;
        // Nur wiederherstellen, wenn die Insel seitdem nicht manuell neu angezeigt/ausgeblendet wurde.
        let still_hidden = {
            let state = handle.state::<Desktop>();
            let s = state.inner.lock().unwrap();
            s.manually_hidden && s.visibility_generation == generation
        };
        if still_hidden {
            if let Err(error) = windows::show_main(&handle) {
                eprintln!("[Window] show after temporary hide: {error}");
            }
        }
    });
    Ok(())
}

/// Wechselt zwischen fixierter und frei verschiebbarer Position.
pub(crate) fn toggle_position_lock(app: &AppHandle) -> Result<()> {
    let locked = app
        .state::<Desktop>()
        .store
        .lock()
        .unwrap()
        .value("island-position-locked", json!(false))
        == true;
    let next = json!(!locked);
    store_write(app, "island-position-locked", &next)?;
    windows::broadcast(app, "store:island-position-locked", next)
}

pub(crate) fn open_logs(app: &AppHandle) -> Result<()> {
    let path = app.path().app_log_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    app.opener()
        .open_path(path.to_string_lossy(), None::<&str>)
        .map_err(|e| e.to_string())
}

pub(crate) fn menu_action(app: &AppHandle, action: &str) -> Result<()> {
    match action {
        "show-island" => windows::show_main(app),
        "hide-island" => windows::hide_main(app),
        "hide-island-10s" => hide_temporarily(app, 10),
        "hide-island-15s" => hide_temporarily(app, 15),
        "toggle-position-lock" => toggle_position_lock(app),
        "open-settings" => {
            store_write(app, "standalone-window-active-tab", &json!("settings"))?;
            windows::broadcast(app, "store:standalone-window-active-tab", json!("settings"))?;
            windows::show_standalone(app)
        }
        "toggle-always-on-top" => {
            let enabled = {
                let state = app.state::<Desktop>();
                let mut s = state.inner.lock().unwrap();
                s.always_on_top = !s.always_on_top;
                s.always_on_top
            };
            for label in ["main", "cli-glow", "agent-voice-input"] {
                if let Some(window) = app.get_webview_window(label) {
                    window
                        .set_always_on_top(enabled)
                        .map_err(|e| e.to_string())?;
                }
            }
            windows::raise_main(app)
        }
        "open-log-folder" => open_logs(app),
        "quit" => {
            app.exit(0);
            Ok(())
        }
        "restart" => app.restart(),
        _ => Err("Unknown tray action".into()),
    }
}

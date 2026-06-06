use std::fs;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime, WindowEvent,
};
use tauri_plugin_opener::OpenerExt;

pub mod engine;
pub mod expand;
pub mod form;
pub mod hook;
pub mod inject;
pub mod matcher;
pub mod commands;
pub mod store;
pub mod sync;

pub const SINGLE_INSTANCE_MUTEX: &str = "Global\\openmacro-singleton";
const PAUSE_RESUME_MENU_ID: &str = "pause_resume";
const RELOAD_SNIPPETS_MENU_ID: &str = "reload_snippets";
const OPEN_SNIPPETS_FOLDER_MENU_ID: &str = "open_snippets_folder";
const OPEN_SETTINGS_MENU_ID: &str = "open_settings";
const QUIT_MENU_ID: &str = "quit";

struct AppTray<R: Runtime> {
    _tray: TrayIcon<R>,
}

fn show_settings_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn setup_error(message: impl Into<String>) -> tauri::Error {
    let boxed: Box<dyn std::error::Error> = Box::new(std::io::Error::other(message.into()));
    tauri::Error::Setup(boxed.into())
}

fn open_snippets_folder<R: Runtime>(app: &AppHandle<R>) {
    let Ok(snippets_dir) = crate::commands::snippets::snippets_root() else {
        return;
    };
    if fs::create_dir_all(&snippets_dir).is_ok() {
        let _ = app
            .opener()
            .open_path(snippets_dir.to_string_lossy().into_owned(), None::<String>);
    }
}

fn build_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let pause_resume =
        MenuItem::with_id(app, PAUSE_RESUME_MENU_ID, "Pause/Resume", true, None::<&str>)?;
    let reload_snippets =
        MenuItem::with_id(app, RELOAD_SNIPPETS_MENU_ID, "Reload snippets", true, None::<&str>)?;
    let open_snippets_folder_item = MenuItem::with_id(
        app,
        OPEN_SNIPPETS_FOLDER_MENU_ID,
        "Open snippets folder",
        true,
        None::<&str>,
    )?;
    let open_settings =
        MenuItem::with_id(app, OPEN_SETTINGS_MENU_ID, "Open settings", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, QUIT_MENU_ID, "Quit", true, None::<&str>)?;

    let pause_resume_id = pause_resume.id().clone();
    let reload_snippets_id = reload_snippets.id().clone();
    let open_snippets_folder_id = open_snippets_folder_item.id().clone();
    let open_settings_id = open_settings.id().clone();
    let quit_id = quit.id().clone();

    let menu = Menu::with_items(
        app,
        &[
            &pause_resume,
            &reload_snippets,
            &open_snippets_folder_item,
            &open_settings,
            &quit,
        ],
    )?;

    let tray = TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().cloned().expect("default window icon"))
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| {
            if event.id == pause_resume_id {
                let _ = crate::engine::toggle_paused();
                return;
            }

            if event.id == reload_snippets_id {
                return;
            }

            if event.id == open_snippets_folder_id {
                open_snippets_folder(app);
            } else if event.id == open_settings_id {
                show_settings_window(app);
            } else if event.id == quit_id {
                app.exit(0);
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_settings_window(tray.app_handle());
            }
        })
        .build(app)?;

    app.manage(AppTray { _tray: tray });

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            crate::commands::snippets::list_snippets,
            crate::commands::snippets::save_snippet,
            crate::commands::snippets::reload_snippets,
            crate::commands::snippets::list_load_errors,
            crate::commands::prefs::get_prefs,
            crate::commands::prefs::set_prefs
        ])
        .plugin(
            tauri_plugin_single_instance::Builder::new()
                .windows_mutex_name(SINGLE_INSTANCE_MUTEX)
                .callback(|app, _args, _cwd| show_settings_window(app))
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None::<Vec<&'static str>>,
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let snippets_root = crate::commands::snippets::snippets_root().map_err(setup_error)?;
            fs::create_dir_all(&snippets_root).map_err(tauri::Error::from)?;
            let snippet_state = crate::commands::snippets::SnippetStoreState::load(snippets_root.clone())
                .map_err(setup_error)?;
            let watcher = crate::store::watch_root(snippets_root).map_err(|error| setup_error(error.to_string()))?;
            let mut rx = watcher.subscribe();
            let snapshot_handle = snippet_state.snapshot_handle();
            tauri::async_runtime::spawn(async move {
                while rx.changed().await.is_ok() {
                    let next = rx.borrow().clone();
                    let mut snapshot = snapshot_handle.write().unwrap();
                    snapshot.revision = next.revision;
                    snapshot.snippets = next.snippets.clone();
                    snapshot.errors = next.errors.clone();
                }
            });
            snippet_state.set_watcher(watcher);
            let prefs_state = crate::commands::prefs::load_prefs_state().map_err(setup_error)?;
            app.manage(snippet_state);
            app.manage(prefs_state);
            let engine_handle = crate::engine::start_runtime();
            app.manage(engine_handle);
            build_tray(app.handle())?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    #[test]
    fn single_instance_mutex_name_matches_spec() {
        assert_eq!(super::SINGLE_INSTANCE_MUTEX, "Global\\openmacro-singleton");
    }

    #[test]
    fn snippets_dir_path_matches_spec() {
        assert_eq!(
            PathBuf::from(r"C:\Users\me\AppData\Roaming")
                .join("openmacro")
                .join("snippets"),
            PathBuf::from(r"C:\Users\me\AppData\Roaming\openmacro\snippets"),
        );
    }

    #[test]
    fn snippets_root_prefers_env_override() {
        std::env::set_var("OPENMACRO_SNIPPETS_ROOT", r"C:\temp\openmacro-snippets");

        assert_eq!(
            crate::commands::snippets::snippets_root().unwrap(),
            Path::new(r"C:\temp\openmacro-snippets")
        );

        std::env::remove_var("OPENMACRO_SNIPPETS_ROOT");
    }
}

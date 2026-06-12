use std::{fs, sync::Arc};

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime, WindowEvent,
};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_opener::OpenerExt;

const DEFAULT_SNIPPETS_YAML: &str = include_str!("../../snippets/default.yaml");

pub mod commands;
pub mod crash;
pub mod engine;
pub mod expand;
pub mod form;
pub mod hook;
pub mod inject;
pub mod log_init;
pub mod matcher;
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

fn seed_default_snippets_if_empty(root: &std::path::Path) -> Result<(), String> {
    let has_existing_files = fs::read_dir(root)
        .map_err(|error| error.to_string())?
        .filter_map(Result::ok)
        .any(|entry| entry.path().is_file());
    if has_existing_files {
        return Ok(());
    }

    fs::write(root.join("default.yaml"), DEFAULT_SNIPPETS_YAML).map_err(|error| error.to_string())
}

fn notify_if_recovered_from_crash<R: Runtime>(
    app: &AppHandle<R>,
    prefs_state: &crate::commands::prefs::PrefsState,
) -> Result<(), String> {
    let mut prefs = crate::commands::prefs::get_prefs_inner(prefs_state);
    let checkpoint = prefs
        .last_crash_check
        .or_else(|| crate::crash::path_mtime_secs(prefs_state.path()))
        .unwrap_or(0);
    let Some(newest) = crate::crash::newest_crash_timestamp_after(checkpoint)? else {
        if prefs.last_crash_check.is_none() {
            prefs.last_crash_check = Some(crate::crash::current_timestamp_secs());
            crate::commands::prefs::set_prefs_inner(prefs_state, prefs)?;
        }
        return Ok(());
    };

    let _ = app
        .notification()
        .builder()
        .title("openmacro recovered from a crash")
        .body("openmacro recovered from a crash — see crashes folder")
        .show();
    prefs.last_crash_check = Some(newest);
    crate::commands::prefs::set_prefs_inner(prefs_state, prefs)
}

fn build_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let pause_resume = MenuItem::with_id(
        app,
        PAUSE_RESUME_MENU_ID,
        "Pause/Resume",
        true,
        None::<&str>,
    )?;
    let reload_snippets = MenuItem::with_id(
        app,
        RELOAD_SNIPPETS_MENU_ID,
        "Reload snippets",
        true,
        None::<&str>,
    )?;
    let open_snippets_folder_item = MenuItem::with_id(
        app,
        OPEN_SNIPPETS_FOLDER_MENU_ID,
        "Open snippets folder",
        true,
        None::<&str>,
    )?;
    let open_settings = MenuItem::with_id(
        app,
        OPEN_SETTINGS_MENU_ID,
        "Open settings",
        true,
        None::<&str>,
    )?;
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
        .icon(
            app.default_window_icon()
                .cloned()
                .expect("default window icon"),
        )
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
    crate::crash::install_panic_hook();
    let logging_config = crate::store::LoggingConfig::default();
    let log_handles = crate::log_init::init(&logging_config);

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            crate::commands::logging::get_logging_frontend_cfg,
            crate::commands::logging::get_log_ring,
            crate::commands::snippets::list_snippets,
            crate::commands::snippets::save_snippet,
            crate::commands::snippets::reload_snippets,
            crate::commands::snippets::list_load_errors,
            crate::commands::snippets::get_store_settings,
            crate::commands::snippets::set_store_settings,
            crate::commands::form::form_submit,
            crate::commands::form::form_cancel,
            crate::commands::prefs::get_prefs,
            crate::commands::prefs::set_prefs,
            crate::commands::sync::sync_test_connection,
            crate::commands::sync::sync_init,
            crate::commands::sync::sync_tick_now,
            crate::commands::sync::sync_status
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
        .setup(move |app| {
            let snippets_root = crate::commands::snippets::snippets_root().map_err(setup_error)?;
            fs::create_dir_all(&snippets_root).map_err(tauri::Error::from)?;
            seed_default_snippets_if_empty(&snippets_root).map_err(setup_error)?;
            let snippet_state =
                crate::commands::snippets::SnippetStoreState::load(snippets_root.clone())
                    .map_err(setup_error)?;
            let watcher = crate::store::watch_root(snippets_root)
                .map_err(|error| setup_error(error.to_string()))?;
            let mut rx = watcher.subscribe();
            let sync_rx = watcher.subscribe();
            let engine_rx = watcher.subscribe();
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
            let prefs_handle = prefs_state.prefs_handle();
            notify_if_recovered_from_crash(app.handle(), &prefs_state).map_err(setup_error)?;
            let form_runner = Arc::new(crate::form::FormRunner::new(app.handle().clone()));
            let runtime_form_runner = Arc::clone(&form_runner);
            let sync_state = crate::commands::sync::SyncCommandState::new(
                crate::commands::sync::sync_root().map_err(setup_error)?,
            );
            let sync_driver = crate::sync::spawn_driver(sync_state.backend(), sync_rx);
            let engine_handle = crate::engine::start_runtime(
                engine_rx,
                prefs_handle,
                runtime_form_runner,
                app.handle().clone(),
            )
            .map_err(setup_error)?;
            app.manage(snippet_state);
            app.manage(prefs_state);
            app.manage(logging_config);
            app.manage(log_handles);
            app.manage(form_runner);
            app.manage(sync_state);
            app.manage(sync_driver);
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
    use std::{
        fs,
        path::{Path, PathBuf},
    };
    use tempfile::TempDir;

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

    #[test]
    fn seeds_default_yaml_only_when_directory_is_empty() {
        let root = TempDir::new().unwrap();

        super::seed_default_snippets_if_empty(root.path()).unwrap();
        let contents = fs::read_to_string(root.path().join("default.yaml")).unwrap();

        assert!(contents.contains(";sig"));
        fs::write(root.path().join("custom.yaml"), "user").unwrap();
        super::seed_default_snippets_if_empty(root.path()).unwrap();
        assert_eq!(
            fs::read_to_string(root.path().join("custom.yaml")).unwrap(),
            "user"
        );
    }
}

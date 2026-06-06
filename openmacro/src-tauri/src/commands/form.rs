use std::{collections::BTreeMap, sync::Arc};

#[tauri::command]
pub fn form_submit(
    runner: tauri::State<'_, Arc<crate::form::FormRunner>>,
    snippet_id: String,
    values: BTreeMap<String, String>,
) -> Result<(), String> {
    runner.submit(&snippet_id, values);
    Ok(())
}

#[tauri::command]
pub fn form_cancel(
    runner: tauri::State<'_, Arc<crate::form::FormRunner>>,
    snippet_id: String,
) -> Result<(), String> {
    runner.cancel(&snippet_id);
    Ok(())
}

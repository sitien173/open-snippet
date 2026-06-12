use std::{collections::BTreeMap, sync::Arc};

#[tauri::command]
#[tracing::instrument(skip(runner, values), fields(snippet_id = %snippet_id, value_count = values.len()))]
pub fn form_submit(
    runner: tauri::State<'_, Arc<crate::form::FormRunner>>,
    snippet_id: String,
    values: BTreeMap<String, String>,
) -> Result<(), String> {
    // SECURITY: form values are user content; this command logs counts only.
    tracing::info!("form submitted");
    runner.submit(&snippet_id, values);
    Ok(())
}

#[tauri::command]
#[tracing::instrument(skip(runner), fields(snippet_id = %snippet_id))]
pub fn form_cancel(
    runner: tauri::State<'_, Arc<crate::form::FormRunner>>,
    snippet_id: String,
) -> Result<(), String> {
    tracing::info!("form cancelled");
    runner.cancel(&snippet_id);
    Ok(())
}

//! Planning commands. Each change tells every window the plan changed.

use tauri::{AppHandle, State};

use crate::error::IpcError;
use crate::plan::{self, GoalForm, PlanView, ProjectForm, QuickAddPreview, TaskForm};
use crate::state::AppState;

/// Runs a change, then tells every window.
fn change<T>(app: &AppHandle, result: crate::error::Result<T>) -> Result<T, IpcError> {
    let value = result?;
    plan::changed(app);
    Ok(value)
}

#[tauri::command]
#[specta::specta]
pub fn plan_view(state: State<'_, AppState>) -> Result<PlanView, IpcError> {
    Ok(plan::view(&state)?)
}

#[tauri::command]
#[specta::specta]
pub fn quick_add_preview(
    state: State<'_, AppState>,
    text: String,
) -> Result<QuickAddPreview, IpcError> {
    Ok(plan::preview_quick_add(&state, &text)?)
}

#[tauri::command]
#[specta::specta]
pub fn quick_add(
    app: AppHandle,
    state: State<'_, AppState>,
    text: String,
) -> Result<String, IpcError> {
    change(&app, plan::quick_add(&state, &text))
}

#[tauri::command]
#[specta::specta]
pub fn goal_create(
    app: AppHandle,
    state: State<'_, AppState>,
    form: GoalForm,
) -> Result<String, IpcError> {
    change(&app, plan::create_goal(&state, &form))
}

#[tauri::command]
#[specta::specta]
pub fn goal_update(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    form: GoalForm,
) -> Result<(), IpcError> {
    change(&app, plan::update_goal(&state, &id, &form))
}

#[tauri::command]
#[specta::specta]
pub fn goal_set_done(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    done: bool,
) -> Result<(), IpcError> {
    change(&app, plan::set_goal_done(&state, &id, done))
}

#[tauri::command]
#[specta::specta]
pub fn goal_delete(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<(), IpcError> {
    change(&app, plan::delete_goal(&state, &id))
}

#[tauri::command]
#[specta::specta]
pub fn milestone_create(
    app: AppHandle,
    state: State<'_, AppState>,
    goal_id: String,
    title: String,
    due_on: Option<String>,
) -> Result<String, IpcError> {
    change(
        &app,
        plan::create_milestone(&state, &goal_id, &title, due_on.as_deref()),
    )
}

#[tauri::command]
#[specta::specta]
pub fn milestone_set_done(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    done: bool,
) -> Result<(), IpcError> {
    change(&app, plan::set_milestone_done(&state, &id, done))
}

#[tauri::command]
#[specta::specta]
pub fn milestone_delete(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), IpcError> {
    change(&app, plan::delete_milestone(&state, &id))
}

#[tauri::command]
#[specta::specta]
pub fn project_create(
    app: AppHandle,
    state: State<'_, AppState>,
    form: ProjectForm,
) -> Result<String, IpcError> {
    change(&app, plan::create_project(&state, &form))
}

#[tauri::command]
#[specta::specta]
pub fn project_update(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    form: ProjectForm,
) -> Result<(), IpcError> {
    change(&app, plan::update_project(&state, &id, &form))
}

#[tauri::command]
#[specta::specta]
pub fn project_archive(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), IpcError> {
    change(&app, plan::archive_project(&state, &id))
}

#[tauri::command]
#[specta::specta]
pub fn task_create(
    app: AppHandle,
    state: State<'_, AppState>,
    form: TaskForm,
) -> Result<String, IpcError> {
    change(&app, plan::create_task(&state, &form))
}

#[tauri::command]
#[specta::specta]
pub fn task_update(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    form: TaskForm,
) -> Result<(), IpcError> {
    change(&app, plan::update_task(&state, &id, &form))
}

/// Finishes a task; a repeating one comes back on its next day.
#[tauri::command]
#[specta::specta]
pub fn task_complete(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), IpcError> {
    change(&app, plan::complete_task(&state, &id))
}

#[tauri::command]
#[specta::specta]
pub fn task_reopen(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<(), IpcError> {
    change(&app, plan::reopen_task(&state, &id))
}

#[tauri::command]
#[specta::specta]
pub fn task_delete(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<(), IpcError> {
    change(&app, plan::delete_task(&state, &id))
}

/// Puts a block on today's timeline.
#[tauri::command]
#[specta::specta]
pub fn block_create(
    app: AppHandle,
    state: State<'_, AppState>,
    task_id: Option<String>,
    title: String,
    start: String,
    minutes: u32,
) -> Result<String, IpcError> {
    change(
        &app,
        plan::create_block(&state, task_id.as_deref(), &title, &start, minutes),
    )
}

#[tauri::command]
#[specta::specta]
pub fn block_move(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    start: String,
    minutes: u32,
) -> Result<(), IpcError> {
    change(&app, plan::move_block(&state, &id, &start, minutes))
}

#[tauri::command]
#[specta::specta]
pub fn block_delete(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), IpcError> {
    change(&app, plan::delete_block(&state, &id))
}

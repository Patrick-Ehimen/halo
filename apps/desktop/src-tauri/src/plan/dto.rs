//! Planning types sent over IPC. Dates are local `YYYY-MM-DD`; times on the
//! day are minutes from local midnight, worked out in Rust.

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum PaceView {
    Done,
    Ahead,
    OnPace,
    Behind,
    JustStarted,
    Overdue,
    NoDeadline,
}

impl From<halo_plan::Pace> for PaceView {
    fn from(p: halo_plan::Pace) -> Self {
        match p {
            halo_plan::Pace::Done => PaceView::Done,
            halo_plan::Pace::Ahead => PaceView::Ahead,
            halo_plan::Pace::OnPace => PaceView::OnPace,
            halo_plan::Pace::Behind => PaceView::Behind,
            halo_plan::Pace::JustStarted => PaceView::JustStarted,
            halo_plan::Pace::Overdue => PaceView::Overdue,
            halo_plan::Pace::NoDeadline => PaceView::NoDeadline,
        }
    }
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MilestoneView {
    pub id: String,
    pub goal_id: String,
    pub title: String,
    pub due_on: Option<String>,
    /// "Due 6 Dec", "Done 2 Nov".
    pub label: String,
    pub done: bool,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GoalView {
    pub id: String,
    pub title: String,
    pub why: String,
    pub color: String,
    /// `active` or `done`.
    pub status: String,
    pub target_sessions: u32,
    pub due_on: Option<String>,
    /// "15 Mar 2027".
    pub due_label: Option<String>,
    /// "16 weeks left", "3 days left", "Due today", "2 weeks overdue".
    pub time_left: Option<String>,
    pub done_sessions: u32,
    pub percent: u32,
    pub pace: PaceView,
    /// Sessions a week to finish on time, in tenths (45 = 4.5).
    pub needed_per_week_tenths: Option<u32>,
    /// Sessions a week over the last four weeks, in tenths.
    pub averaged_per_week_tenths: u32,
    /// Finished sessions in the current week (Monday start).
    pub this_week: u32,
    /// Sessions still to do on today's tasks for this goal.
    pub today_sessions: u32,
    /// What the goal's tasks' estimates add up to (a hint for the target).
    pub estimate_total: u32,
    pub milestones: Vec<MilestoneView>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ProjectView {
    pub id: String,
    pub goal_id: Option<String>,
    pub name: String,
    pub color: String,
    pub description: String,
    pub open_tasks: u32,
    /// Estimated sessions left on its open tasks.
    pub sessions_left: u32,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TaskView {
    pub id: String,
    pub title: String,
    pub project_id: Option<String>,
    pub milestone_id: Option<String>,
    pub parent_id: Option<String>,
    pub estimate_sessions: u32,
    pub done_sessions: u32,
    pub planned_on: Option<String>,
    /// Minutes from local midnight, when due on the day being shown.
    pub due_minute: Option<u32>,
    /// Local time it's due, `HH:MM`, whatever the day (for editing).
    pub due_time: Option<String>,
    /// Where it ends on the timeline: its end time, else a 25-minute session
    /// per estimated one.
    pub due_end_minute: Option<u32>,
    /// Local `HH:MM` it ends, when set (for editing).
    pub end_time: Option<String>,
    pub description: String,
    /// "10:00", "Tomorrow 10:00", "Fri".
    pub when_label: Option<String>,
    pub recurrence: Option<String>,
    /// "Repeats weekdays".
    pub recurrence_label: Option<String>,
    pub completed: bool,
    /// Planned before today and still open.
    pub overdue: bool,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BlockView {
    pub id: String,
    pub task_id: Option<String>,
    pub title: String,
    /// `focus`, `event` or `break`.
    pub kind: String,
    pub start_minute: u32,
    pub end_minute: u32,
    /// "09:30–11:30".
    pub label: String,
    /// Local `HH:MM` it starts, and its length, for editing.
    pub start_time: String,
    pub minutes: u32,
}

/// Everything the Today plan and Goals screens draw.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PlanView {
    pub date: String,
    /// "Saturday, 21 November".
    pub date_label: String,
    /// Minutes from local midnight, for the "now" line.
    pub now_minute: u32,
    pub goals: Vec<GoalView>,
    pub projects: Vec<ProjectView>,
    /// Today's tasks (with their subtasks).
    pub today: Vec<TaskView>,
    /// Every open top-level task, for the Goals screen and the palette.
    pub open: Vec<TaskView>,
    pub blocks: Vec<BlockView>,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GoalForm {
    pub title: String,
    pub why: String,
    pub target_sessions: u32,
    pub due_on: Option<String>,
    pub color: String,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ProjectForm {
    pub name: String,
    pub color: String,
    pub goal_id: Option<String>,
    /// What the project is about. Optional.
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TaskForm {
    pub title: String,
    pub project_id: Option<String>,
    pub milestone_id: Option<String>,
    pub parent_id: Option<String>,
    pub estimate_sessions: u32,
    pub planned_on: Option<String>,
    /// Local `HH:MM` on `planned_on`.
    pub due_time: Option<String>,
    /// Local `HH:MM` it ends, after `due_time`. Optional.
    #[serde(default)]
    pub end_time: Option<String>,
    pub recurrence: Option<String>,
    /// What the task involves. Optional.
    #[serde(default)]
    pub description: String,
}

/// What quick add understood, for the chips shown while typing.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct QuickAddPreview {
    pub title: String,
    /// "Tomorrow", "Fri 27 Nov".
    pub date_label: Option<String>,
    /// "10:00".
    pub time_label: Option<String>,
    pub estimate: Option<u32>,
    pub project_name: Option<String>,
    /// False when `#name` matches no project (one will be made).
    pub project_exists: bool,
    pub goal_title: Option<String>,
    pub recurrence_label: Option<String>,
}

/// Emitted after anything in the plan changes, so every window refreshes.
#[derive(Debug, Clone, Serialize, Type, tauri_specta::Event)]
#[tauri_specta(event_name = "plan://changed")]
pub struct PlanChanged;

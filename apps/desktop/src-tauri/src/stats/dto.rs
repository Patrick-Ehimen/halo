//! Stats types sent over IPC. Every label is formatted in Rust.

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum StatsRange {
    Day,
    Week,
    Month,
    Year,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum TreeStageView {
    Seed,
    Sprout,
    Sapling,
    YoungTree,
    Tree,
    Bloom,
}

impl From<halo_stats::TreeStage> for TreeStageView {
    fn from(s: halo_stats::TreeStage) -> Self {
        match s {
            halo_stats::TreeStage::Seed => TreeStageView::Seed,
            halo_stats::TreeStage::Sprout => TreeStageView::Sprout,
            halo_stats::TreeStage::Sapling => TreeStageView::Sapling,
            halo_stats::TreeStage::YoungTree => TreeStageView::YoungTree,
            halo_stats::TreeStage::Tree => TreeStageView::Tree,
            halo_stats::TreeStage::Bloom => TreeStageView::Bloom,
        }
    }
}

/// A headline number: "Focus score · 82 · +6 vs last week".
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Kpi {
    pub label: String,
    pub value: String,
    pub detail: String,
    /// The detail is good news (shown in green).
    pub good: bool,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HeatDay {
    /// "Mon 16 Nov · 2h 10m".
    pub label: String,
    /// 0..=4; `null` for days still to come.
    pub level: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ProjectTime {
    pub name: String,
    pub color: String,
    /// "9h 06m".
    pub label: String,
    /// Share of the range's focus, in thousandths.
    pub permille: u32,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AchievementView {
    pub id: String,
    pub name: String,
    pub description: String,
    pub progress: u32,
    pub target: u32,
    /// "Earned 14 Nov", or "6 of 10".
    pub status: String,
    pub earned: bool,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GroveDay {
    /// "Mon".
    pub label: String,
    pub stage: TreeStageView,
    pub today: bool,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct StatsView {
    /// "23–29 Nov".
    pub range_label: String,
    pub kpis: Vec<Kpi>,
    /// 24 weeks, oldest first, Monday to Sunday.
    pub heatmap: Vec<Vec<HeatDay>>,
    /// Focus minutes for each hour of the day, 0 to 23, in the range.
    pub hours: Vec<u32>,
    /// "09:00–11:00", the best two hours in the range.
    pub best_hours: Option<String>,
    pub projects: Vec<ProjectTime>,
    pub achievements: Vec<AchievementView>,
    /// The last 14 days' trees.
    pub grove: Vec<GroveDay>,
}

/// Today at a glance: focus, the goal ring, the tree, the streak.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TodayFocus {
    pub focused_secs: u32,
    pub sessions: u32,
    /// "3h 20m".
    pub focused_label: String,
    /// "of your 5h goal".
    pub goal_label: String,
    /// Progress toward the daily goal, thousandths, capped at 1000.
    pub goal_permille: u32,
    pub tree: TreeStageView,
    pub streak: u32,
    pub streak_longest: u32,
    /// Today already counts toward the streak.
    pub streak_today: bool,
}

/// What the reflection card shows for the session that just ended.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ReflectionView {
    /// "50:00".
    pub focused_label: String,
    pub intention: String,
    pub distractions: u32,
    pub notes: String,
    /// "Start 10 min break" when a break is waiting to start.
    pub break_label: Option<String>,
}

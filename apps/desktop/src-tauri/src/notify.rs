//! Native notifications at phase changes. They only alert; every action stays
//! in Halo's own windows.

use std::sync::atomic::{AtomicBool, Ordering};

use halo_timer::{MINUTE, Phase, Transition};
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

/// Asked once per run, the first time a focus session starts.
static ASKED: AtomicBool = AtomicBool::new(false);

/// The notification for a transition, if it gets one: (title, body).
pub fn message(t: &Transition, planned_ms: u64) -> Option<(&'static str, String)> {
    match t {
        Transition::FocusEnded(s) if s.completed => Some((
            "Session complete",
            if s.planned_ms == 0 {
                format!(
                    "{} min of Flowtime. Take your break.",
                    s.focused_ms / MINUTE
                )
            } else {
                "Nice work. Time for a break.".to_string()
            },
        )),
        Transition::GoalReached => Some((
            "Goal reached",
            format!(
                "{} min done. Keep going, or end the session.",
                planned_ms / MINUTE
            ),
        )),
        Transition::BreakEnded { skipped: false } => {
            Some(("Break's over", "Ready for the next session.".to_string()))
        }
        _ => None,
    }
}

/// Sends notifications for these transitions, if the user wants them.
pub fn on_transitions(app: &AppHandle, transitions: &[Transition], planned_ms: u64) {
    if !crate::prefs::get(app).notifications {
        return;
    }
    for t in transitions {
        if matches!(t, Transition::PhaseStarted(Phase::Focus))
            && !ASKED.swap(true, Ordering::Relaxed)
        {
            // On macOS the system asks the first time Halo posts; this covers
            // platforms where permission is requested up front.
            if let Err(e) = app.notification().request_permission() {
                eprintln!("notification permission: {e}");
            }
        }
        if let Some((title, body)) = message(t, planned_ms)
            && let Err(e) = app.notification().builder().title(title).body(body).show()
        {
            eprintln!("notification: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use halo_timer::{Kind, SessionSummary};

    use super::*;

    fn summary(planned_ms: u64, focused_ms: u64, completed: bool) -> SessionSummary {
        SessionSummary {
            kind: Kind::Deep,
            started_at: 0,
            ended_at: focused_ms,
            planned_ms,
            focused_ms,
            overflow_ms: 0,
            pauses: 0,
            completed,
        }
    }

    #[test]
    fn completed_sessions_goals_and_breaks_notify() {
        let done = Transition::FocusEnded(summary(50 * MINUTE, 50 * MINUTE, true));
        assert_eq!(message(&done, 0).map(|m| m.0), Some("Session complete"));
        let flow = Transition::FocusEnded(summary(0, 40 * MINUTE, true));
        assert_eq!(
            message(&flow, 0).map(|m| m.1),
            Some("40 min of Flowtime. Take your break.".to_string())
        );
        assert_eq!(
            message(&Transition::GoalReached, 50 * MINUTE).map(|m| m.1),
            Some("50 min done. Keep going, or end the session.".to_string())
        );
        assert!(message(&Transition::BreakEnded { skipped: false }, 0).is_some());
    }

    #[test]
    fn quiet_for_the_rest() {
        let early = Transition::FocusEnded(summary(50 * MINUTE, 10 * MINUTE, false));
        assert_eq!(message(&early, 0), None);
        assert_eq!(message(&Transition::BreakEnded { skipped: true }, 0), None);
        assert_eq!(message(&Transition::CheckInDue, 0), None);
        assert_eq!(message(&Transition::PhaseStarted(Phase::Focus), 0), None);
    }
}

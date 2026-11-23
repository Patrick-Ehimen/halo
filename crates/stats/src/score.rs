//! The focus score, 0 to 100.
//!
//! A weighted mix of four parts, each 0..1:
//!
//! | Part        | Weight | What it measures |
//! | ----------- | ------ | ---------------- |
//! | consistency | 40%    | Each day's focus against the daily goal, capped at 1, averaged over the days |
//! | completion  | 30%    | Sessions that reached their goal (Flowtime always does) |
//! | quality     | 20%    | Post-session ratings: Rough 0, Okay ⅓, Good ⅔, In flow 1 |
//! | attention   | 10%    | Check-ins answered "focused" |
//!
//! A part with no data (no ratings, no check-ins) is left out and the other
//! weights are rescaled, so nobody is marked down for skipping a prompt.
//! No sessions at all scores 0.

use crate::Session;

/// The parts behind a score, each 0..=1 when present.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScoreParts {
    pub consistency: f64,
    pub completion: Option<f64>,
    pub quality: Option<f64>,
    pub attention: Option<f64>,
}

/// The score for a range: `daily_secs` is focus per day in the range
/// (future days excluded), `check_ins` is (focused, total).
pub fn focus_score(
    sessions: &[Session],
    daily_secs: &[u64],
    goal_secs: u64,
    check_ins: (u32, u32),
) -> (u32, ScoreParts) {
    let parts = ScoreParts {
        consistency: if daily_secs.is_empty() || goal_secs == 0 {
            0.0
        } else {
            daily_secs
                .iter()
                .map(|&s| (s as f64 / goal_secs as f64).min(1.0))
                .sum::<f64>()
                / daily_secs.len() as f64
        },
        completion: ratio(
            sessions.iter().filter(|s| s.completed()).count(),
            sessions.len(),
        ),
        quality: {
            let rated: Vec<f64> = sessions
                .iter()
                .filter_map(|s| s.rating)
                .filter(|r| (1..=4).contains(r))
                .map(|r| f64::from(r - 1) / 3.0)
                .collect();
            (!rated.is_empty()).then(|| rated.iter().sum::<f64>() / rated.len() as f64)
        },
        attention: ratio(check_ins.0 as usize, check_ins.1 as usize),
    };
    if sessions.is_empty() {
        return (0, parts);
    }
    let weighted = [
        (Some(parts.consistency), 0.4),
        (parts.completion, 0.3),
        (parts.quality, 0.2),
        (parts.attention, 0.1),
    ];
    let (sum, weight) = weighted
        .iter()
        .filter_map(|(v, w)| v.map(|v| (v * w, *w)))
        .fold((0.0, 0.0), |(s, w), (v, wt)| (s + v, w + wt));
    let score = if weight > 0.0 {
        sum / weight * 100.0
    } else {
        0.0
    };
    (score.round().clamp(0.0, 100.0) as u32, parts)
}

fn ratio(n: usize, of: usize) -> Option<f64> {
    (of > 0).then(|| n as f64 / of as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(focused: u32, planned: u32, rating: Option<u8>) -> Session {
        Session {
            started_at: 0,
            ended_at: i64::from(focused) * 1000,
            focused_secs: focused,
            planned_secs: planned,
            rating,
            distractions: 0,
            strict: false,
            project_id: None,
        }
    }

    const GOAL: u64 = 4 * 3600;

    #[test]
    fn a_perfect_week_scores_100() {
        let sessions = vec![s(3000, 3000, Some(4)); 5];
        let (score, _) = focus_score(&sessions, &[GOAL; 7], GOAL, (6, 6));
        assert_eq!(score, 100);
    }

    #[test]
    fn missing_parts_are_left_out() {
        // Half the goal each day, every session completed, no ratings or check-ins:
        // (0.5 × 0.4 + 1 × 0.3) / 0.7 = 71.4.
        let sessions = vec![s(3000, 3000, None); 3];
        let (score, parts) = focus_score(&sessions, &[GOAL / 2; 3], GOAL, (0, 0));
        assert_eq!(parts.quality, None);
        assert_eq!(parts.attention, None);
        assert_eq!(score, 71);
    }

    #[test]
    fn every_part_counts() {
        // consistency 0.25, completion 0.5, quality (Rough, In flow) 0.5, attention 0.5
        // = (0.1 + 0.15 + 0.1 + 0.05) = 40.
        let sessions = vec![s(3000, 3000, Some(1)), s(600, 3000, Some(4))];
        let (score, parts) = focus_score(&sessions, &[GOAL / 4], GOAL, (1, 2));
        assert_eq!(parts.completion, Some(0.5));
        assert_eq!(parts.quality, Some(0.5));
        assert_eq!(score, 40);
    }

    #[test]
    fn days_over_the_goal_are_capped() {
        let sessions = vec![s(3000, 0, None)];
        let (_, parts) = focus_score(&sessions, &[GOAL * 3, 0], GOAL, (0, 0));
        assert_eq!(parts.consistency, 0.5);
    }

    #[test]
    fn no_sessions_scores_zero() {
        assert_eq!(focus_score(&[], &[0; 7], GOAL, (0, 0)).0, 0);
    }
}

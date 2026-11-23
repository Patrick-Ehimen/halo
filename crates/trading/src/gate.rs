//! The unlock gate: exchanges unlock only when the checklist is done, the
//! head check is 3 or more, and the time is inside the trading window.

/// Where the user stands against their own rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gate {
    /// Head check 1–2: locked for the rest of the day.
    SitOut,
    /// Steps still to do: the checklist, the head check, or both.
    Steps { checklist: bool, head_check: bool },
    /// Rules met, waiting for the window to open.
    Ready { half_size: bool },
    /// Rules met and inside the window. Score 3 trades at half size, max 2 trades.
    Open { half_size: bool },
}

/// `head` is today's head check, 1–5, if any.
pub fn gate(checklist_done: bool, head: Option<u8>, in_window: bool) -> Gate {
    if head.is_some_and(|h| h <= 2) {
        return Gate::SitOut;
    }
    if !checklist_done || head.is_none() {
        return Gate::Steps {
            checklist: !checklist_done,
            head_check: head.is_none(),
        };
    }
    let half_size = head == Some(3);
    if in_window {
        Gate::Open { half_size }
    } else {
        Gate::Ready { half_size }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_combination() {
        for checklist in [false, true] {
            for head in [None, Some(1), Some(2), Some(3), Some(4), Some(5)] {
                for in_window in [false, true] {
                    let g = gate(checklist, head, in_window);
                    let expected = match head {
                        Some(1 | 2) => Gate::SitOut,
                        None => Gate::Steps {
                            checklist: !checklist,
                            head_check: true,
                        },
                        Some(_) if !checklist => Gate::Steps {
                            checklist: true,
                            head_check: false,
                        },
                        Some(h) if in_window => Gate::Open { half_size: h == 3 },
                        Some(h) => Gate::Ready { half_size: h == 3 },
                    };
                    assert_eq!(g, expected, "{checklist} {head:?} {in_window}");
                }
            }
        }
    }

    #[test]
    fn unlocks_only_with_all_three() {
        assert_eq!(gate(true, Some(4), true), Gate::Open { half_size: false });
        assert!(!matches!(gate(false, Some(4), true), Gate::Open { .. }));
        assert!(!matches!(gate(true, None, true), Gate::Open { .. }));
        assert!(!matches!(gate(true, Some(4), false), Gate::Open { .. }));
    }

    #[test]
    fn a_low_score_locks_even_with_the_checklist_done_in_window() {
        assert_eq!(gate(true, Some(2), true), Gate::SitOut);
        assert_eq!(gate(false, Some(1), false), Gate::SitOut);
    }

    #[test]
    fn three_means_half_size() {
        assert_eq!(gate(true, Some(3), true), Gate::Open { half_size: true });
        assert_eq!(gate(true, Some(3), false), Gate::Ready { half_size: true });
    }
}

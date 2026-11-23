//! The tree that grows with today's focus.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TreeStage {
    Seed,
    Sprout,
    Sapling,
    YoungTree,
    Tree,
    /// The daily goal is met.
    Bloom,
}

/// The stage for `focused` seconds today against the daily goal:
/// any focus sprouts it; a quarter, half, three quarters, and the goal grow it.
pub fn tree_stage(focused: u64, goal: u64) -> TreeStage {
    if focused == 0 {
        return TreeStage::Seed;
    }
    if goal == 0 || focused >= goal {
        return TreeStage::Bloom;
    }
    match focused * 4 / goal {
        0 => TreeStage::Sprout,
        1 => TreeStage::Sapling,
        2 => TreeStage::YoungTree,
        _ => TreeStage::Tree,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grows_through_the_day() {
        let goal = 4 * 3600;
        let stages: Vec<TreeStage> = [0, 60, 3600, 7200, 3 * 3600, 4 * 3600, 6 * 3600]
            .iter()
            .map(|&s| tree_stage(s, goal))
            .collect();
        assert_eq!(
            stages,
            [
                TreeStage::Seed,
                TreeStage::Sprout,
                TreeStage::Sapling,
                TreeStage::YoungTree,
                TreeStage::Tree,
                TreeStage::Bloom,
                TreeStage::Bloom
            ]
        );
    }
}

use super::ZhixingHeyi;
use super::constants::ATTR_JOURNAL_CARRYOVER;
use super::metadata::parse_carryover_count;
use crate::Result;

impl ZhixingHeyi {
    /// Checks for Heart-Demon blockers (tasks carried over too many times).
    ///
    /// # Errors
    /// Returns `Error::Logic` when at least one stale task blocks the user.
    pub fn check_heart_demon_blocker(&self) -> Result<()> {
        let tasks = self.graph.get_entities_by_type("OTHER(Task)");
        let stale_count = tasks
            .into_iter()
            .filter_map(|entity| entity.metadata.get(ATTR_JOURNAL_CARRYOVER).cloned())
            .filter_map(|v| parse_carryover_count(&v))
            .filter(|carryover| *carryover >= 3)
            .count();

        if stale_count > 0 {
            return Err(crate::Error::Logic(format!(
                "Blocked by {stale_count} Heart-Demons."
            )));
        }
        Ok(())
    }
}

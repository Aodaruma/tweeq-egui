use std::collections::BTreeSet;

use crate::ParamId;

/// Persistent parameter selection with spreadsheet-like range selection.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Selection {
    selected: BTreeSet<ParamId>,
    anchor: Option<ParamId>,
}

impl Selection {
    /// Selects only `id` and moves the range anchor.
    pub fn select_only(&mut self, id: ParamId) {
        self.selected.clear();
        self.selected.insert(id);
        self.anchor = Some(id);
    }

    /// Toggles one ID and moves the range anchor to an added ID.
    pub fn toggle(&mut self, id: ParamId) {
        if !self.selected.remove(&id) {
            self.selected.insert(id);
            self.anchor = Some(id);
        }
    }

    /// Selects the inclusive range between the anchor and `id`.
    pub fn select_range(&mut self, order: &[ParamId], id: ParamId) {
        let Some(anchor) = self.anchor else {
            self.select_only(id);
            return;
        };
        let Some(start) = order.iter().position(|candidate| *candidate == anchor) else {
            self.select_only(id);
            return;
        };
        let Some(end) = order.iter().position(|candidate| *candidate == id) else {
            self.select_only(id);
            return;
        };
        let (first, last) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };
        self.selected.extend(order[first..=last].iter().copied());
    }

    /// Returns whether `id` is selected.
    #[must_use]
    pub fn contains(&self, id: ParamId) -> bool {
        self.selected.contains(&id)
    }

    /// Returns selected IDs in deterministic ID order.
    pub fn iter(&self) -> impl Iterator<Item = ParamId> + '_ {
        self.selected.iter().copied()
    }

    /// Clears all selected parameters.
    pub fn clear(&mut self) {
        self.selected.clear();
        self.anchor = None;
    }
}

#[cfg(test)]
mod tests {
    use super::Selection;
    use crate::ParamId;

    #[test]
    fn range_selection_is_inclusive() {
        let ids = [1, 2, 3, 4].map(ParamId::from_u64);
        let mut selection = Selection::default();
        selection.select_only(ids[1]);
        selection.select_range(&ids, ids[3]);
        assert!(!selection.contains(ids[0]));
        assert!(selection.contains(ids[1]));
        assert!(selection.contains(ids[2]));
        assert!(selection.contains(ids[3]));
    }
}

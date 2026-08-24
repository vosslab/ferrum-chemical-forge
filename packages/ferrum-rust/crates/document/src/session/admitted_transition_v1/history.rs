//! Bounded logical navigation retained exclusively by admitted transitions.

use crate::session_state::RevisionState;

/// Opaque mutable timeline for renderer-admitted state transitions.
///
/// Its methods are visible only to the parent admitted-transition core. Route
/// modules receive read-only `DocumentSession` observations instead.
#[derive(Debug)]
pub(in crate::session) struct AdmittedHistoryV1 {
    entries: Vec<RevisionState>,
    cursor: usize,
    capacity: usize,
}

impl AdmittedHistoryV1 {
    pub(super) fn new(initial: RevisionState, capacity: usize) -> Self {
        Self {
            entries: vec![initial],
            cursor: 0,
            capacity: capacity.max(2),
        }
    }

    pub(super) fn current(&self) -> &RevisionState {
        &self.entries[self.cursor]
    }

    pub(super) fn current_mut(&mut self) -> &mut RevisionState {
        &mut self.entries[self.cursor]
    }

    /// Ensure the next append can complete without allocating.
    ///
    /// Preparation calls this while it can still report an allocation failure.
    /// The slot belongs to the timeline rather than one prepared transition,
    /// so sibling candidates and dropped candidates need no lease bookkeeping.
    pub(super) fn ensure_append_slot(&mut self) -> Result<(), std::collections::TryReserveError> {
        self.entries.try_reserve(1)
    }

    /// Append into the slot established during preparation.
    pub(super) fn append(&mut self, state: RevisionState) {
        self.entries.truncate(self.cursor + 1);
        debug_assert!(self.entries.capacity() > self.entries.len());
        self.entries.push(state);
        self.cursor = self.entries.len() - 1;
        while self.entries.len() > self.capacity {
            self.entries.remove(0);
            self.cursor -= 1;
        }
    }

    pub(super) fn undo_target(&self) -> Option<&RevisionState> {
        self.cursor.checked_sub(1).map(|index| &self.entries[index])
    }

    pub(super) fn redo_target(&self) -> Option<&RevisionState> {
        self.entries.get(self.cursor + 1)
    }

    pub(super) fn move_undo(&mut self) {
        self.cursor -= 1;
    }

    pub(super) fn move_redo(&mut self) {
        self.cursor += 1;
    }

    pub(super) fn replace_current(&mut self, state: RevisionState) {
        self.entries[self.cursor] = state;
    }

    #[cfg(test)]
    pub(super) fn set_current_revision_for_test(&mut self, revision: u64) {
        self.current_mut().set_revision_for_test(revision);
    }
}

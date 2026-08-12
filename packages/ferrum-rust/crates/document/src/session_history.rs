//! Bounded logical navigation over immutable document states.

use super::session_state::RevisionState;

/// A bounded full-snapshot timeline with an explicit logical cursor.
#[derive(Debug)]
pub(super) struct SessionHistory {
    entries: Vec<RevisionState>,
    cursor: usize,
    capacity: usize,
}

impl SessionHistory {
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
    pub(super) fn append(&mut self, state: RevisionState) {
        self.entries.truncate(self.cursor + 1);
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
}

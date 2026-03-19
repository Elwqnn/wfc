use rand::rngs::SmallRng;

use crate::bitset::Bitset;
use crate::rules::Rules;
use crate::state::State;

/// A snapshot of the solver state for backtracking.
struct Snapshot {
    wave: Bitset,
    compat: Vec<u16>,
    num_possible: Vec<usize>,
    weight_sum: Vec<f64>,
    wlog_sum: Vec<f64>,
    rng: SmallRng,
    /// The cell that was collapsed after this snapshot.
    cell: usize,
    /// The pattern that was chosen (to ban on backtrack).
    chosen: usize,
}

impl Snapshot {
    fn capture(state: &State) -> Self {
        Self {
            wave: state.wave.clone(),
            compat: state.compat.clone(),
            num_possible: state.num_possible.clone(),
            weight_sum: state.weight_sum.clone(),
            wlog_sum: state.wlog_sum.clone(),
            rng: state.rng.clone(),
            cell: 0,
            chosen: 0,
        }
    }

    fn restore(self, state: &mut State) {
        state.wave = self.wave;
        state.compat = self.compat;
        state.num_possible = self.num_possible;
        state.weight_sum = self.weight_sum;
        state.wlog_sum = self.wlog_sum;
        state.rng = self.rng;
        state.stack.clear();
        state.contradiction = false;
        state.done = false;
        state.last_collapsed = None;
    }
}

/// Manages backtracking state.
pub(crate) struct BacktrackState {
    snapshots: Vec<Snapshot>,
    pending_snapshot: Option<Snapshot>,
    collapse_count: usize,
    backtrack_count: usize,
    snapshot_interval: usize,
    max_backtracks: usize,
}

impl BacktrackState {
    pub(crate) fn new(snapshot_interval: usize, max_backtracks: usize) -> Self {
        Self {
            snapshots: Vec::new(),
            pending_snapshot: None,
            collapse_count: 0,
            backtrack_count: 0,
            snapshot_interval,
            max_backtracks,
        }
    }

    /// Called BEFORE collapse to capture pre-collapse state if needed.
    pub(crate) fn before_collapse(&mut self, state: &State) {
        self.collapse_count += 1;
        if self.collapse_count.is_multiple_of(self.snapshot_interval) {
            self.pending_snapshot = Some(Snapshot::capture(state));
        } else {
            self.pending_snapshot = None;
        }
    }

    /// Called AFTER collapse to record which cell/pattern was chosen.
    pub(crate) fn after_collapse(&mut self, cell: usize, chosen: usize) {
        if let Some(mut snapshot) = self.pending_snapshot.take() {
            snapshot.cell = cell;
            snapshot.chosen = chosen;
            self.snapshots.push(snapshot);
        }
    }

    /// Attempt to backtrack on contradiction.
    /// Returns true if backtrack succeeded, false if exhausted.
    pub(crate) fn try_backtrack(&mut self, state: &mut State, rules: &Rules) -> bool {
        while let Some(snapshot) = self.snapshots.pop() {
            self.backtrack_count += 1;
            if self.backtrack_count > self.max_backtracks {
                return false;
            }

            let banned_cell = snapshot.cell;
            let banned_pattern = snapshot.chosen;
            snapshot.restore(state);

            // Ban the pattern that led to contradiction
            state.ban(banned_cell, banned_pattern, rules);

            // Check if the cell still has possibilities
            if state.num_possible[banned_cell] == 0 {
                state.contradiction = true;
                continue;
            }

            return true;
        }
        false
    }
}

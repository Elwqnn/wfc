use rand::rngs::SmallRng;

use crate::bitset::Bitset;
use crate::rules::Rules;
use crate::state::State;

struct Snapshot {
    wave: Bitset,
    compat: Vec<u16>,
    num_possible: Vec<usize>,
    weight_sum: Vec<f64>,
    wlog_sum: Vec<f64>,
    rng: SmallRng,
    /// Cell collapsed after this snapshot was taken.
    cell: usize,
    /// Pattern chosen (banned on backtrack).
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

    /// Capture pre-collapse state if due for a snapshot.
    pub(crate) fn before_collapse(&mut self, state: &State) {
        self.collapse_count += 1;
        if self.collapse_count.is_multiple_of(self.snapshot_interval) {
            self.pending_snapshot = Some(Snapshot::capture(state));
        } else {
            self.pending_snapshot = None;
        }
    }

    /// Finalize pending snapshot with the collapse decision.
    pub(crate) fn after_collapse(&mut self, cell: usize, chosen: usize) {
        if let Some(mut snapshot) = self.pending_snapshot.take() {
            snapshot.cell = cell;
            snapshot.chosen = chosen;
            self.snapshots.push(snapshot);
        }
    }

    /// Restore to an earlier snapshot and ban the pattern that caused the contradiction.
    pub(crate) fn try_backtrack(&mut self, state: &mut State, rules: &Rules) -> bool {
        while let Some(snapshot) = self.snapshots.pop() {
            self.backtrack_count += 1;
            if self.backtrack_count > self.max_backtracks {
                return false;
            }

            let banned_cell = snapshot.cell;
            let banned_pattern = snapshot.chosen;
            snapshot.restore(state);

            state.ban(banned_cell, banned_pattern, rules);

            if state.num_possible[banned_cell] == 0 {
                state.contradiction = true;
                continue;
            }

            return true;
        }
        false
    }
}

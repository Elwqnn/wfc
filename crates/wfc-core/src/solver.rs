use rand::Rng;

use crate::backtrack::BacktrackState;
use crate::config::Config;
use crate::grid::Direction;
use crate::rules::{self, Rules};
use crate::state::State;
use crate::{Color, Sample};

/// WFC solver combining immutable rules with mutable state.
pub struct Wfc {
    pub(crate) rules: Rules,
    pub(crate) state: State,
    backtrack: Option<BacktrackState>,
    /// Reusable buffer for weighted pattern selection during collapse.
    candidates: Vec<(usize, f64)>,
}

impl Wfc {
    /// Create a new WFC instance from a sample.
    #[must_use]
    pub fn new(sample: &Sample, config: Config) -> Self {
        let backtrack = if config.backtracking {
            Some(BacktrackState::new(
                config.snapshot_interval,
                config.max_backtracks,
            ))
        } else {
            None
        };
        let rules = Rules::from_sample(sample, config);
        let state = State::new(&rules);

        let mut wfc = Self {
            rules,
            state,
            backtrack,
            candidates: Vec::new(),
        };
        wfc.apply_edge_constraints();
        wfc
    }

    #[must_use]
    pub fn is_done(&self) -> bool {
        self.state.done
    }

    #[must_use]
    pub fn has_contradiction(&self) -> bool {
        self.state.contradiction
    }

    #[must_use]
    pub fn last_collapsed(&self) -> Option<(usize, usize)> {
        self.state.last_collapsed
    }

    #[must_use]
    pub fn num_patterns(&self) -> usize {
        self.rules.num_patterns()
    }

    #[must_use]
    pub fn config(&self) -> &Config {
        &self.rules.config
    }

    fn apply_edge_constraints(&mut self) {
        let w = self.rules.config.output_width;
        let h = self.rules.config.output_height;
        let rules = &self.rules;
        let state = &mut self.state;

        if rules.config.ground {
            for cell in 0..w {
                for (p, mask) in rules.edge_mask.iter().enumerate() {
                    if state.wave.is_set(cell, p) && !mask[rules::TOP] {
                        state.ban(cell, p, rules);
                    }
                }
            }
            for x in 0..w {
                let cell = (h - 1) * w + x;
                for (p, mask) in rules.edge_mask.iter().enumerate() {
                    if state.wave.is_set(cell, p) && !mask[rules::BOTTOM] {
                        state.ban(cell, p, rules);
                    }
                }
            }
        }

        if rules.config.sides {
            for y in 0..h {
                let cell = y * w;
                for (p, mask) in rules.edge_mask.iter().enumerate() {
                    if state.wave.is_set(cell, p) && !mask[rules::LEFT] {
                        state.ban(cell, p, rules);
                    }
                }
            }
            for y in 0..h {
                let cell = y * w + (w - 1);
                for (p, mask) in rules.edge_mask.iter().enumerate() {
                    if state.wave.is_set(cell, p) && !mask[rules::RIGHT] {
                        state.ban(cell, p, rules);
                    }
                }
            }
        }

        Self::propagate_from(state, rules);
    }

    pub fn reset(&mut self) {
        self.state = State::new(&self.rules);
        if self.rules.config.backtracking {
            self.backtrack = Some(BacktrackState::new(
                self.rules.config.snapshot_interval,
                self.rules.config.max_backtracks,
            ));
        }
        self.apply_edge_constraints();
    }

    fn entropy(&self, cell: usize) -> f64 {
        let sum = self.state.weight_sum[cell];
        if sum <= 0.0 {
            return 0.0;
        }
        sum.ln() - self.state.wlog_sum[cell] / sum
    }

    #[must_use]
    pub fn normalized_entropy(&self, x: usize, y: usize) -> f64 {
        let cell = self.rules.grid.cell(x, y);
        if self.state.num_possible[cell] <= 1 {
            return 0.0;
        }
        let e = self.entropy(cell);
        (e / self.rules.starting_entropy).clamp(0.0, 1.0)
    }

    fn observe(&mut self) -> Option<usize> {
        let wave_size = self.state.num_possible.len();
        let mut min_entropy = f64::MAX;
        let mut min_cell = None;

        for cell in 0..wave_size {
            let count = self.state.num_possible[cell];
            if count == 0 {
                self.state.contradiction = true;
                return None;
            }
            if count == 1 {
                continue;
            }

            let entropy = self.entropy(cell) + self.state.rng.random::<f64>() * 1e-6;
            if entropy < min_entropy {
                min_entropy = entropy;
                min_cell = Some(cell);
            }
        }

        min_cell
    }

    /// Collapse a cell by choosing a pattern. Returns the chosen pattern index.
    fn collapse(&mut self, cell: usize) -> usize {
        let use_flex = self.rules.config.use_flexibility;

        // Pass 1: compute effective weights and total
        self.candidates.clear();
        let mut total: f64 = 0.0;

        for p in self.state.wave.iter_set(cell) {
            let w = if use_flex {
                self.rules.weight(p) * pattern_flexibility(&self.state, &self.rules, cell, p).sqrt()
            } else {
                self.rules.weight(p)
            };
            total += w;
            self.candidates.push((p, w));
        }

        if total <= 0.0 {
            self.state.contradiction = true;
            return 0;
        }

        // Pass 2: select pattern by weighted random
        let mut r = self.state.rng.random::<f64>() * total;
        let mut chosen = self.candidates[0].0;
        for &(p, w) in &self.candidates {
            r -= w;
            if r <= 0.0 {
                chosen = p;
                break;
            }
            chosen = p;
        }

        // Ban all other candidates (only visits live patterns, not 0..num_patterns)
        for &(p, _) in &self.candidates {
            if p != chosen {
                self.state.ban(cell, p, &self.rules);
            }
        }

        chosen
    }

    fn propagate(&mut self) {
        Self::propagate_from(&mut self.state, &self.rules);
    }

    /// Core propagation loop with explicit state/rules split.
    fn propagate_from(state: &mut State, rules: &Rules) {
        while let Some((cell, banned)) = state.stack.pop() {
            for dir in Direction::ALL {
                let Some(neighbor) = rules.grid.neighbor(cell, dir as usize) else {
                    continue;
                };
                let opp = dir.opposite() as usize;

                for &other in rules.propagator.compatible(banned, dir as usize) {
                    let ci = state.compat_index(neighbor, other as usize, opp);
                    state.compat[ci] -= 1;

                    if state.compat[ci] == 0 {
                        state.ban(neighbor, other as usize, rules);
                        if state.num_possible[neighbor] == 0 {
                            state.contradiction = true;
                            return;
                        }
                    }
                }
            }
        }
    }

    pub fn step(&mut self) -> bool {
        if self.state.done {
            return false;
        }

        // Handle contradiction with backtracking
        if self.state.contradiction {
            if let Some(bt) = &mut self.backtrack
                && bt.try_backtrack(&mut self.state, &self.rules)
            {
                self.propagate();
                return !self.state.contradiction;
            }
            return false;
        }

        match self.observe() {
            None => {
                if !self.state.contradiction {
                    self.state.done = true;
                }
                false
            }
            Some(cell) => {
                let (x, y) = self.rules.grid.coords(cell);
                self.state.last_collapsed = Some((x, y));

                if let Some(bt) = &mut self.backtrack {
                    bt.before_collapse(&self.state);
                }

                let chosen = self.collapse(cell);

                if let Some(bt) = &mut self.backtrack {
                    bt.after_collapse(cell, chosen);
                }

                self.propagate();
                true
            }
        }
    }

    pub fn run(&mut self) {
        while self.step() {}
    }

    #[must_use]
    pub fn is_collapsed(&self, x: usize, y: usize) -> bool {
        let cell = self.rules.grid.cell(x, y);
        self.state.num_possible[cell] == 1
    }

    #[must_use]
    pub fn get_color(&self, x: usize, y: usize) -> Color {
        let cell = self.rules.grid.cell(x, y);
        let count = self.state.num_possible[cell];

        match count {
            0 => [128, 0, 128],
            1 => self.rules.colors[self.state.wave.first_set(cell)],
            _ => {
                let (r, g, b, total) =
                    self.state
                        .wave
                        .iter_set(cell)
                        .fold((0.0, 0.0, 0.0, 0.0), |acc, p| {
                            let w = self.rules.weight(p);
                            let c = self.rules.colors[p];
                            (
                                acc.0 + c[0] as f64 * w,
                                acc.1 + c[1] as f64 * w,
                                acc.2 + c[2] as f64 * w,
                                acc.3 + w,
                            )
                        });
                [(r / total) as u8, (g / total) as u8, (b / total) as u8]
            }
        }
    }

    #[must_use]
    pub fn render(&self) -> Vec<Color> {
        let w = self.rules.config.output_width;
        let h = self.rules.config.output_height;
        let mut output = Vec::with_capacity(w * h);
        for y in 0..h {
            for x in 0..w {
                output.push(self.get_color(x, y));
            }
        }
        output
    }
}

/// Count compatible patterns still possible at neighbors for a given pattern.
fn pattern_flexibility(state: &State, rules: &Rules, cell: usize, pattern: usize) -> f64 {
    let mut flexibility: f64 = 0.0;

    for dir in Direction::ALL {
        let Some(neighbor) = rules.grid.neighbor(cell, dir as usize) else {
            flexibility += 1.0;
            continue;
        };
        let mut count = 0usize;
        for &compatible in rules.propagator.compatible(pattern, dir as usize) {
            if state.wave.is_set(neighbor, compatible as usize) {
                count += 1;
            }
        }
        flexibility += count as f64;
    }

    flexibility.max(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::default_pipe_sample;

    #[test]
    fn deterministic_seed_produces_same_result() {
        let sample = default_pipe_sample();
        let config = Config {
            seed: Some(42),
            ..Default::default()
        };

        let mut wfc1 = Wfc::new(&sample, config.clone());
        wfc1.run();

        let mut wfc2 = Wfc::new(&sample, config);
        wfc2.run();

        let render1 = wfc1.render();
        let render2 = wfc2.render();
        assert_eq!(render1, render2, "Same seed must produce identical output");
    }

    #[test]
    fn completes_without_panic() {
        let sample = default_pipe_sample();
        let config = Config {
            seed: Some(123),
            output_width: 16,
            output_height: 16,
            ..Default::default()
        };
        let mut wfc = Wfc::new(&sample, config);
        wfc.run();
        assert!(wfc.is_done() || wfc.has_contradiction());
    }

    #[test]
    fn reset_produces_fresh_state() {
        let sample = default_pipe_sample();
        let config = Config {
            seed: Some(42),
            ..Default::default()
        };
        let mut wfc = Wfc::new(&sample, config);
        wfc.run();
        wfc.reset();
        assert!(!wfc.is_done());
        assert!(!wfc.has_contradiction());
    }

    #[test]
    fn backtracking_reduces_contradictions() {
        let sample = default_pipe_sample();
        let runs = 50;

        let mut contradictions_without = 0;
        for seed in 0..runs {
            let config = Config {
                seed: Some(seed),
                output_width: 32,
                output_height: 32,
                backtracking: false,
                ..Default::default()
            };
            let mut wfc = Wfc::new(&sample, config);
            wfc.run();
            if wfc.has_contradiction() {
                contradictions_without += 1;
            }
        }

        let mut contradictions_with = 0;
        for seed in 0..runs {
            let config = Config {
                seed: Some(seed),
                output_width: 32,
                output_height: 32,
                backtracking: true,
                ..Default::default()
            };
            let mut wfc = Wfc::new(&sample, config);
            wfc.run();
            if wfc.has_contradiction() {
                contradictions_with += 1;
            }
        }

        assert!(
            contradictions_with <= contradictions_without,
            "Backtracking should not increase contradictions: with={} without={}",
            contradictions_with,
            contradictions_without
        );
    }

    #[test]
    fn backtracking_deterministic() {
        let sample = default_pipe_sample();
        let config = Config {
            seed: Some(42),
            backtracking: true,
            ..Default::default()
        };

        let mut wfc1 = Wfc::new(&sample, config.clone());
        wfc1.run();

        let mut wfc2 = Wfc::new(&sample, config);
        wfc2.run();

        let render1 = wfc1.render();
        let render2 = wfc2.render();
        assert_eq!(
            render1, render2,
            "Same seed + backtracking must produce identical output"
        );
    }
}

use crate::Color;
use crate::rules::Rules;
use crate::state::State;

/// User-defined constraint applied to the wave before solving.
pub trait CellConstraint {
    fn apply(&self, ctx: &mut ConstraintContext);
}

/// Wave state handle for applying constraints.
pub struct ConstraintContext<'a> {
    pub(crate) state: &'a mut State,
    pub(crate) rules: &'a Rules,
}

impl<'a> ConstraintContext<'a> {
    pub(crate) fn new(state: &'a mut State, rules: &'a Rules) -> Self {
        Self { state, rules }
    }

    #[must_use]
    pub fn grid_width(&self) -> usize {
        self.rules.config.output_width
    }

    #[must_use]
    pub fn grid_height(&self) -> usize {
        self.rules.config.output_height
    }

    #[must_use]
    pub fn num_patterns(&self) -> usize {
        self.rules.num_patterns()
    }

    #[must_use]
    pub fn is_possible(&self, x: usize, y: usize, pattern: usize) -> bool {
        let cell = self.rules.grid.cell(x, y);
        self.state.wave.is_set(cell, pattern)
    }

    /// Top-left color of a pattern.
    #[must_use]
    pub fn pattern_color(&self, pattern: usize) -> Color {
        self.rules.colors[pattern]
    }

    #[must_use]
    pub fn pattern_color_at(&self, pattern: usize, px: usize, py: usize) -> Color {
        self.rules.patterns[pattern].get(px, py)
    }

    /// No-op if already banned.
    pub fn ban(&mut self, x: usize, y: usize, pattern: usize) {
        let cell = self.rules.grid.cell(x, y);
        self.state.ban(cell, pattern, self.rules);
    }

    pub fn ban_all_except(&mut self, x: usize, y: usize, keep: usize) {
        let cell = self.rules.grid.cell(x, y);
        let np = self.rules.num_patterns();
        for p in 0..np {
            if p != keep {
                self.state.ban(cell, p, self.rules);
            }
        }
    }
}

use rand::SeedableRng;
use rand::rngs::SmallRng;

use crate::bitset::Bitset;
use crate::rules::Rules;

pub struct State {
    pub(crate) wave: Bitset,
    /// `compat[(cell * num_patterns + pattern) * 4 + dir]`
    pub(crate) compat: Vec<u16>,
    pub(crate) num_patterns: usize,
    pub(crate) num_possible: Vec<usize>,
    pub(crate) weight_sum: Vec<f64>,
    /// Sum of w*ln(w) per cell, for entropy calculation.
    pub(crate) wlog_sum: Vec<f64>,
    /// (cell, banned_pattern) pairs pending propagation.
    pub(crate) stack: Vec<(usize, usize)>,
    pub(crate) contradiction: bool,
    pub(crate) done: bool,
    pub(crate) last_collapsed: Option<(usize, usize)>,
    pub(crate) rng: SmallRng,
}

impl State {
    pub fn new(rules: &Rules) -> Self {
        let num_patterns = rules.num_patterns();
        let wave_size = rules.grid.size();

        let total_weight: f64 = rules.weight_table.iter().map(|(w, _)| w).sum();
        let wlog: f64 = rules.weight_table.iter().map(|(w, lw)| w * lw).sum();

        let rng = match rules.config.seed {
            Some(seed) => SmallRng::seed_from_u64(seed),
            None => SmallRng::from_os_rng(),
        };

        let block = num_patterns * 4;
        let mut compat = vec![0u16; wave_size * block];
        for cell in 0..wave_size {
            let start = cell * block;
            compat[start..start + block].copy_from_slice(&rules.base_compat);
        }

        let mut state = Self {
            wave: Bitset::new(wave_size, num_patterns),
            compat,
            num_patterns,
            num_possible: vec![num_patterns; wave_size],
            weight_sum: vec![total_weight; wave_size],
            wlog_sum: vec![wlog; wave_size],
            stack: Vec::new(),
            contradiction: false,
            done: false,
            last_collapsed: None,
            rng,
        };

        // Pre-ban non-viable patterns from every cell
        for cell in 0..wave_size {
            for p in 0..num_patterns {
                if !rules.viable[p] {
                    state.ban(cell, p, rules);
                }
            }
        }
        // Clear stack -- these bans don't need propagation since all
        // non-viable patterns are removed uniformly
        state.stack.clear();

        state
    }

    #[inline]
    pub(crate) fn compat_index(&self, cell: usize, pattern: usize, dir: usize) -> usize {
        (cell * self.num_patterns + pattern) * 4 + dir
    }

    #[inline(always)]
    pub(crate) fn ban(&mut self, cell: usize, pattern: usize, rules: &Rules) {
        if !self.wave.is_set(cell, pattern) {
            return;
        }
        self.wave.clear(cell, pattern);
        self.num_possible[cell] -= 1;
        let (w, lw) = rules.weight_table[pattern];
        self.weight_sum[cell] -= w;
        self.wlog_sum[cell] -= w * lw;
        self.stack.push((cell, pattern));
    }
}

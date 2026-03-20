use std::collections::HashMap;

use crate::config::Config;
use crate::grid::{Direction, Grid};
use crate::{Color, Pattern, Sample};

/// Contiguous storage of compatible pattern indices per (pattern, direction).
pub(crate) struct FlatPropagator {
    data: Vec<u16>,
    /// `offsets[pattern * 4 + dir]` = (start, end) into data
    offsets: Vec<(u32, u32)>,
}

impl FlatPropagator {
    #[inline]
    pub(crate) fn compatible(&self, pattern: usize, dir: usize) -> &[u16] {
        let idx = pattern * 4 + dir;
        let (start, end) = self.offsets[idx];
        &self.data[start as usize..end as usize]
    }
}

pub(crate) const TOP: usize = 0;
pub(crate) const BOTTOM: usize = 1;
pub(crate) const LEFT: usize = 2;
pub(crate) const RIGHT: usize = 3;

/// Immutable rules derived from the sample.
pub struct Rules {
    pub(crate) config: Config,
    pub(crate) grid: Grid,
    pub(crate) patterns: Vec<Pattern>,
    /// `(weight, log_weight)` per pattern.
    pub(crate) weight_table: Vec<(f64, f64)>,
    pub(crate) propagator: FlatPropagator,
    pub(crate) starting_entropy: f64,
    /// Base compatibility counts per (pattern, direction).
    pub(crate) base_compat: Vec<u16>,
    /// `edge_mask[pattern]`: sample edges where this pattern appeared.
    pub(crate) edge_mask: Vec<[bool; 4]>,
    /// Patterns with at least one neighbor in every direction.
    pub(crate) viable: Vec<bool>,
    /// Top-left color per pattern (render cache).
    pub(crate) colors: Vec<Color>,
}

impl Rules {
    pub fn from_sample(sample: &Sample, config: Config) -> Self {
        let grid = Grid::new(config.output_width, config.output_height, config.boundary);
        let extracted = Self::extract_patterns(sample, &config);
        let propagator = Self::build_propagator(&extracted.patterns, &config);

        let patterns = extracted.patterns;
        let edge_mask = extracted.edge_mask;

        let weight_table: Vec<(f64, f64)> =
            extracted.weights.iter().map(|&w| (w, w.ln())).collect();
        let total_weight: f64 = weight_table.iter().map(|(w, _)| w).sum();
        let sum_wlog: f64 = weight_table.iter().map(|(w, lw)| w * lw).sum();
        let starting_entropy = total_weight.ln() - sum_wlog / total_weight;

        let num_patterns = patterns.len();
        let viable = Self::compute_viable(&propagator, num_patterns);

        // Precompute base_compat[t * 4 + d] considering only viable patterns
        let mut base_compat = vec![0u16; num_patterns * 4];
        for p in 0..num_patterns {
            if !viable[p] {
                continue;
            }
            for dir in Direction::ALL {
                let opp = dir.opposite() as usize;
                for &t in propagator.compatible(p, dir as usize) {
                    if viable[t as usize] {
                        base_compat[t as usize * 4 + opp] += 1;
                    }
                }
            }
        }

        let colors: Vec<Color> = patterns.iter().map(|p| p.get(0, 0)).collect();

        Self {
            config,
            grid,
            patterns,
            weight_table,
            propagator,
            starting_entropy,
            base_compat,
            edge_mask,
            viable,
            colors,
        }
    }

    #[inline]
    pub fn num_patterns(&self) -> usize {
        self.patterns.len()
    }

    #[inline]
    pub(crate) fn weight(&self, p: usize) -> f64 {
        self.weight_table[p].0
    }

    /// Fixpoint: remove patterns with no viable neighbor in any direction.
    fn compute_viable(propagator: &FlatPropagator, num_patterns: usize) -> Vec<bool> {
        let mut viable = vec![true; num_patterns];
        loop {
            let mut changed = false;
            for p in 0..num_patterns {
                if !viable[p] {
                    continue;
                }
                for dir in Direction::ALL {
                    let has_viable = propagator
                        .compatible(p, dir as usize)
                        .iter()
                        .any(|&t| viable[t as usize]);
                    if !has_viable {
                        viable[p] = false;
                        changed = true;
                        break;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        viable
    }

    fn extract_patterns(sample: &Sample, config: &Config) -> ExtractedPatterns {
        let n = config.pattern_size;
        let mut pattern_counts: HashMap<Pattern, usize> = HashMap::new();
        let mut pattern_edges: HashMap<Pattern, [bool; 4]> = HashMap::new();

        let x_max = if config.periodic_input {
            sample.width
        } else {
            sample.width.saturating_sub(n - 1)
        };
        let y_max = if config.periodic_input {
            sample.height
        } else {
            sample.height.saturating_sub(n - 1)
        };

        for y in 0..y_max {
            for x in 0..x_max {
                let mut pixels = Vec::with_capacity(n * n);
                for dy in 0..n {
                    for dx in 0..n {
                        let sx = (x + dx) % sample.width;
                        let sy = (y + dy) % sample.height;
                        pixels.push(sample.get(sx, sy));
                    }
                }
                let pattern = Pattern::new(n, pixels);

                let variants = if config.symmetry {
                    if config.ground || config.sides {
                        vec![pattern.clone(), pattern.reflect()]
                    } else {
                        pattern.symmetries()
                    }
                } else {
                    vec![pattern]
                };

                for variant in variants {
                    *pattern_counts.entry(variant.clone()).or_insert(0) += 1;
                    let edges = pattern_edges.entry(variant).or_insert([false; 4]);
                    if y == 0 {
                        edges[TOP] = true;
                    }
                    if y + n >= sample.height {
                        edges[BOTTOM] = true;
                    }
                    if x == 0 {
                        edges[LEFT] = true;
                    }
                    if x + n >= sample.width {
                        edges[RIGHT] = true;
                    }
                }
            }
        }

        let mut pairs: Vec<_> = pattern_counts.into_iter().collect();
        pairs.sort_by(|(a, _), (b, _)| a.cmp(b));

        let mut patterns = Vec::with_capacity(pairs.len());
        let mut weights = Vec::with_capacity(pairs.len());
        let mut edge_mask = Vec::with_capacity(pairs.len());
        for (pattern, count) in pairs {
            let edges = pattern_edges.get(&pattern).copied().unwrap_or([false; 4]);
            edge_mask.push(edges);
            patterns.push(pattern);
            weights.push(count as f64);
        }

        ExtractedPatterns {
            patterns,
            weights,
            edge_mask,
        }
    }

    fn build_propagator(patterns: &[Pattern], config: &Config) -> FlatPropagator {
        let n = config.pattern_size;
        let num_patterns = patterns.len();

        // Hash overlap strips to find compatible pairs in O(P) instead of O(P^2).
        // Right: p1 cols [1..n] must match p2 cols [0..n-1], etc.
        let mut nested_vecs = vec![vec![Vec::<u16>::new(); 4]; num_patterns];

        // Right: p1 cols [1..n] == p2 cols [0..n-1]
        Self::fill_compatible_hashed(
            patterns,
            n,
            &mut nested_vecs,
            Direction::Right as usize,
            |p, n| Self::hash_cols(p, 1, n, n),
            |p, n| Self::hash_cols(p, 0, n - 1, n),
        );
        // Down: p1 rows [1..n] == p2 rows [0..n-1]
        Self::fill_compatible_hashed(
            patterns,
            n,
            &mut nested_vecs,
            Direction::Down as usize,
            |p, n| Self::hash_rows(p, 1, n, n),
            |p, n| Self::hash_rows(p, 0, n - 1, n),
        );
        // Left: p1 cols [0..n-1] == p2 cols [1..n]
        Self::fill_compatible_hashed(
            patterns,
            n,
            &mut nested_vecs,
            Direction::Left as usize,
            |p, n| Self::hash_cols(p, 0, n - 1, n),
            |p, n| Self::hash_cols(p, 1, n, n),
        );
        // Up: p1 rows [0..n-1] == p2 rows [1..n]
        Self::fill_compatible_hashed(
            patterns,
            n,
            &mut nested_vecs,
            Direction::Up as usize,
            |p, n| Self::hash_rows(p, 0, n - 1, n),
            |p, n| Self::hash_rows(p, 1, n, n),
        );

        // Flatten into contiguous layout
        let mut data = Vec::new();
        let mut offsets = Vec::with_capacity(num_patterns * 4);
        for dirs in &nested_vecs {
            for compat in dirs {
                let start = data.len() as u32;
                data.extend_from_slice(compat);
                let end = data.len() as u32;
                offsets.push((start, end));
            }
        }

        FlatPropagator { data, offsets }
    }

    /// Hash-match one direction: candidates by hash, then verify pixels.
    fn fill_compatible_hashed(
        patterns: &[Pattern],
        n: usize,
        nested: &mut [Vec<Vec<u16>>],
        dir: usize,
        source_hash: impl Fn(&Pattern, usize) -> u64,
        target_hash: impl Fn(&Pattern, usize) -> u64,
    ) {
        let mut target_map: HashMap<u64, Vec<u16>> = HashMap::new();
        for (j, p2) in patterns.iter().enumerate() {
            target_map
                .entry(target_hash(p2, n))
                .or_default()
                .push(j as u16);
        }

        for (i, p1) in patterns.iter().enumerate() {
            let h = source_hash(p1, n);
            if let Some(candidates) = target_map.get(&h) {
                for &j in candidates {
                    let p2 = &patterns[j as usize];
                    if Self::strips_match(p1, p2, dir, n) {
                        nested[i][dir].push(j);
                    }
                }
            }
        }
    }

    /// Pixel-level overlap strip verification.
    fn strips_match(p1: &Pattern, p2: &Pattern, dir: usize, n: usize) -> bool {
        let (dx, dy): (i32, i32) = match dir {
            0 => (1, 0),  // Right
            1 => (0, 1),  // Down
            2 => (-1, 0), // Left
            _ => (0, -1), // Up
        };
        let xmin = dx.max(0) as usize;
        let xmax = (n as i32 + dx.min(0)) as usize;
        let ymin = dy.max(0) as usize;
        let ymax = (n as i32 + dy.min(0)) as usize;

        for y in ymin..ymax {
            for x in xmin..xmax {
                let x2 = (x as i32 - dx) as usize;
                let y2 = (y as i32 - dy) as usize;
                if p1.get(x, y) != p2.get(x2, y2) {
                    return false;
                }
            }
        }
        true
    }

    /// Hash columns [col_start..col_end) of a pattern (all rows).
    fn hash_cols(p: &Pattern, col_start: usize, col_end: usize, n: usize) -> u64 {
        let mut h: u64 = 0xcbf29ce484222325; // FNV offset basis
        for y in 0..n {
            for x in col_start..col_end {
                let c = p.get(x, y);
                h = h.wrapping_mul(0x100000001b3); // FNV prime
                h ^= c[0] as u64;
                h = h.wrapping_mul(0x100000001b3);
                h ^= c[1] as u64;
                h = h.wrapping_mul(0x100000001b3);
                h ^= c[2] as u64;
            }
        }
        h
    }

    /// Hash rows [row_start..row_end) of a pattern (all columns).
    fn hash_rows(p: &Pattern, row_start: usize, row_end: usize, n: usize) -> u64 {
        let mut h: u64 = 0xcbf29ce484222325;
        for y in row_start..row_end {
            for x in 0..n {
                let c = p.get(x, y);
                h = h.wrapping_mul(0x100000001b3);
                h ^= c[0] as u64;
                h = h.wrapping_mul(0x100000001b3);
                h ^= c[1] as u64;
                h = h.wrapping_mul(0x100000001b3);
                h ^= c[2] as u64;
            }
        }
        h
    }
}

struct ExtractedPatterns {
    patterns: Vec<Pattern>,
    weights: Vec<f64>,
    edge_mask: Vec<[bool; 4]>,
}

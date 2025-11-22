use std::collections::{HashMap, HashSet};

use rand::Rng;

use crate::{Color, Pattern, Sample};

/// Pattern extraction result with edge constraint sets
type PatternExtraction = (
    Vec<Pattern>,
    Vec<f64>,
    HashSet<Pattern>,
    HashSet<Pattern>,
    HashSet<Pattern>,
    HashSet<Pattern>,
);

/// Configuration for WFC
#[derive(Clone)]
pub struct WfcConfig {
    /// Pattern size (N in NxN)
    pub pattern_size: usize,
    /// Output width in cells
    pub output_width: usize,
    /// Output height in cells
    pub output_height: usize,
    /// Whether to wrap around edges
    pub periodic_input: bool,
    pub periodic_output: bool,
    /// Whether to include rotations/reflections
    pub symmetry: bool,
    /// Constrain patterns at edges based on sample position
    pub ground: bool,
    /// Constrain left/right edges
    pub sides: bool,
}

impl Default for WfcConfig {
    fn default() -> Self {
        Self {
            pattern_size: 3,
            output_width: 32,
            output_height: 32,
            periodic_input: true,
            periodic_output: false,
            symmetry: true,
            ground: false,
            sides: false,
        }
    }
}

/// Direction for adjacency checking
#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Right = 0,
    Down = 1,
    Left = 2,
    Up = 3,
}

impl Direction {
    pub const ALL: [Direction; 4] = [
        Direction::Right,
        Direction::Down,
        Direction::Left,
        Direction::Up,
    ];

    pub fn opposite(self) -> Self {
        match self {
            Direction::Right => Direction::Left,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Up => Direction::Down,
        }
    }

    pub fn dx(self) -> i32 {
        match self {
            Direction::Right => 1,
            Direction::Left => -1,
            _ => 0,
        }
    }

    pub fn dy(self) -> i32 {
        match self {
            Direction::Down => 1,
            Direction::Up => -1,
            _ => 0,
        }
    }
}

/// WFC state
pub struct Wfc {
    pub config: WfcConfig,
    pub patterns: Vec<Pattern>,
    pub weights: Vec<f64>,
    propagator: Vec<Vec<Vec<usize>>>,
    wave: Vec<Vec<bool>>,
    sumsone: Vec<usize>,
    sumweights: Vec<f64>,
    sumweightlogweights: Vec<f64>,
    log_weights: Vec<f64>,
    starting_entropy: f64,
    stack: Vec<(usize, usize)>,
    pub contradiction: bool,
    pub done: bool,
    pub last_collapsed: Option<(usize, usize)>,
    top_patterns: Vec<bool>,
    bottom_patterns: Vec<bool>,
    left_patterns: Vec<bool>,
    right_patterns: Vec<bool>,
}

impl Wfc {
    /// Create a new WFC instance from a sample
    pub fn new(sample: &Sample, config: WfcConfig) -> Self {
        let (patterns, weights, top_set, bottom_set, left_set, right_set) =
            Self::extract_patterns(sample, &config);
        let propagator = Self::build_propagator(&patterns, &config);
        let num_patterns = patterns.len();

        let top_patterns: Vec<bool> = patterns.iter().map(|p| top_set.contains(p)).collect();
        let bottom_patterns: Vec<bool> = patterns.iter().map(|p| bottom_set.contains(p)).collect();
        let left_patterns: Vec<bool> = patterns.iter().map(|p| left_set.contains(p)).collect();
        let right_patterns: Vec<bool> = patterns.iter().map(|p| right_set.contains(p)).collect();

        let wave_size = config.output_width * config.output_height;
        let wave = vec![vec![true; num_patterns]; wave_size];

        let total_weight: f64 = weights.iter().sum();
        let log_weights: Vec<f64> = weights.iter().map(|&w| w.ln()).collect();
        let sumweightlogweight: f64 = weights.iter().zip(&log_weights).map(|(w, lw)| w * lw).sum();
        let starting_entropy = total_weight.ln() - sumweightlogweight / total_weight;

        let mut wfc = Self {
            config,
            patterns,
            weights: weights.clone(),
            propagator,
            wave,
            sumsone: vec![num_patterns; wave_size],
            sumweights: vec![total_weight; wave_size],
            sumweightlogweights: vec![sumweightlogweight; wave_size],
            log_weights,
            starting_entropy,
            stack: Vec::new(),
            contradiction: false,
            done: false,
            last_collapsed: None,
            top_patterns,
            bottom_patterns,
            left_patterns,
            right_patterns,
        };

        wfc.apply_edge_constraints();
        wfc
    }

    fn apply_edge_constraints(&mut self) {
        let w = self.config.output_width;
        let h = self.config.output_height;

        if self.config.ground {
            for x in 0..w {
                let cell = x;
                for p in 0..self.patterns.len() {
                    if self.wave[cell][p] && !self.top_patterns[p] {
                        self.ban(cell, p);
                    }
                }
            }

            for x in 0..w {
                let cell = (h - 1) * w + x;
                for p in 0..self.patterns.len() {
                    if self.wave[cell][p] && !self.bottom_patterns[p] {
                        self.ban(cell, p);
                    }
                }
            }
        }

        if self.config.sides {
            for y in 0..h {
                let cell = y * w;
                for p in 0..self.patterns.len() {
                    if self.wave[cell][p] && !self.left_patterns[p] {
                        self.ban(cell, p);
                    }
                }
            }

            for y in 0..h {
                let cell = y * w + (w - 1);
                for p in 0..self.patterns.len() {
                    if self.wave[cell][p] && !self.right_patterns[p] {
                        self.ban(cell, p);
                    }
                }
            }
        }

        self.propagate();
    }

    fn extract_patterns(sample: &Sample, config: &WfcConfig) -> PatternExtraction {
        let n = config.pattern_size;
        let mut pattern_counts: HashMap<Pattern, usize> = HashMap::new();
        let mut top_set: HashSet<Pattern> = HashSet::new();
        let mut bottom_set: HashSet<Pattern> = HashSet::new();
        let mut left_set: HashSet<Pattern> = HashSet::new();
        let mut right_set: HashSet<Pattern> = HashSet::new();

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
                    if y == 0 {
                        top_set.insert(variant.clone());
                    }
                    if y + n >= sample.height {
                        bottom_set.insert(variant.clone());
                    }
                    if x == 0 {
                        left_set.insert(variant.clone());
                    }
                    if x + n >= sample.width {
                        right_set.insert(variant);
                    }
                }
            }
        }

        let mut patterns = Vec::new();
        let mut weights = Vec::new();
        for (pattern, count) in pattern_counts {
            patterns.push(pattern);
            weights.push(count as f64);
        }

        (patterns, weights, top_set, bottom_set, left_set, right_set)
    }

    fn build_propagator(patterns: &[Pattern], config: &WfcConfig) -> Vec<Vec<Vec<usize>>> {
        let n = config.pattern_size;
        let num_patterns = patterns.len();

        let mut propagator = vec![vec![Vec::new(); 4]; num_patterns];

        for (i, p1) in patterns.iter().enumerate() {
            for (j, p2) in patterns.iter().enumerate() {
                if Self::patterns_agree(p1, p2, 1, 0, n) {
                    propagator[i][Direction::Right as usize].push(j);
                }
                if Self::patterns_agree(p1, p2, 0, 1, n) {
                    propagator[i][Direction::Down as usize].push(j);
                }
                if Self::patterns_agree(p1, p2, -1, 0, n) {
                    propagator[i][Direction::Left as usize].push(j);
                }
                if Self::patterns_agree(p1, p2, 0, -1, n) {
                    propagator[i][Direction::Up as usize].push(j);
                }
            }
        }

        propagator
    }

    fn patterns_agree(p1: &Pattern, p2: &Pattern, dx: i32, dy: i32, n: usize) -> bool {
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

    pub fn reset(&mut self) {
        let num_patterns = self.patterns.len();
        let wave_size = self.config.output_width * self.config.output_height;

        self.wave = vec![vec![true; num_patterns]; wave_size];

        let total_weight: f64 = self.weights.iter().sum();
        let sumweightlogweight: f64 = self
            .weights
            .iter()
            .zip(&self.log_weights)
            .map(|(w, lw)| w * lw)
            .sum();

        self.sumsone = vec![num_patterns; wave_size];
        self.sumweights = vec![total_weight; wave_size];
        self.sumweightlogweights = vec![sumweightlogweight; wave_size];
        self.stack.clear();
        self.contradiction = false;
        self.done = false;
        self.last_collapsed = None;

        self.apply_edge_constraints();
    }

    fn cell_index(&self, x: usize, y: usize) -> usize {
        y * self.config.output_width + x
    }

    fn entropy(&self, cell: usize) -> f64 {
        let sum = self.sumweights[cell];
        if sum <= 0.0 {
            return 0.0;
        }
        self.sumweights[cell].ln() - self.sumweightlogweights[cell] / self.sumweights[cell]
    }

    pub fn normalized_entropy(&self, x: usize, y: usize) -> f64 {
        let cell = self.cell_index(x, y);
        if self.sumsone[cell] <= 1 {
            return 0.0;
        }
        let e = self.entropy(cell);
        (e / self.starting_entropy).clamp(0.0, 1.0)
    }

    fn observe(&mut self, rng: &mut impl Rng) -> Option<usize> {
        let mut min_entropy = f64::MAX;
        let mut min_cell = None;

        for cell in 0..self.wave.len() {
            let count = self.sumsone[cell];
            if count == 0 {
                self.contradiction = true;
                return None;
            }
            if count == 1 {
                continue;
            }

            let entropy = self.entropy(cell) + rng.random::<f64>() * 1e-6;
            if entropy < min_entropy {
                min_entropy = entropy;
                min_cell = Some(cell);
            }
        }

        min_cell
    }

    fn collapse(&mut self, cell: usize, rng: &mut impl Rng) {
        let possible: Vec<usize> = (0..self.patterns.len())
            .filter(|&i| self.wave[cell][i])
            .collect();

        if possible.is_empty() {
            self.contradiction = true;
            return;
        }

        let total: f64 = possible.iter().map(|&i| self.weights[i]).sum();
        let mut r = rng.random::<f64>() * total;

        let chosen = possible
            .iter()
            .find(|&&p| {
                r -= self.weights[p];
                r <= 0.0
            })
            .copied()
            .unwrap_or(possible[0]);

        for p in 0..self.patterns.len() {
            if p != chosen && self.wave[cell][p] {
                self.ban(cell, p);
            }
        }
    }

    fn ban(&mut self, cell: usize, pattern: usize) {
        if !self.wave[cell][pattern] {
            return;
        }

        self.wave[cell][pattern] = false;
        self.sumsone[cell] -= 1;
        self.sumweights[cell] -= self.weights[pattern];
        self.sumweightlogweights[cell] -= self.weights[pattern] * self.log_weights[pattern];

        self.stack.push((cell, pattern));
    }

    fn propagate(&mut self) {
        let w = self.config.output_width;
        let h = self.config.output_height;

        while let Some((cell, pattern)) = self.stack.pop() {
            let x = cell % w;
            let y = cell / w;

            for dir in Direction::ALL {
                let nx = x as i32 + dir.dx();
                let ny = y as i32 + dir.dy();

                let (nx, ny) = if self.config.periodic_output {
                    (
                        nx.rem_euclid(w as i32) as usize,
                        ny.rem_euclid(h as i32) as usize,
                    )
                } else {
                    if nx < 0 || nx >= w as i32 || ny < 0 || ny >= h as i32 {
                        continue;
                    }
                    (nx as usize, ny as usize)
                };

                let neighbor = self.cell_index(nx, ny);

                let to_ban: Vec<usize> = self.propagator[pattern][dir as usize]
                    .iter()
                    .copied()
                    .filter(|&other| {
                        self.wave[neighbor][other]
                            && !self.wave[cell].iter().enumerate().any(|(p, &possible)| {
                                possible && self.propagator[p][dir as usize].contains(&other)
                            })
                    })
                    .collect();

                for other_pattern in to_ban {
                    self.ban(neighbor, other_pattern);
                    if self.sumsone[neighbor] == 0 {
                        self.contradiction = true;
                        return;
                    }
                }
            }
        }
    }

    pub fn step(&mut self) -> bool {
        if self.contradiction || self.done {
            return false;
        }

        let mut rng = rand::rng();

        match self.observe(&mut rng) {
            None => {
                if !self.contradiction {
                    self.done = true;
                }
                false
            }
            Some(cell) => {
                let x = cell % self.config.output_width;
                let y = cell / self.config.output_width;
                self.last_collapsed = Some((x, y));

                self.collapse(cell, &mut rng);
                self.propagate();
                true
            }
        }
    }

    pub fn run(&mut self) {
        while self.step() {}
    }

    pub fn is_collapsed(&self, x: usize, y: usize) -> bool {
        let cell = self.cell_index(x, y);
        self.sumsone[cell] == 1
    }

    pub fn get_color(&self, x: usize, y: usize) -> Color {
        let cell = self.cell_index(x, y);
        let possible: Vec<usize> = (0..self.patterns.len())
            .filter(|&i| self.wave[cell][i])
            .collect();

        match possible.len() {
            0 => [128, 0, 128],
            1 => self.patterns[possible[0]].get(0, 0),
            _ => {
                let (r, g, b, total) = possible.iter().fold((0.0, 0.0, 0.0, 0.0), |acc, &p| {
                    let w = self.weights[p];
                    let c = self.patterns[p].get(0, 0);
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

    pub fn render(&self) -> Vec<Color> {
        let mut output = Vec::with_capacity(self.config.output_width * self.config.output_height);
        for y in 0..self.config.output_height {
            for x in 0..self.config.output_width {
                output.push(self.get_color(x, y));
            }
        }
        output
    }
}

//! Wave Function Collapse - Overlapping Model
//!
//! An implementation of the overlapping model WFC algorithm that extracts
//! NxN patterns from a sample image and uses them to generate new outputs.

use std::collections::HashMap;

use rand::Rng;

/// RGB color type
pub type Color = [u8; 3];

/// A sample image represented as a 2D grid of colors
#[derive(Clone)]
pub struct Sample {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Color>,
}

impl Sample {
    pub fn new(width: usize, height: usize, pixels: Vec<Color>) -> Self {
        assert_eq!(pixels.len(), width * height);
        Self { width, height, pixels }
    }

    pub fn get(&self, x: usize, y: usize) -> Color {
        self.pixels[y * self.width + x]
    }

    /// Load a sample from an image file
    pub fn from_image(path: &std::path::Path) -> Result<Self, String> {
        let img = image::open(path).map_err(|e| e.to_string())?;
        let rgb = img.to_rgb8();
        let width = rgb.width() as usize;
        let height = rgb.height() as usize;
        let pixels: Vec<Color> = rgb.pixels().map(|p| [p[0], p[1], p[2]]).collect();
        Ok(Self { width, height, pixels })
    }

    /// Save sample to an image file
    pub fn save(&self, path: &std::path::Path) -> Result<(), String> {
        let mut img = image::RgbImage::new(self.width as u32, self.height as u32);
        for y in 0..self.height {
            for x in 0..self.width {
                let c = self.get(x, y);
                img.put_pixel(x as u32, y as u32, image::Rgb(c));
            }
        }
        img.save(path).map_err(|e| e.to_string())
    }
}

/// An NxN pattern extracted from the sample
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Pattern {
    pub size: usize,
    pub pixels: Vec<Color>,
}

impl Pattern {
    pub fn new(size: usize, pixels: Vec<Color>) -> Self {
        assert_eq!(pixels.len(), size * size);
        Self { size, pixels }
    }

    pub fn get(&self, x: usize, y: usize) -> Color {
        self.pixels[y * self.size + x]
    }

    /// Rotate pattern 90 degrees clockwise
    pub fn rotate(&self) -> Self {
        let n = self.size;
        let mut rotated = vec![[0u8; 3]; n * n];
        for y in 0..n {
            for x in 0..n {
                rotated[x * n + (n - 1 - y)] = self.get(x, y);
            }
        }
        Self::new(n, rotated)
    }

    /// Reflect pattern horizontally
    pub fn reflect(&self) -> Self {
        let n = self.size;
        let mut reflected = vec![[0u8; 3]; n * n];
        for y in 0..n {
            for x in 0..n {
                reflected[y * n + (n - 1 - x)] = self.get(x, y);
            }
        }
        Self::new(n, reflected)
    }

    /// Generate all symmetry variants (up to 8)
    pub fn symmetries(&self) -> Vec<Self> {
        let mut variants = Vec::with_capacity(8);
        let mut current = self.clone();

        for _ in 0..4 {
            if !variants.contains(&current) {
                variants.push(current.clone());
            }
            let reflected = current.reflect();
            if !variants.contains(&reflected) {
                variants.push(reflected);
            }
            current = current.rotate();
        }
        variants
    }
}

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
    /// All extracted patterns
    pub patterns: Vec<Pattern>,
    /// Frequency weight for each pattern
    pub weights: Vec<f64>,
    /// Propagator: for each (pattern, direction), which patterns are compatible
    propagator: Vec<Vec<Vec<usize>>>,
    /// Wave: for each cell, which patterns are still possible (bitset as Vec<bool>)
    wave: Vec<Vec<bool>>,
    /// Number of possible patterns remaining per cell
    sumsone: Vec<usize>,
    /// Sum of weights of possible patterns per cell
    sumweights: Vec<f64>,
    /// Sum of weight*log(weight) for entropy calculation
    sumweightlogweights: Vec<f64>,
    /// Precomputed log weights
    log_weights: Vec<f64>,
    /// Starting entropy (for normalization)
    starting_entropy: f64,
    /// Stack for propagation
    stack: Vec<(usize, usize)>,
    /// Whether we hit a contradiction
    pub contradiction: bool,
    /// Whether generation is complete
    pub done: bool,
    /// Last collapsed cell (for visualization)
    pub last_collapsed: Option<(usize, usize)>,
}

impl Wfc {
    /// Create a new WFC instance from a sample
    pub fn new(sample: &Sample, config: WfcConfig) -> Self {
        let (patterns, weights) = Self::extract_patterns(sample, &config);
        let propagator = Self::build_propagator(&patterns, &config);
        let num_patterns = patterns.len();

        let wave_size = config.output_width * config.output_height;
        let wave = vec![vec![true; num_patterns]; wave_size];

        let total_weight: f64 = weights.iter().sum();
        let log_weights: Vec<f64> = weights.iter().map(|&w| w.ln()).collect();
        let sumweightlogweight: f64 = weights.iter().zip(&log_weights).map(|(w, lw)| w * lw).sum();
        let starting_entropy = total_weight.ln() - sumweightlogweight / total_weight;

        Self {
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
        }
    }

    /// Extract patterns from sample
    fn extract_patterns(sample: &Sample, config: &WfcConfig) -> (Vec<Pattern>, Vec<f64>) {
        let n = config.pattern_size;
        let mut pattern_counts: HashMap<Pattern, usize> = HashMap::new();

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
                    pattern.symmetries()
                } else {
                    vec![pattern]
                };

                for variant in variants {
                    *pattern_counts.entry(variant).or_insert(0) += 1;
                }
            }
        }

        let mut patterns = Vec::new();
        let mut weights = Vec::new();
        for (pattern, count) in pattern_counts {
            patterns.push(pattern);
            weights.push(count as f64);
        }

        (patterns, weights)
    }

    /// Build the propagator (adjacency constraints)
    fn build_propagator(patterns: &[Pattern], config: &WfcConfig) -> Vec<Vec<Vec<usize>>> {
        let n = config.pattern_size;
        let num_patterns = patterns.len();

        // propagator[pattern_idx][direction] = vec of compatible pattern indices
        let mut propagator = vec![vec![Vec::new(); 4]; num_patterns];

        for (i, p1) in patterns.iter().enumerate() {
            for (j, p2) in patterns.iter().enumerate() {
                // Check right adjacency: p1's right edge matches p2's left edge
                if Self::patterns_agree(p1, p2, 1, 0, n) {
                    propagator[i][Direction::Right as usize].push(j);
                }
                // Check down adjacency
                if Self::patterns_agree(p1, p2, 0, 1, n) {
                    propagator[i][Direction::Down as usize].push(j);
                }
                // Check left adjacency
                if Self::patterns_agree(p1, p2, -1, 0, n) {
                    propagator[i][Direction::Left as usize].push(j);
                }
                // Check up adjacency
                if Self::patterns_agree(p1, p2, 0, -1, n) {
                    propagator[i][Direction::Up as usize].push(j);
                }
            }
        }

        propagator
    }

    /// Check if two patterns agree when p2 is offset from p1
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

    /// Reset to initial state
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
    }

    /// Get cell index from coordinates
    fn cell_index(&self, x: usize, y: usize) -> usize {
        y * self.config.output_width + x
    }

    /// Calculate entropy for a cell
    fn entropy(&self, cell: usize) -> f64 {
        let sum = self.sumweights[cell];
        if sum <= 0.0 {
            return 0.0;
        }
        self.sumweights[cell].ln() - self.sumweightlogweights[cell] / self.sumweights[cell]
    }

    /// Get entropy for visualization (normalized 0-1)
    pub fn normalized_entropy(&self, x: usize, y: usize) -> f64 {
        let cell = self.cell_index(x, y);
        if self.sumsone[cell] <= 1 {
            return 0.0;
        }
        let e = self.entropy(cell);
        (e / self.starting_entropy).clamp(0.0, 1.0)
    }

    /// Find cell with minimum entropy (that isn't collapsed)
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

            // Add small noise to break ties randomly
            let entropy = self.entropy(cell) + rng.random::<f64>() * 1e-6;
            if entropy < min_entropy {
                min_entropy = entropy;
                min_cell = Some(cell);
            }
        }

        min_cell
    }

    /// Collapse a cell to a single pattern
    fn collapse(&mut self, cell: usize, rng: &mut impl Rng) {
        let possible: Vec<usize> = self.wave[cell]
            .iter()
            .enumerate()
            .filter_map(|(i, &b)| if b { Some(i) } else { None })
            .collect();

        if possible.is_empty() {
            self.contradiction = true;
            return;
        }

        // Weighted random selection
        let total: f64 = possible.iter().map(|&i| self.weights[i]).sum();
        let mut r = rng.random::<f64>() * total;

        let mut chosen = possible[0];
        for &p in &possible {
            r -= self.weights[p];
            if r <= 0.0 {
                chosen = p;
                break;
            }
        }

        // Ban all other patterns
        for p in 0..self.patterns.len() {
            if p != chosen && self.wave[cell][p] {
                self.ban(cell, p);
            }
        }
    }

    /// Ban a pattern from a cell
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

    /// Propagate constraints after banning patterns
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
                    (nx.rem_euclid(w as i32) as usize, ny.rem_euclid(h as i32) as usize)
                } else {
                    if nx < 0 || nx >= w as i32 || ny < 0 || ny >= h as i32 {
                        continue;
                    }
                    (nx as usize, ny as usize)
                };

                let neighbor = self.cell_index(nx, ny);

                // Collect patterns to ban first to avoid borrow conflicts
                let to_ban: Vec<usize> = self.propagator[pattern][dir as usize]
                    .iter()
                    .copied()
                    .filter(|&other_pattern| {
                        if !self.wave[neighbor][other_pattern] {
                            return false;
                        }
                        // Check if any remaining pattern in current cell supports this
                        // If neighbor is to the RIGHT of cell, we need patterns in cell
                        // that can have other_pattern to their RIGHT
                        !self.wave[cell].iter().enumerate().any(|(p, &possible)| {
                            possible && self.propagator[p][dir as usize].contains(&other_pattern)
                        })
                    })
                    .collect();

                // Now ban them
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

    /// Perform one step: observe and propagate
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

    /// Run until done or contradiction
    pub fn run(&mut self) {
        while self.step() {}
    }

    /// Check if a cell is collapsed (single pattern)
    pub fn is_collapsed(&self, x: usize, y: usize) -> bool {
        let cell = self.cell_index(x, y);
        self.sumsone[cell] == 1
    }

    /// Get the color for a cell (average of possible patterns' top-left pixel)
    pub fn get_color(&self, x: usize, y: usize) -> Color {
        let cell = self.cell_index(x, y);
        let possible: Vec<usize> = self.wave[cell]
            .iter()
            .enumerate()
            .filter_map(|(i, &b)| if b { Some(i) } else { None })
            .collect();

        if possible.is_empty() {
            return [128, 0, 128]; // Magenta for contradiction
        }

        if possible.len() == 1 {
            return self.patterns[possible[0]].get(0, 0);
        }

        // Weighted average color
        let mut r = 0.0;
        let mut g = 0.0;
        let mut b = 0.0;
        let mut total = 0.0;

        for &p in &possible {
            let w = self.weights[p];
            let c = self.patterns[p].get(0, 0);
            r += c[0] as f64 * w;
            g += c[1] as f64 * w;
            b += c[2] as f64 * w;
            total += w;
        }

        [
            (r / total) as u8,
            (g / total) as u8,
            (b / total) as u8,
        ]
    }

    /// Render the current state to an image
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

/// Default pipe/circuit sample for testing
pub fn default_pipe_sample() -> Sample {
    // 8x8 pipe pattern
    // Colors: background (dark), pipe (light), junction (bright)
    let bg: Color = [32, 32, 48];
    let pipe: Color = [64, 128, 192];
    let junction: Color = [96, 192, 255];

    #[rustfmt::skip]
    let pixels = vec![
        bg, bg, bg, bg, bg, bg, bg, bg,
        bg, junction, pipe, pipe, pipe, junction, bg, bg,
        bg, pipe, bg, bg, bg, pipe, bg, bg,
        bg, pipe, bg, junction, pipe, junction, pipe, bg,
        bg, pipe, bg, pipe, bg, bg, pipe, bg,
        bg, junction, pipe, junction, bg, junction, pipe, bg,
        bg, bg, bg, pipe, bg, pipe, bg, bg,
        bg, bg, bg, junction, pipe, junction, bg, bg,
    ];

    Sample::new(8, 8, pixels)
}

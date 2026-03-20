use crate::boundary::Boundary;

#[derive(Clone, Debug)]
pub struct Config {
    /// N in NxN pattern extraction.
    pub pattern_size: usize,
    pub output_width: usize,
    pub output_height: usize,
    /// Wrap sample scanning around edges.
    pub periodic_input: bool,
    pub boundary: Boundary,
    /// Include rotation/reflection variants.
    pub symmetry: bool,
    /// Constrain top/bottom patterns to match sample edge positions.
    pub ground: bool,
    /// Constrain left/right patterns to match sample edge positions.
    pub sides: bool,
    /// RNG seed for deterministic output.
    pub seed: Option<u64>,
    /// Bias collapse toward patterns with more viable neighbors.
    pub use_flexibility: bool,
    pub backtracking: bool,
    pub max_backtracks: usize,
    /// Snapshot interval (in collapses) for backtracking.
    pub snapshot_interval: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pattern_size: 3,
            output_width: 32,
            output_height: 32,
            periodic_input: true,
            boundary: Boundary::Fixed,
            symmetry: true,
            ground: false,
            sides: false,
            seed: None,
            use_flexibility: true,
            backtracking: true,
            max_backtracks: 100,
            snapshot_interval: 10,
        }
    }
}

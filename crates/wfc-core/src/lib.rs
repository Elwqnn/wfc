//! Wave Function Collapse - Overlapping Model
//!
//! Implementation of the overlapping model WFC algorithm that extracts
//! NxN patterns from a sample image to generate new outputs.

pub(crate) mod backtrack;
pub(crate) mod bitset;
mod config;
mod error;
mod grid;
mod pattern;
pub(crate) mod rules;
mod sample;
pub(crate) mod solver;
pub(crate) mod state;

pub use config::Config;
pub use error::Error;
pub use grid::Direction;
pub use pattern::Pattern;
pub use rules::Rules;
pub use sample::{Sample, default_pipe_sample};
pub use solver::Wfc;
pub use state::State;

/// RGB color type
pub type Color = [u8; 3];

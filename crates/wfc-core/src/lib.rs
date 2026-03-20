//! Wave Function Collapse - overlapping model.

pub(crate) mod backtrack;
pub(crate) mod bitset;
mod boundary;
mod config;
mod constraint;
mod error;
mod grid;
mod pattern;
mod retry;
pub(crate) mod rules;
mod sample;
pub(crate) mod solver;
pub(crate) mod state;

pub use boundary::Boundary;
pub use config::Config;
pub use constraint::{CellConstraint, ConstraintContext};
pub use error::{Error, RunOutcome, StepOutcome};
pub use grid::Direction;
pub use pattern::Pattern;
pub use rules::Rules;
pub use sample::{Sample, default_pipe_sample};
pub use solver::Wfc;
pub use state::State;

#[cfg(feature = "parallel")]
pub use retry::parallel_solve;

pub type Color = [u8; 3];

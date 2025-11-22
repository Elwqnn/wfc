//! Wave Function Collapse - Overlapping Model
//!
//! Implementation of the overlapping model WFC algorithm that extracts
//! NxN patterns from a sample image to generate new outputs.

pub mod app;
mod pattern;
mod sample;
mod wfc;

pub use pattern::Pattern;
pub use sample::{Sample, default_pipe_sample};
pub use wfc::{Direction, Wfc, WfcConfig};

/// RGB color type
pub type Color = [u8; 3];

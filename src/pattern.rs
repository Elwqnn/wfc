use std::collections::HashSet;

use crate::Color;

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

    /// Generate all unique symmetry variants (up to 8)
    pub fn symmetries(&self) -> Vec<Self> {
        let mut seen = HashSet::new();
        let mut current = self.clone();

        for _ in 0..4 {
            seen.insert(current.clone());
            seen.insert(current.reflect());
            current = current.rotate();
        }
        seen.into_iter().collect()
    }
}

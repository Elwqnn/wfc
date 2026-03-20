use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

use crate::Color;

const MAX_INLINE: usize = 16;

/// NxN pattern stored inline (no heap for N<=4).
#[derive(Clone, Debug)]
pub struct Pattern {
    size: usize,
    len: usize,
    pixels: [Color; MAX_INLINE],
}

impl PartialEq for Pattern {
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size && self.pixels[..self.len] == other.pixels[..other.len]
    }
}

impl Eq for Pattern {}

impl Hash for Pattern {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.size.hash(state);
        self.pixels[..self.len].hash(state);
    }
}

impl PartialOrd for Pattern {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Pattern {
    fn cmp(&self, other: &Self) -> Ordering {
        self.size
            .cmp(&other.size)
            .then_with(|| self.pixels[..self.len].cmp(&other.pixels[..other.len]))
    }
}

impl Pattern {
    pub fn new(size: usize, pixels: Vec<Color>) -> Self {
        let len = size * size;
        assert_eq!(pixels.len(), len, "pixels length must be size*size");
        assert!(len <= MAX_INLINE, "pattern size > 4 not supported");
        let mut buf = [[0u8; 3]; MAX_INLINE];
        buf[..len].copy_from_slice(&pixels);
        Self {
            size,
            len,
            pixels: buf,
        }
    }

    pub(crate) fn from_buf(size: usize, pixels: [Color; MAX_INLINE], len: usize) -> Self {
        debug_assert_eq!(len, size * size);
        Self { size, len, pixels }
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Color {
        self.pixels[y * self.size + x]
    }

    /// Rotate 90 degrees clockwise.
    pub fn rotate(&self) -> Self {
        let n = self.size;
        let mut buf = [[0u8; 3]; MAX_INLINE];
        for y in 0..n {
            for x in 0..n {
                buf[x * n + (n - 1 - y)] = self.get(x, y);
            }
        }
        Self::from_buf(n, buf, self.len)
    }

    /// Reflect horizontally.
    pub fn reflect(&self) -> Self {
        let n = self.size;
        let mut buf = [[0u8; 3]; MAX_INLINE];
        for y in 0..n {
            for x in 0..n {
                buf[y * n + (n - 1 - x)] = self.get(x, y);
            }
        }
        Self::from_buf(n, buf, self.len)
    }

    /// All unique symmetry variants (up to 8), sorted.
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
        variants.sort();
        variants
    }
}

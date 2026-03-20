/// Per-cell pattern bitset. Unused bits in the last word are always
/// cleared, so iteration needs no bounds check.
#[derive(Clone)]
pub(crate) struct Bitset {
    bits: Vec<u64>,
    words_per_cell: usize,
}

impl Bitset {
    pub(crate) fn new(num_cells: usize, num_patterns: usize) -> Self {
        let words_per_cell = num_patterns.div_ceil(64);
        let total_words = num_cells * words_per_cell;
        let mut bits = vec![u64::MAX; total_words];

        // Clear unused bits in the last word of each cell
        let remainder = num_patterns % 64;
        if remainder != 0 {
            let mask = (1u64 << remainder) - 1;
            for cell in 0..num_cells {
                let last_word = cell * words_per_cell + (words_per_cell - 1);
                bits[last_word] = mask;
            }
        }

        Self {
            bits,
            words_per_cell,
        }
    }

    #[inline(always)]
    pub(crate) fn is_set(&self, cell: usize, pattern: usize) -> bool {
        let word_idx = cell * self.words_per_cell + pattern / 64;
        let bit_idx = pattern % 64;
        (self.bits[word_idx] >> bit_idx) & 1 != 0
    }

    #[inline(always)]
    pub(crate) fn clear(&mut self, cell: usize, pattern: usize) {
        let word_idx = cell * self.words_per_cell + pattern / 64;
        let bit_idx = pattern % 64;
        self.bits[word_idx] &= !(1u64 << bit_idx);
    }

    #[cfg(test)]
    pub(crate) fn count_ones(&self, cell: usize) -> usize {
        let base = cell * self.words_per_cell;
        let mut count = 0u32;
        for w in 0..self.words_per_cell {
            count += self.bits[base + w].count_ones();
        }
        count as usize
    }

    /// First set bit index (fast path for collapsed cells).
    #[inline(always)]
    pub(crate) fn first_set(&self, cell: usize) -> usize {
        let base = cell * self.words_per_cell;
        for w in 0..self.words_per_cell {
            let word = self.bits[base + w];
            if word != 0 {
                return w * 64 + word.trailing_zeros() as usize;
            }
        }
        0 // unreachable if any bit is set
    }

    /// Set bit indices for a cell, ascending.
    #[inline]
    pub(crate) fn iter_set(&self, cell: usize) -> BitsetIter<'_> {
        let base = cell * self.words_per_cell;
        BitsetIter {
            bits: &self.bits[base..base + self.words_per_cell],
            word_idx: 0,
            current: if self.words_per_cell > 0 {
                self.bits[base]
            } else {
                0
            },
            word_offset: 0,
        }
    }
}

pub(crate) struct BitsetIter<'a> {
    bits: &'a [u64],
    word_idx: usize,
    current: u64,
    word_offset: usize,
}

impl Iterator for BitsetIter<'_> {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<usize> {
        loop {
            if self.current != 0 {
                let tz = self.current.trailing_zeros() as usize;
                self.current &= self.current - 1; // clear lowest set bit
                return Some(self.word_offset + tz);
            }
            self.word_idx += 1;
            if self.word_idx >= self.bits.len() {
                return None;
            }
            self.word_offset = self.word_idx * 64;
            self.current = self.bits[self.word_idx];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_operations() {
        let mut bs = Bitset::new(2, 10);
        assert!(bs.is_set(0, 0));
        assert!(bs.is_set(0, 9));
        assert_eq!(bs.count_ones(0), 10);

        bs.clear(0, 3);
        assert!(!bs.is_set(0, 3));
        assert_eq!(bs.count_ones(0), 9);
        // Cell 1 unaffected
        assert_eq!(bs.count_ones(1), 10);
    }

    #[test]
    fn iter_ascending_order() {
        let mut bs = Bitset::new(1, 8);
        bs.clear(0, 1);
        bs.clear(0, 4);
        bs.clear(0, 6);
        let set: Vec<usize> = bs.iter_set(0).collect();
        assert_eq!(set, vec![0, 2, 3, 5, 7]);
    }

    #[test]
    fn multi_word() {
        let mut bs = Bitset::new(1, 100);
        assert_eq!(bs.count_ones(0), 100);
        bs.clear(0, 65);
        assert!(!bs.is_set(0, 65));
        assert_eq!(bs.count_ones(0), 99);

        let set: Vec<usize> = bs.iter_set(0).collect();
        assert_eq!(set.len(), 99);
        assert!(set.contains(&64));
        assert!(!set.contains(&65));
        assert!(set.contains(&66));
    }

    #[test]
    fn iter_does_not_yield_beyond_num_patterns() {
        // 10 patterns -> 1 word, bits 10..63 are cleared by new()
        let bs = Bitset::new(1, 10);
        let set: Vec<usize> = bs.iter_set(0).collect();
        assert_eq!(set, (0..10).collect::<Vec<_>>());
    }
}

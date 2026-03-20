use crate::boundary::Boundary;

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

    #[inline]
    pub fn opposite(self) -> Self {
        match self {
            Direction::Right => Direction::Left,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Up => Direction::Down,
        }
    }

    #[inline]
    pub(crate) fn dx(self) -> i32 {
        match self {
            Direction::Right => 1,
            Direction::Left => -1,
            _ => 0,
        }
    }

    #[inline]
    pub(crate) fn dy(self) -> i32 {
        match self {
            Direction::Down => 1,
            Direction::Up => -1,
            _ => 0,
        }
    }
}

const NO_NEIGHBOR: u32 = u32::MAX;

/// Precomputed neighbor lookup; avoids coordinate math on the propagation hot path.
pub(crate) struct Grid {
    pub(crate) width: usize,
    pub(crate) height: usize,
    /// `neighbors[cell * 4 + dir]`: neighbor cell index, or sentinel if out of bounds.
    neighbors: Vec<u32>,
}

impl Grid {
    pub(crate) fn new(width: usize, height: usize, boundary: Boundary) -> Self {
        let size = width * height;
        let mut neighbors = vec![NO_NEIGHBOR; size * 4];
        let wrap_x = boundary.wraps_x();
        let wrap_y = boundary.wraps_y();

        for cell in 0..size {
            let x = cell % width;
            let y = cell / width;

            for dir in Direction::ALL {
                let raw_x = x as i32 + dir.dx();
                let raw_y = y as i32 + dir.dy();

                let resolved_x = if raw_x >= 0 && raw_x < width as i32 {
                    Some(raw_x as usize)
                } else if wrap_x {
                    Some(raw_x.rem_euclid(width as i32) as usize)
                } else {
                    None
                };

                let resolved_y = if raw_y >= 0 && raw_y < height as i32 {
                    Some(raw_y as usize)
                } else if wrap_y {
                    Some(raw_y.rem_euclid(height as i32) as usize)
                } else {
                    None
                };

                if let (Some(nx), Some(ny)) = (resolved_x, resolved_y) {
                    neighbors[cell * 4 + dir as usize] = (ny * width + nx) as u32;
                }
            }
        }

        Self {
            width,
            height,
            neighbors,
        }
    }

    #[inline(always)]
    pub(crate) fn neighbor(&self, cell: usize, dir: usize) -> Option<usize> {
        let n = self.neighbors[cell * 4 + dir];
        if n == NO_NEIGHBOR {
            None
        } else {
            Some(n as usize)
        }
    }

    #[inline]
    pub(crate) fn cell(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    #[inline]
    pub(crate) fn coords(&self, cell: usize) -> (usize, usize) {
        (cell % self.width, cell / self.width)
    }

    #[inline]
    pub(crate) fn size(&self) -> usize {
        self.width * self.height
    }
}

/// Output grid edge behavior.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Boundary {
    /// No wrapping.
    #[default]
    Fixed,
    /// Wrap horizontally (left/right edges connect).
    PeriodicX,
    /// Wrap vertically (top/bottom edges connect).
    PeriodicY,
    /// Wrap both axes (toroidal topology).
    Periodic,
}

impl Boundary {
    #[inline]
    pub fn wraps_x(self) -> bool {
        matches!(self, Boundary::PeriodicX | Boundary::Periodic)
    }

    #[inline]
    pub fn wraps_y(self) -> bool {
        matches!(self, Boundary::PeriodicY | Boundary::Periodic)
    }
}

//! Dividing a geometry on an axis-aligned grid.
//!
//! [`DivideByGrid`] cuts a geometry into the cells of a [`GridSpec`], emitting
//! one piece per cell it touches. The cut preserves elevation, interpolating Z
//! at every new vertex, and preserves appearance, interpolating `Explicit` UV
//! with the same parameter so texture and geometry stay in step.
//!
//! Each emitted piece keeps the leaf kind it came from: a mesh divides into
//! meshes, a face into faces. Fan-out to individual faces is
//! [`Split`](crate::ops::Split)'s job, not this one's.
//!
//! `cell_size` is expressed in whatever units the geometry's frame uses. A grid
//! on an angular frame is a legitimate thing to build, so this op does not
//! second-guess units; a caller that wants to warn about it reads the frame
//! itself.

mod halfplane;
mod window;

#[cfg(test)]
mod tests;

pub(crate) use halfplane::Corner;
pub(crate) use window::{clip_to_window, faces_area_xy, signed_area_xy, Face, Window};

use crate::ops::UnsupportedOperation;
use crate::Geometry;

/// Relative tolerance for judging a cell full by area.
///
/// A single face's fullness is exact, because a cut vertex takes the cell's
/// coordinate verbatim. A mesh's is not: it rests on its faces' areas summing to
/// the cell's, so it needs a tolerance.
pub const COVERAGE_TOLERANCE: f64 = 1e-9;

/// The lattice a geometry is divided on, in that geometry's own frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GridSpec {
    origin: [f64; 2],
    cell_size: f64,
}

impl GridSpec {
    /// A grid anchored at `origin` with square cells of side `cell_size`.
    ///
    /// Cell `(0, 0)` has `origin` as its minimum corner.
    pub fn new(origin: [f64; 2], cell_size: f64) -> Result<Self, GridDivideError> {
        if !cell_size.is_finite() || cell_size <= 0.0 {
            return Err(GridDivideError::InvalidSpec(format!(
                "cell size must be positive and finite, got {cell_size}"
            )));
        }
        if !origin[0].is_finite() || !origin[1].is_finite() {
            return Err(GridDivideError::InvalidSpec(format!(
                "grid origin must be finite, got [{}, {}]",
                origin[0], origin[1]
            )));
        }
        Ok(Self { origin, cell_size })
    }

    /// The grid origin.
    pub fn origin(&self) -> [f64; 2] {
        self.origin
    }

    /// The cell side length.
    pub fn cell_size(&self) -> f64 {
        self.cell_size
    }

    /// The minimum and maximum corners of one cell.
    pub fn cell_bounds(&self, cell: GridCell) -> ([f64; 2], [f64; 2]) {
        let min = [
            self.origin[0] + cell.col as f64 * self.cell_size,
            self.origin[1] + cell.row as f64 * self.cell_size,
        ];
        let max = [min[0] + self.cell_size, min[1] + self.cell_size];
        (min, max)
    }

    /// The inclusive range of cells a box overlaps, as `(low, high)`.
    ///
    /// Indices are signed: an origin sitting inside or beyond the data yields
    /// negative rows and columns rather than clamping them away, which is the
    /// silent data loss the old implementation had.
    pub fn cell_range(&self, min: [f64; 2], max: [f64; 2]) -> (GridCell, GridCell) {
        let col_lo = ((min[0] - self.origin[0]) / self.cell_size).floor() as i64;
        let col_hi = ((max[0] - self.origin[0]) / self.cell_size).ceil() as i64 - 1;
        let row_lo = ((min[1] - self.origin[1]) / self.cell_size).floor() as i64;
        let row_hi = ((max[1] - self.origin[1]) / self.cell_size).ceil() as i64 - 1;
        (
            GridCell {
                row: row_lo,
                col: col_lo,
            },
            GridCell {
                row: row_hi.max(row_lo),
                col: col_hi.max(col_lo),
            },
        )
    }

    /// The number of cells the given box implies, saturating rather than
    /// overflowing. Callers use this to refuse an absurd cell size before doing
    /// the work.
    pub fn cell_count(&self, min: [f64; 2], max: [f64; 2]) -> u128 {
        let (lo, hi) = self.cell_range(min, max);
        let cols = (hi.col - lo.col + 1).max(0) as u128;
        let rows = (hi.row - lo.row + 1).max(0) as u128;
        cols.saturating_mul(rows)
    }

    pub(crate) fn window(&self, cell: GridCell) -> Window {
        let (min, max) = self.cell_bounds(cell);
        Window { min, max }
    }
}

/// Which cell of the lattice a piece came from.
///
/// Signed, because an explicit origin can sit anywhere relative to the data.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GridCell {
    pub row: i64,
    pub col: i64,
}

/// Whether an emitted piece fills its cell or is a fragment of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellCoverage {
    /// The piece is the whole cell.
    Full,
    /// The piece is part of the cell.
    Partial,
}

impl CellCoverage {
    /// Judge coverage by comparing a piece's area against its cell's.
    pub(crate) fn from_area(piece: f64, cell: f64) -> Self {
        if cell > 0.0 && piece >= cell * (1.0 - COVERAGE_TOLERANCE) {
            CellCoverage::Full
        } else {
            CellCoverage::Partial
        }
    }
}

/// Why a geometry could not be divided.
#[derive(Clone, Debug, PartialEq)]
pub enum GridDivideError {
    /// A leaf type with no areal extent to divide.
    Unsupported(UnsupportedOperation),
    /// No geometry, or nothing with area to divide.
    Empty,
    /// Members do not all lie in one coordinate frame, so one grid cannot
    /// describe them all.
    MixedFrames,
    /// The grid itself is not usable.
    InvalidSpec(String),
}

impl core::fmt::Display for GridDivideError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            GridDivideError::Unsupported(e) => e.fmt(f),
            GridDivideError::Empty => write!(f, "geometry has no area to divide"),
            GridDivideError::MixedFrames => {
                write!(f, "members do not all lie in one coordinate frame")
            }
            GridDivideError::InvalidSpec(why) => write!(f, "invalid grid: {why}"),
        }
    }
}

impl std::error::Error for GridDivideError {}

impl From<UnsupportedOperation> for GridDivideError {
    fn from(e: UnsupportedOperation) -> Self {
        GridDivideError::Unsupported(e)
    }
}

/// Cut a geometry into the cells of a grid.
#[enum_dispatch::enum_dispatch]
pub trait DivideByGrid {
    /// Divide on `grid`, invoking `emit` once per cell the geometry touches with
    /// that cell, whether the piece fills it, and the piece itself.
    ///
    /// Cells are emitted row-major and reproducibly. The emitted geometry keeps
    /// the leaf kind of the input.
    fn divide_by_grid(
        &self,
        grid: &GridSpec,
        emit: &mut dyn FnMut(GridCell, CellCoverage, Geometry),
    ) -> Result<(), GridDivideError> {
        let _ = (grid, emit);
        Err(GridDivideError::Unsupported(UnsupportedOperation {
            geometry: core::any::type_name::<Self>(),
            operation: "divide_by_grid",
        }))
    }
}

// The boxed enum variants need the trait on the `Box` itself: `enum_dispatch`
// forwards by UFCS, not auto-deref.
impl<T: DivideByGrid + ?Sized> DivideByGrid for Box<T> {
    fn divide_by_grid(
        &self,
        grid: &GridSpec,
        emit: &mut dyn FnMut(GridCell, CellCoverage, Geometry),
    ) -> Result<(), GridDivideError> {
        (**self).divide_by_grid(grid, emit)
    }
}

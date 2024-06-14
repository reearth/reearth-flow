use std::str::FromStr;

use crate::algorithm::{coordinate_position::CoordPos, dimensions::Dimensions};

#[derive(PartialEq, Eq, Clone)]
pub struct IntersectionMatrix(LocationArray<LocationArray<Dimensions>>);

#[derive(PartialEq, Eq, Clone, Copy)]
struct LocationArray<T>([T; 3]);

impl<T> LocationArray<T> {
    fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter()
    }
}

impl<T> std::ops::Index<CoordPos> for LocationArray<T> {
    type Output = T;

    fn index(&self, index: CoordPos) -> &Self::Output {
        match index {
            CoordPos::Inside => &self.0[0],
            CoordPos::OnBoundary => &self.0[1],
            CoordPos::Outside => &self.0[2],
        }
    }
}

impl<T> std::ops::IndexMut<CoordPos> for LocationArray<T> {
    fn index_mut(&mut self, index: CoordPos) -> &mut Self::Output {
        match index {
            CoordPos::Inside => &mut self.0[0],
            CoordPos::OnBoundary => &mut self.0[1],
            CoordPos::Outside => &mut self.0[2],
        }
    }
}

#[derive(Debug)]
pub struct InvalidInputError {
    message: String,
}

impl InvalidInputError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl std::error::Error for InvalidInputError {}
impl std::fmt::Display for InvalidInputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid input:  {}", self.message)
    }
}

impl std::fmt::Debug for IntersectionMatrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn char_for_dim(dim: &Dimensions) -> &'static str {
            match dim {
                Dimensions::Empty => "F",
                Dimensions::ZeroDimensional => "0",
                Dimensions::OneDimensional => "1",
                Dimensions::TwoDimensional => "2",
            }
        }
        let text = self
            .0
            .iter()
            .flat_map(|r| r.iter().map(char_for_dim))
            .collect::<Vec<&str>>()
            .join("");

        write!(f, "IntersectionMatrix({})", &text)
    }
}

impl IntersectionMatrix {
    pub fn empty() -> Self {
        IntersectionMatrix(LocationArray([LocationArray([Dimensions::Empty; 3]); 3]))
    }

    /// Set `dimensions` of the cell specified by the positions.
    ///
    /// `position_a`: which position `dimensions` applies to within the first geometry
    /// `position_b`: which position `dimensions` applies to within the second geometry
    /// `dimensions`: the dimension of the incident
    pub(crate) fn set(
        &mut self,
        position_a: CoordPos,
        position_b: CoordPos,
        dimensions: Dimensions,
    ) {
        self.0[position_a][position_b] = dimensions;
    }

    /// Reports an incident of `dimensions`, which updates the IntersectionMatrix if it's greater
    /// than what has been reported so far.
    ///
    /// `position_a`: which position `minimum_dimensions` applies to within the first geometry
    /// `position_b`: which position `minimum_dimensions` applies to within the second geometry
    /// `minimum_dimensions`: the dimension of the incident
    pub(crate) fn set_at_least(
        &mut self,
        position_a: CoordPos,
        position_b: CoordPos,
        minimum_dimensions: Dimensions,
    ) {
        if self.0[position_a][position_b] < minimum_dimensions {
            self.0[position_a][position_b] = minimum_dimensions;
        }
    }

    /// If both geometries have `Some` position, then changes the specified element to at
    /// least `minimum_dimensions`.
    ///
    /// Else, if either is none, do nothing.
    ///
    /// `position_a`: which position `minimum_dimensions` applies to within the first geometry, or
    ///               `None` if the dimension was not incident with the first geometry.
    /// `position_b`: which position `minimum_dimensions` applies to within the second geometry, or
    ///               `None` if the dimension was not incident with the second geometry.
    /// `minimum_dimensions`: the dimension of the incident
    pub(crate) fn set_at_least_if_in_both(
        &mut self,
        position_a: Option<CoordPos>,
        position_b: Option<CoordPos>,
        minimum_dimensions: Dimensions,
    ) {
        if let (Some(position_a), Some(position_b)) = (position_a, position_b) {
            self.set_at_least(position_a, position_b, minimum_dimensions);
        }
    }

    pub(crate) fn set_at_least_from_string(
        &mut self,
        dimensions: &str,
    ) -> Result<(), InvalidInputError> {
        if dimensions.len() != 9 {
            let message = format!("Expected dimensions length 9, found: {}", dimensions.len());
            return Err(InvalidInputError::new(message));
        }

        let mut chars = dimensions.chars();
        for a in &[CoordPos::Inside, CoordPos::OnBoundary, CoordPos::Outside] {
            for b in &[CoordPos::Inside, CoordPos::OnBoundary, CoordPos::Outside] {
                match chars.next().expect("already validated length is 9") {
                    '0' => self.0[*a][*b] = self.0[*a][*b].max(Dimensions::ZeroDimensional),
                    '1' => self.0[*a][*b] = self.0[*a][*b].max(Dimensions::OneDimensional),
                    '2' => self.0[*a][*b] = self.0[*a][*b].max(Dimensions::TwoDimensional),
                    'F' => {}
                    other => {
                        let message = format!("expected '0', '1', '2', or 'F'. Found: {other}");
                        return Err(InvalidInputError::new(message));
                    }
                }
            }
        }

        Ok(())
    }

    pub fn is_disjoint(&self) -> bool {
        self.0[CoordPos::Inside][CoordPos::Inside] == Dimensions::Empty
            && self.0[CoordPos::Inside][CoordPos::OnBoundary] == Dimensions::Empty
            && self.0[CoordPos::OnBoundary][CoordPos::Inside] == Dimensions::Empty
            && self.0[CoordPos::OnBoundary][CoordPos::OnBoundary] == Dimensions::Empty
    }

    pub fn is_intersects(&self) -> bool {
        !self.is_disjoint()
    }

    pub fn is_within(&self) -> bool {
        self.0[CoordPos::Inside][CoordPos::Inside] != Dimensions::Empty
            && self.0[CoordPos::Inside][CoordPos::Outside] == Dimensions::Empty
            && self.0[CoordPos::OnBoundary][CoordPos::Outside] == Dimensions::Empty
    }

    pub fn is_contains(&self) -> bool {
        self.0[CoordPos::Inside][CoordPos::Inside] != Dimensions::Empty
            && self.0[CoordPos::Outside][CoordPos::Inside] == Dimensions::Empty
            && self.0[CoordPos::Outside][CoordPos::OnBoundary] == Dimensions::Empty
    }

    pub fn is_equal_topo(&self) -> bool {
        self.0[CoordPos::Inside][CoordPos::Inside] != Dimensions::Empty
            && self.0[CoordPos::Inside][CoordPos::Outside] == Dimensions::Empty
            && self.0[CoordPos::Outside][CoordPos::Inside] == Dimensions::Empty
            && self.0[CoordPos::Outside][CoordPos::OnBoundary] == Dimensions::Empty
            && self.0[CoordPos::OnBoundary][CoordPos::Outside] == Dimensions::Empty
    }

    #[allow(clippy::nonminimal_bool)]
    pub fn is_coveredby(&self) -> bool {
        // [T*F**F***]
        self.0[CoordPos::Inside][CoordPos::Inside] != Dimensions::Empty
            && self.0[CoordPos::Inside][CoordPos::Outside] == Dimensions::Empty
            && self.0[CoordPos::OnBoundary][CoordPos::Outside] == Dimensions::Empty ||
        // [*TF**F***]
        self.0[CoordPos::Inside][CoordPos::OnBoundary] != Dimensions::Empty
            && self.0[CoordPos::Inside][CoordPos::Outside] == Dimensions::Empty
            && self.0[CoordPos::OnBoundary][CoordPos::Outside] == Dimensions::Empty ||
        // [**FT*F***]
        self.0[CoordPos::Inside][CoordPos::Outside] == Dimensions::Empty
            && self.0[CoordPos::OnBoundary][CoordPos::Inside] != Dimensions::Empty
            && self.0[CoordPos::OnBoundary][CoordPos::Outside] == Dimensions::Empty ||
        // [**F*TF***]
        self.0[CoordPos::Inside][CoordPos::Outside] == Dimensions::Empty
            && self.0[CoordPos::OnBoundary][CoordPos::OnBoundary] != Dimensions::Empty
            && self.0[CoordPos::OnBoundary][CoordPos::Outside] == Dimensions::Empty
    }

    #[allow(clippy::nonminimal_bool)]
    pub fn is_covers(&self) -> bool {
        // [T*****FF*]
        self.0[CoordPos::Inside][CoordPos::Inside] != Dimensions::Empty
        && self.0[CoordPos::Outside][CoordPos::Inside] == Dimensions::Empty
        && self.0[CoordPos::Outside][CoordPos::OnBoundary] == Dimensions::Empty ||
        // [*T****FF*]
        self.0[CoordPos::Inside][CoordPos::OnBoundary] != Dimensions::Empty
        && self.0[CoordPos::Outside][CoordPos::Inside] == Dimensions::Empty
        && self.0[CoordPos::Outside][CoordPos::OnBoundary] == Dimensions::Empty ||
        // [***T**FF*]
        self.0[CoordPos::OnBoundary][CoordPos::Inside] != Dimensions::Empty
        && self.0[CoordPos::Outside][CoordPos::Inside] == Dimensions::Empty
        && self.0[CoordPos::Outside][CoordPos::OnBoundary] == Dimensions::Empty ||
        // [****T*FF*]
        self.0[CoordPos::OnBoundary][CoordPos::OnBoundary] != Dimensions::Empty
        && self.0[CoordPos::Outside][CoordPos::Inside] == Dimensions::Empty
        && self.0[CoordPos::Outside][CoordPos::OnBoundary] == Dimensions::Empty
    }

    #[allow(clippy::nonminimal_bool)]
    pub fn is_touches(&self) -> bool {
        // [FT*******]
        self.0[CoordPos::Inside][CoordPos::Inside] == Dimensions::Empty
        && self.0[CoordPos::Inside][CoordPos::OnBoundary] != Dimensions::Empty ||
        // [F**T*****]
        self.0[CoordPos::Inside][CoordPos::Inside] == Dimensions::Empty
        && self.0[CoordPos::OnBoundary][CoordPos::Inside] != Dimensions::Empty ||
        // [F***T****]
        self.0[CoordPos::Inside][CoordPos::Inside] == Dimensions::Empty
        && self.0[CoordPos::OnBoundary][CoordPos::OnBoundary] != Dimensions::Empty
    }

    pub fn is_crosses(&self) -> bool {
        let dims_a = self.0[CoordPos::Inside][CoordPos::Inside]
            .max(self.0[CoordPos::Inside][CoordPos::OnBoundary])
            .max(self.0[CoordPos::Inside][CoordPos::Outside]);

        let dims_b = self.0[CoordPos::Inside][CoordPos::Inside]
            .max(self.0[CoordPos::OnBoundary][CoordPos::Inside])
            .max(self.0[CoordPos::Outside][CoordPos::Inside]);
        match (dims_a, dims_b) {
            // a < b
            _ if dims_a < dims_b =>
            // [T*T******]
            {
                self.0[CoordPos::Inside][CoordPos::Inside] != Dimensions::Empty
                    && self.0[CoordPos::Inside][CoordPos::Outside] != Dimensions::Empty
            }
            // a > b
            _ if dims_a > dims_b =>
            // [T*****T**]
            {
                self.0[CoordPos::Inside][CoordPos::Inside] != Dimensions::Empty
                    && self.0[CoordPos::Outside][CoordPos::Inside] != Dimensions::Empty
            }
            // a == b, only line / line permitted
            (Dimensions::OneDimensional, Dimensions::OneDimensional) =>
            // [0********]
            {
                self.0[CoordPos::Inside][CoordPos::Inside] == Dimensions::ZeroDimensional
            }
            _ => false,
        }
    }

    #[allow(clippy::nonminimal_bool)]
    pub fn is_overlaps(&self) -> bool {
        // dimensions must be non-empty, equal, and line / line is a special case
        let dims_a = self.0[CoordPos::Inside][CoordPos::Inside]
            .max(self.0[CoordPos::Inside][CoordPos::OnBoundary])
            .max(self.0[CoordPos::Inside][CoordPos::Outside]);

        let dims_b = self.0[CoordPos::Inside][CoordPos::Inside]
            .max(self.0[CoordPos::OnBoundary][CoordPos::Inside])
            .max(self.0[CoordPos::Outside][CoordPos::Inside]);
        match (dims_a, dims_b) {
            // line / line: [1*T***T**]
            (Dimensions::OneDimensional, Dimensions::OneDimensional) => {
                self.0[CoordPos::Inside][CoordPos::Inside] == Dimensions::OneDimensional
                    && self.0[CoordPos::Inside][CoordPos::Outside] != Dimensions::Empty
                    && self.0[CoordPos::Outside][CoordPos::Inside] != Dimensions::Empty
            }
            // point / point or polygon / polygon: [T*T***T**]
            (Dimensions::ZeroDimensional, Dimensions::ZeroDimensional)
            | (Dimensions::TwoDimensional, Dimensions::TwoDimensional) => {
                self.0[CoordPos::Inside][CoordPos::Inside] != Dimensions::Empty
                    && self.0[CoordPos::Inside][CoordPos::Outside] != Dimensions::Empty
                    && self.0[CoordPos::Outside][CoordPos::Inside] != Dimensions::Empty
            }
            _ => false,
        }
    }

    pub fn get(&self, lhs: CoordPos, rhs: CoordPos) -> Dimensions {
        self.0[lhs][rhs]
    }

    pub fn matches(&self, spec: &str) -> Result<bool, InvalidInputError> {
        if spec.len() != 9 {
            return Err(InvalidInputError::new(format!(
                "DE-9IM specification must be exactly 9 characters. Got {len}",
                len = spec.len()
            )));
        }

        let mut chars = spec.chars();
        for a in &[CoordPos::Inside, CoordPos::OnBoundary, CoordPos::Outside] {
            for b in &[CoordPos::Inside, CoordPos::OnBoundary, CoordPos::Outside] {
                let dim_spec = dimension_matcher::DimensionMatcher::try_from(
                    chars.next().expect("already validated length is 9"),
                )?;
                if !dim_spec.matches(self.0[*a][*b]) {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }
}

pub(crate) mod dimension_matcher {
    use super::Dimensions;
    use super::InvalidInputError;

    /// A single letter from a DE-9IM matching specification like "1*T**FFF*"
    pub(crate) enum DimensionMatcher {
        Anything,
        NonEmpty,
        Exact(Dimensions),
    }

    impl DimensionMatcher {
        pub fn matches(&self, dim: Dimensions) -> bool {
            match (self, dim) {
                (Self::Anything, _) => true,
                (DimensionMatcher::NonEmpty, d) => d != Dimensions::Empty,
                (DimensionMatcher::Exact(a), b) => a == &b,
            }
        }
    }

    impl TryFrom<char> for DimensionMatcher {
        type Error = InvalidInputError;

        fn try_from(value: char) -> Result<Self, Self::Error> {
            Ok(match value {
                '*' => Self::Anything,
                't' | 'T' => Self::NonEmpty,
                'f' | 'F' => Self::Exact(Dimensions::Empty),
                '0' => Self::Exact(Dimensions::ZeroDimensional),
                '1' => Self::Exact(Dimensions::OneDimensional),
                '2' => Self::Exact(Dimensions::TwoDimensional),
                _ => {
                    return Err(InvalidInputError::new(format!(
                        "invalid DE-9IM specification character: {value}"
                    )))
                }
            })
        }
    }
}

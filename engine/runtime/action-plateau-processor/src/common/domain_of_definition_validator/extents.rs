//! L06: the coordinates of each member lie within the `gml:Envelope` of the
//! city model.

use std::cell::RefCell;
use std::collections::BTreeSet;

use fastxml::transform::{EditableNode, Transformer};
use reearth_flow_types::{AttributeValue, Feature};
use serde_json::Number;

use super::report::MemberReporter;
use crate::common::errors::{PlateauProcessorError, Result};

/// An invalid coordinate value found in the file.
#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum CoordinateError {
    #[error("The gml:Envelope has a NaN or infinite coordinate.")]
    NonFiniteEnvelope,
    #[error("A geometry has a NaN or infinite coordinate.")]
    NonFiniteGeometry,
    #[error("A geometry has no coordinate on some axis.")]
    MissingAxis,
}

/// The messages of `errors` without repeats, as a note; `None` when there are
/// no errors.
fn note(errors: impl IntoIterator<Item = CoordinateError>) -> Option<AttributeValue> {
    let errors = errors.into_iter().collect::<BTreeSet<_>>();
    (!errors.is_empty())
        .then(|| AttributeValue::String(errors.iter().map(ToString::to_string).collect()))
}

/// The `gml:Envelope` in `gml:boundedBy` of the city model.
#[derive(Debug, Default)]
pub(super) struct BoundedBy {
    pub(super) srs_name: String,
    /// `None` when the corners are not all finite, in which case L06 checks no
    /// geometry.
    pub(super) envelope: Option<Envelope>,
}

/// The `gml:Envelope` of the city model, finite on every axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Envelope(Bounds);

impl Envelope {
    /// `None` when a coordinate is NaN or infinite.
    fn new(lower: [f64; 3], upper: [f64; 3]) -> Option<Self> {
        let finite = lower.iter().chain(&upper).all(|v| v.is_finite());
        finite.then(|| {
            Self(Bounds([0, 1, 2].map(|i| {
                Ok(AxisRange {
                    min: lower[i],
                    max: upper[i],
                })
            })))
        })
    }

    /// Whether `extent` lies within this envelope, boundaries included. An
    /// extent with an axis of unknown range does not.
    fn contains(&self, extent: &Extent) -> bool {
        self.0
            .axes()
            .iter()
            .zip(extent.0.axes())
            .all(|(envelope, extent)| {
                matches!(
                    (envelope, extent),
                    (Ok(envelope), Ok(extent))
                        if envelope.min <= extent.min && extent.max <= envelope.max
                )
            })
    }
}

/// The minimum and maximum coordinates of a feature, per axis.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Extent(Bounds);

impl Extent {
    /// The extent of the positions given as a flat list of (x, y, z).
    fn from_positions(positions: &[f64]) -> Self {
        Self(Bounds([0, 1, 2].map(|i| {
            positions
                .iter()
                .skip(i)
                .step_by(3)
                .try_fold(None, |range: Option<AxisRange>, &v| {
                    if !v.is_finite() {
                        return Err(CoordinateError::NonFiniteGeometry);
                    }
                    Ok(Some(match range {
                        None => AxisRange { min: v, max: v },
                        Some(r) => AxisRange {
                            min: r.min.min(v),
                            max: r.max.max(v),
                        },
                    }))
                })?
                .ok_or(CoordinateError::MissingAxis)
        })))
    }

    /// The reasons of the axes whose range is unknown.
    fn errors(&self) -> impl Iterator<Item = CoordinateError> + '_ {
        self.0.axes().iter().filter_map(|axis| axis.err())
    }
}

/// The finite minimum and maximum of one axis.
#[derive(Debug, Clone, Copy, PartialEq)]
struct AxisRange {
    min: f64,
    max: f64,
}

/// The ranges of (latitude, longitude, elevation), the common form of an
/// [`Envelope`] and an [`Extent`]. An axis whose range is unknown holds the
/// reason.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Bounds([std::result::Result<AxisRange, CoordinateError>; 3]);

impl Bounds {
    /// The ranges of (latitude, longitude, elevation).
    fn axes(&self) -> &[std::result::Result<AxisRange, CoordinateError>; 3] {
        &self.0
    }

    /// Inserts the axes as `{prefix}Latitude` / `Longitude` / `Elevation`, with
    /// nulls for an axis of unknown range or when there are no bounds.
    fn insert_into(
        bounds: Option<&Self>,
        feature: &mut Feature,
        lower_prefix: &str,
        upper_prefix: &str,
    ) {
        let mut insert = |prefix: &str, end: fn(&AxisRange) -> f64| {
            for (i, axis) in ["Latitude", "Longitude", "Elevation"].iter().enumerate() {
                let value = bounds
                    .and_then(|b| b.axes()[i].as_ref().ok())
                    .and_then(|range| Number::from_f64(end(range)))
                    .map_or(AttributeValue::Null, AttributeValue::Number);
                feature.insert(format!("{prefix}{axis}"), value);
            }
        };
        insert(lower_prefix, |range| range.min);
        insert(upper_prefix, |range| range.max);
    }
}

/// Parses a whitespace-separated list of numbers such as `gml:posList`.
fn parse_coordinates(content: &str) -> Result<Vec<f64>> {
    content
        .split_whitespace()
        .map(|v| {
            v.parse()
                .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))
        })
        .collect()
}

fn parse_envelope_from_node(node: &EditableNode) -> Result<BoundedBy> {
    let srs_name = node.get_attribute("srsName").unwrap_or_default();

    let mut lower_corner_value = None;
    let mut upper_corner_value = None;
    for child in node.children() {
        match child.name().as_str() {
            "lowerCorner" => lower_corner_value = child.get_content(),
            "upperCorner" => upper_corner_value = child.get_content(),
            _ => {}
        }
    }

    let parse_corner = |value: Option<String>, name: &str| {
        let value = value.ok_or_else(|| {
            PlateauProcessorError::DomainOfDefinitionValidator(format!(
                "Failed to get {name} corner node"
            ))
        })?;
        match parse_coordinates(&value)?[..] {
            [x, y, z, ..] => Ok([x, y, z]),
            _ => Err(PlateauProcessorError::DomainOfDefinitionValidator(format!(
                "Failed to parse {name} corner"
            ))),
        }
    };
    let lower = parse_corner(lower_corner_value, "lower")?;
    let upper = parse_corner(upper_corner_value, "upper")?;

    Ok(BoundedBy {
        srs_name,
        envelope: Envelope::new(lower, upper),
    })
}

/// The first element of `raw_xml` matching `xpath`, read as a `gml:Envelope`.
pub(super) fn stream_extract_envelope(raw_xml: &str, xpath: &str) -> Result<BoundedBy> {
    let envelope = RefCell::new(None);
    let error = RefCell::new(None);

    let transformer = Transformer::from(raw_xml)
        .with_root_namespaces()
        .map_err(|e| {
            PlateauProcessorError::DomainOfDefinitionValidator(format!(
                "Failed to parse root namespaces: {e:?}"
            ))
        })?;

    transformer
        .on(xpath, |node| {
            if envelope.borrow().is_some() || error.borrow().is_some() {
                return;
            }
            match parse_envelope_from_node(node) {
                Ok(value) => {
                    *envelope.borrow_mut() = Some(value);
                }
                Err(err) => {
                    *error.borrow_mut() = Some(err);
                }
            }
        })
        .for_each()
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;

    if let Some(err) = error.into_inner() {
        return Err(err);
    }

    envelope.into_inner().ok_or_else(|| {
        PlateauProcessorError::DomainOfDefinitionValidator(
            "Failed to get envelope element".to_string(),
        )
    })
}

/// The L06 check of a file.
#[derive(Debug, Default)]
pub(super) struct ExtentsCheck {
    /// `None` until the envelope is read, or when it is not finite.
    envelope: Option<Envelope>,
    correct: u32,
    incorrect: u32,
    /// Reported in the summary as a note.
    coordinate_errors: BTreeSet<CoordinateError>,
}

impl ExtentsCheck {
    /// A check against `envelope`. A file without a finite envelope is one
    /// error, as no geometry can be checked.
    pub(super) fn new(envelope: Option<Envelope>) -> Self {
        let mut check = Self {
            envelope,
            ..Self::default()
        };
        if envelope.is_none() {
            check.incorrect += 1;
            check
                .coordinate_errors
                .insert(CoordinateError::NonFiniteEnvelope);
        }
        check
    }

    /// Checks the contents of the `gml:pos` / `gml:posList` of a member.
    pub(super) fn check_member(
        &mut self,
        pos_contents: &[String],
        report: &MemberReporter,
    ) -> Result<()> {
        let Some(envelope) = self.envelope else {
            return Ok(());
        };
        let mut positions = Vec::<f64>::new();
        for content in pos_contents {
            positions.extend(parse_coordinates(content)?);
        }
        if positions.is_empty() {
            return Ok(());
        }
        let extent = Extent::from_positions(&positions);
        if envelope.contains(&extent) {
            self.correct += 1;
            return Ok(());
        }
        self.incorrect += 1;
        report.send_with("ExtentsValidation", |feature| {
            feature.insert("isExtentsCorrect", AttributeValue::Bool(false));
            Bounds::insert_into(Some(&envelope.0), feature, "lower", "upper");
            Bounds::insert_into(Some(&extent.0), feature, "min", "max");
            if let Some(note) = note(extent.errors()) {
                feature.insert("miscellaneous", note);
            }
        });
        Ok(())
    }

    pub(super) fn write_summary(&self, summary: &mut Feature) {
        summary.insert(
            "correctExtents",
            AttributeValue::Number(Number::from(self.correct)),
        );
        summary.insert(
            "incorrectExtents",
            AttributeValue::Number(Number::from(self.incorrect)),
        );
        Bounds::insert_into(
            self.envelope.as_ref().map(|e| &e.0),
            summary,
            "lower",
            "upper",
        );
        if let Some(note) = note(self.coordinate_errors.iter().copied()) {
            summary.insert("miscellaneous", note);
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_bounds_reject_non_finite_coordinates() {
        for v in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(
                Envelope::new([v, 0.0, 0.0], [1.0, 1.0, 1.0]).is_none(),
                "lower corner with {v}"
            );
            assert!(
                Envelope::new([0.0, 0.0, 0.0], [1.0, 1.0, v]).is_none(),
                "upper corner with {v}"
            );
            // Only the elevation, which has the value, is unknown.
            assert_eq!(
                Extent::from_positions(&[0.0, 0.0, 0.0, 1.0, 1.0, v]),
                Extent(Bounds([
                    range(0.0, 1.0),
                    range(0.0, 1.0),
                    Err(CoordinateError::NonFiniteGeometry)
                ])),
                "position with {v}"
            );
        }
        assert_eq!(
            Extent::from_positions(&[0.0, 0.0]),
            Extent(Bounds([
                range(0.0, 0.0),
                range(0.0, 0.0),
                Err(CoordinateError::MissingAxis)
            ]))
        );
    }

    #[test]
    fn test_extent_from_positions_and_envelope_contains() {
        let extent = Extent::from_positions(&[1.0, 5.0, 3.0, 4.0, 2.0, 6.0]);
        assert_eq!(
            extent,
            Extent(Bounds([range(1.0, 4.0), range(2.0, 5.0), range(3.0, 6.0)]))
        );

        let envelope = Envelope::new([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]).unwrap();
        assert!(envelope.contains(&extent));
        let outside = Extent::from_positions(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.5]);
        assert!(!envelope.contains(&outside));
        let unknown = Extent::from_positions(&[1.0, 2.0, 3.0, 4.0, 5.0, f64::NAN]);
        assert!(!envelope.contains(&unknown));
    }

    fn range(min: f64, max: f64) -> std::result::Result<AxisRange, CoordinateError> {
        Ok(AxisRange { min, max })
    }
}

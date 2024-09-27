use super::{CoordNode, EdgeEnd};
use crate::algorithm::utils::lex_cmp;
use crate::algorithm::GeoFloat;
use crate::types::coordinate::Coordinate;

use std::collections::BTreeMap;
use std::fmt;
use std::marker::PhantomData;

/// A map of nodes, indexed by the coordinate of the node
pub(crate) struct NodeMap<T, Z, NF>
where
    T: GeoFloat,
    Z: GeoFloat,
    NF: NodeFactory<T, Z>,
{
    map: BTreeMap<NodeKey<T, Z>, NF::Node>,
    _node_factory: PhantomData<NF>,
}

/// Creates the node stored in `NodeMap`
pub(crate) trait NodeFactory<T: GeoFloat, Z: GeoFloat> {
    type Node;
    fn create_node(coordinate: Coordinate<T, Z>) -> Self::Node;
}

impl<T, Z, NF> fmt::Debug for NodeMap<T, Z, NF>
where
    T: GeoFloat,
    Z: GeoFloat,
    NF: NodeFactory<T, Z>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeMap")
            .field("map.len()", &self.map.len())
            .finish()
    }
}

#[derive(Clone)]
struct NodeKey<T: GeoFloat, Z: GeoFloat>(Coordinate<T, Z>);

impl<T: GeoFloat, Z: GeoFloat> std::cmp::Ord for NodeKey<T, Z> {
    fn cmp(&self, other: &NodeKey<T, Z>) -> std::cmp::Ordering {
        debug_assert!(!self.0.x.is_nan());
        debug_assert!(!self.0.y.is_nan());
        debug_assert!(!other.0.x.is_nan());
        debug_assert!(!other.0.y.is_nan());
        lex_cmp(&self.0, &other.0)
    }
}

impl<T: GeoFloat, Z: GeoFloat> std::cmp::PartialOrd for NodeKey<T, Z> {
    fn partial_cmp(&self, other: &NodeKey<T, Z>) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: GeoFloat, Z: GeoFloat> std::cmp::PartialEq for NodeKey<T, Z> {
    fn eq(&self, other: &NodeKey<T, Z>) -> bool {
        debug_assert!(!self.0.x.is_nan());
        debug_assert!(!self.0.y.is_nan());
        debug_assert!(!other.0.x.is_nan());
        debug_assert!(!other.0.y.is_nan());
        self.0 == other.0
    }
}

impl<T: GeoFloat, Z: GeoFloat> std::cmp::Eq for NodeKey<T, Z> {}

impl<T, Z, NF> NodeMap<T, Z, NF>
where
    T: GeoFloat,
    Z: GeoFloat,
    NF: NodeFactory<T, Z>,
{
    pub fn new() -> Self {
        NodeMap {
            map: BTreeMap::new(),
            _node_factory: PhantomData,
        }
    }
    /// Adds a `NF::Node` with the given `Coord`.
    ///
    /// Note: Coords must be non-NaN.
    pub fn insert_node_with_coordinate(&mut self, coord: Coordinate<T, Z>) -> &mut NF::Node {
        debug_assert!(
            !coord.x.is_nan() && !coord.y.is_nan(),
            "NaN coordinates are not supported"
        );
        let key = NodeKey(coord);
        self.map
            .entry(key)
            .or_insert_with(|| NF::create_node(coord))
    }

    /// returns the `NF::Node`, if any, matching `coord`
    pub fn find(&self, coord: Coordinate<T, Z>) -> Option<&NF::Node> {
        self.map.get(&NodeKey(coord))
    }

    /// Iterates across `NF::Node`s in lexical order of their `Coord`
    pub fn iter(&self) -> impl Iterator<Item = &NF::Node> {
        self.map.values()
    }

    /// Iterates across `NF::Node`s in lexical order of their `Coord`
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut NF::Node> {
        self.map.values_mut()
    }

    /// Iterates across `NF::Node`s in lexical order of their `Coord`
    pub fn into_iter(self) -> impl Iterator<Item = NF::Node> {
        self.map.into_values()
    }
}

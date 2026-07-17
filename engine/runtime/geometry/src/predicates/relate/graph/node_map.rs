use std::collections::BTreeMap;
use std::fmt;
use std::marker::PhantomData;

/// Lexicographic comparison of two coordinates (x first, then y). The
/// coordinates must be non-NaN, the invariant of every graph coordinate.
pub(crate) fn lex_cmp(a: &[f64; 2], b: &[f64; 2]) -> std::cmp::Ordering {
    a.partial_cmp(b).expect("graph coordinates must not be NaN")
}

/// A map of nodes, indexed by the coordinate of the node
pub(crate) struct NodeMap<NF>
where
    NF: NodeFactory,
{
    map: BTreeMap<NodeKey, NF::Node>,
    _node_factory: PhantomData<NF>,
}

/// Creates the node stored in `NodeMap`
pub(crate) trait NodeFactory {
    type Node;
    fn create_node(coordinate: [f64; 2]) -> Self::Node;
}

impl<NF> fmt::Debug for NodeMap<NF>
where
    NF: NodeFactory,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeMap")
            .field("map.len()", &self.map.len())
            .finish()
    }
}

#[derive(Clone)]
struct NodeKey([f64; 2]);

impl std::cmp::Ord for NodeKey {
    fn cmp(&self, other: &NodeKey) -> std::cmp::Ordering {
        debug_assert!(!self.0[0].is_nan());
        debug_assert!(!self.0[1].is_nan());
        debug_assert!(!other.0[0].is_nan());
        debug_assert!(!other.0[1].is_nan());
        lex_cmp(&self.0, &other.0)
    }
}

impl std::cmp::PartialOrd for NodeKey {
    fn partial_cmp(&self, other: &NodeKey) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::PartialEq for NodeKey {
    fn eq(&self, other: &NodeKey) -> bool {
        debug_assert!(!self.0[0].is_nan());
        debug_assert!(!self.0[1].is_nan());
        debug_assert!(!other.0[0].is_nan());
        debug_assert!(!other.0[1].is_nan());
        self.0 == other.0
    }
}

impl std::cmp::Eq for NodeKey {}

impl<NF> NodeMap<NF>
where
    NF: NodeFactory,
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
    pub fn insert_node_with_coordinate(&mut self, coord: [f64; 2]) -> &mut NF::Node {
        debug_assert!(
            !coord[0].is_nan() && !coord[1].is_nan(),
            "NaN coordinates are not supported"
        );
        let key = NodeKey(coord);
        self.map
            .entry(key)
            .or_insert_with(|| NF::create_node(coord))
    }

    /// returns the `NF::Node`, if any, matching `coord`
    pub fn find(&self, coord: [f64; 2]) -> Option<&NF::Node> {
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

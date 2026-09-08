/// A label on the relation between two elements of a [`UnionFind`].
///
/// Implementors must form a commutative group in which every value is its own
/// inverse, so that a label composed with itself is [`IDENTITY`](Self::IDENTITY).
/// `()` records no relation at all and gives a plain union-find; [`Parity`]
/// records a same/opposite bit.
pub trait Relation: Copy + Eq {
    const IDENTITY: Self;

    fn combine(self, other: Self) -> Self;
}

impl Relation for () {
    const IDENTITY: Self = ();

    fn combine(self, _other: Self) -> Self {}
}

/// Whether two elements agree or disagree, for constraints that flip.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parity {
    Same,
    Opposite,
}

impl Relation for Parity {
    const IDENTITY: Self = Self::Same;

    fn combine(self, other: Self) -> Self {
        if self == other {
            Self::Same
        } else {
            Self::Opposite
        }
    }
}

/// Disjoint sets over `0..nodes`, with union by rank and path compression.
///
/// Each element carries a [`Relation`] to its set representative, so a
/// constraint between two elements can be recorded and contradictions
/// detected. `UnionFind<()>` carries none and is a plain union-find; use
/// [`root`](Self::root) and [`merge`](Self::merge) with it.
#[derive(Debug)]
pub struct UnionFind<L = ()> {
    parent: Vec<usize>,
    /// Relation of each element to its parent.
    label: Vec<L>,
    rank: Vec<u32>,
}

impl<L: Relation> UnionFind<L> {
    pub fn new(nodes: usize) -> Self {
        Self {
            parent: (0..nodes).collect(),
            label: vec![L::IDENTITY; nodes],
            rank: vec![0; nodes],
        }
    }

    /// The set representative of `node` and `node`'s relation to it,
    /// compressing the path on the way out.
    pub fn find(&mut self, node: usize) -> (usize, L) {
        let mut root = node;
        let mut relation = L::IDENTITY;
        while self.parent[root] != root {
            relation = relation.combine(self.label[root]);
            root = self.parent[root];
        }
        let mut current = node;
        let mut current_relation = relation;
        while self.parent[current] != root {
            let next = self.parent[current];
            let next_relation = current_relation.combine(self.label[current]);
            self.parent[current] = root;
            self.label[current] = current_relation;
            current = next;
            current_relation = next_relation;
        }
        (root, relation)
    }

    /// Record that `a` and `b` stand in `relation`, merging their sets. Returns
    /// `false` if that contradicts what the sets already say, in which case
    /// nothing is merged.
    pub fn union(&mut self, a: usize, b: usize, relation: L) -> bool {
        let (root_a, relation_a) = self.find(a);
        let (root_b, relation_b) = self.find(b);
        if root_a == root_b {
            return relation_a.combine(relation_b) == relation;
        }
        let merged = relation_a.combine(relation).combine(relation_b);
        match self.rank[root_a].cmp(&self.rank[root_b]) {
            std::cmp::Ordering::Less => {
                self.parent[root_a] = root_b;
                self.label[root_a] = merged;
            }
            std::cmp::Ordering::Greater => {
                self.parent[root_b] = root_a;
                self.label[root_b] = merged;
            }
            std::cmp::Ordering::Equal => {
                self.parent[root_b] = root_a;
                self.label[root_b] = merged;
                self.rank[root_a] += 1;
            }
        }
        true
    }
}

impl UnionFind<()> {
    /// The set representative of `node`.
    pub fn root(&mut self, node: usize) -> usize {
        self.find(node).0
    }

    /// Merge the sets of `a` and `b`.
    pub fn merge(&mut self, a: usize, b: usize) {
        self.union(a, b, ());
    }
}

use super::{Active, ActiveSet};
use std::{
    collections::BTreeSet,
    ops::{RangeFrom, RangeTo},
};

/// A simple ordered set implementation backed by a `Vec`.

pub struct VecSet<T: Ord>(BTreeSet<T>);

impl<T: Ord> Default for VecSet<T> {
    fn default() -> Self {
        Self(BTreeSet::new())
    }
}

impl<T: Ord> VecSet<T> {
    pub fn get(&self, segment: &T) -> Option<&T> {
        self.0.get(segment)
    }

    pub fn get_next(&self, segment: &T) -> Option<&T> {
        let mut range = self.0.range(segment..);
        range.next();
        return range.next();
    }

    pub fn get_prev(&self, segment: &T) -> Option<&T> {
        let mut range = self.0.range(..segment);
        range.next_back();
        return range.next_back();
    }

    pub fn insert(&mut self, segment: T) {
        self.0.insert(segment);
    }

    pub fn remove(&mut self, segment: &T) {
        self.0.remove(segment);
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter()
    }
}

impl<T: Ord> ActiveSet for VecSet<Active<T>> {
    type Seg = T;

    fn previous_find<F: FnMut(&Active<Self::Seg>) -> bool>(
        &self,
        segment: &Self::Seg,
        mut f: F,
    ) -> Option<&Active<Self::Seg>> {
        //let segment = Active::active_ref(segment);
        // let ub = match self.data.binary_search(segment) {
        //     Ok(i) => i,
        //     Err(i) => i,
        // };
        // self.data[..ub].iter().rev().find(|s| f(s))
        self.0
            .range::<Self::Seg, RangeTo<&Self::Seg>>(..segment)
            .next_back()
            .filter(|s| f(s))
    }

    fn next_find<F: FnMut(&Active<Self::Seg>) -> bool>(
        &self,
        segment: &Self::Seg,
        mut f: F,
    ) -> Option<&Active<Self::Seg>> {
        // let segment = Active::active_ref(segment);
        // let start = match self.data.binary_search(segment) {
        //     Ok(i) => i + 1,
        //     Err(i) => i,
        // };
        // self.data[start..].iter().find(|s| f(s))
        self.0
            .range::<Self::Seg, RangeFrom<&Self::Seg>>(segment..)
            .next()
            .filter(|s| f(s))
    }

    fn insert_active(&mut self, segment: Self::Seg) {
        self.0.insert(Active(segment));
    }

    fn remove_active(&mut self, segment: &Self::Seg) {
        let segment = Active::active_ref(segment);
        self.0.remove(segment);
    }
}

// #[derive(Debug, Clone)]
// pub struct VecSet<T: Ord> {
//     data: Vec<T>,
// }

// impl<T: Ord> Default for VecSet<T> {
//     fn default() -> Self {
//         Self {
//             data: Default::default(),
//         }
//     }
// }

// impl<T: PartialOrd + Debug> VecSet<Active<T>> {
//     #[allow(unused)]
//     pub fn partition_point<P>(&self, mut pred: P) -> usize
//     where
//         P: FnMut(&T) -> bool,
//     {
//         self.data.partition_point(|s| pred(&s.0))
//     }

//     pub fn index_of(&self, segment: &T) -> usize {
//         self.data
//             .binary_search(Active::active_ref(segment))
//             .expect("segment not found in active-vec-set")
//     }

//     pub fn index_not_of(&self, segment: &T) -> usize {
//         self.data
//             .binary_search(Active::active_ref(segment))
//             .expect_err("segment already found in active-vec-set")
//     }
//     pub fn len(&self) -> usize {
//         self.data.len()
//     }

//     pub fn insert_at(&mut self, idx: usize, segment: T) {
//         self.data.insert(idx, Active(segment))
//     }

//     pub fn remove_at(&mut self, idx: usize) -> T {
//         self.data.remove(idx).0
//     }

//     #[allow(unused)]
//     pub fn check_swap(&mut self, idx: usize) -> bool {
//         if self.data[idx].cmp(&self.data[idx + 1]) == Ordering::Greater {
//             self.data.swap(idx, idx + 1);
//             true
//         } else {
//             false
//         }
//     }
// }

// impl<T: Ord> Index<usize> for VecSet<T> {
//     type Output = T;

//     fn index(&self, index: usize) -> &Self::Output {
//         &self.data[index]
//     }
// }

// impl<T: PartialOrd + Debug> ActiveSet for VecSet<Active<T>> {
//     type Seg = T;

//     fn previous_find<F: FnMut(&Active<Self::Seg>) -> bool>(
//         &self,
//         segment: &Self::Seg,
//         mut f: F,
//     ) -> Option<&Active<Self::Seg>> {
//         let segment = Active::active_ref(segment);
//         let ub = match self.data.binary_search(segment) {
//             Ok(i) => i,
//             Err(i) => i,
//         };
//         self.data[..ub].iter().rev().find(|s| f(s))
//     }

//     fn next_find<F: FnMut(&Active<Self::Seg>) -> bool>(
//         &self,
//         segment: &Self::Seg,
//         mut f: F,
//     ) -> Option<&Active<Self::Seg>> {
//         let segment = Active::active_ref(segment);
//         let start = match self.data.binary_search(segment) {
//             Ok(i) => i + 1,
//             Err(i) => i,
//         };
//         self.data[start..].iter().find(|s| f(s))
//     }

//     fn insert_active(&mut self, segment: Self::Seg) {
//         let idx = {
//             let segment = Active::active_ref(&segment);
//             self.data
//                 .binary_search(segment)
//                 .expect_err("element already in active-vec-set")
//         };

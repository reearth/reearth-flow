use std::{borrow::Borrow, cmp::Ordering, fmt::Debug, ops::Deref};

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(transparent)]
pub(crate) struct Active<T>(pub(crate) T);

impl<T> Active<T> {
    pub(crate) fn active_ref(t: &T) -> &Active<T> {
        unsafe { std::mem::transmute(t) }
    }
}

impl<T> Borrow<T> for Active<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for Active<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Assert total equality.
impl<T: PartialEq> Eq for Active<T> {}

impl<T: Ord> PartialOrd for Active<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Assert total ordering of active segments.
impl<T: Ord> Ord for Active<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        // if let Some(c) = T::partial_cmp(self, other) {
        //     c
        // } else {
        //     panic!("unable to compare active segments!");
        // }
        self.0.cmp(&other.0)
    }
}

// impl<T: PartialOrd + Debug> PartialOrd for Active<T> {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         Some(self.cmp(other))
//     }
// }

#[allow(unused)]
pub trait ActiveSet: Default {
    type Seg;
    fn previous_find<F: FnMut(&Active<Self::Seg>) -> bool>(
        &self,
        segment: &Self::Seg,
        f: F,
    ) -> Option<&Active<Self::Seg>>;
    fn previous(&self, segment: &Self::Seg) -> Option<&Active<Self::Seg>> {
        self.previous_find(segment, |_| true)
    }
    fn next_find<F: FnMut(&Active<Self::Seg>) -> bool>(
        &self,
        segment: &Self::Seg,
        f: F,
    ) -> Option<&Active<Self::Seg>>;
    fn next(&self, segment: &Self::Seg) -> Option<&Active<Self::Seg>> {
        self.next_find(segment, |_| true)
    }
    fn insert_active(&mut self, segment: Self::Seg);
    fn remove_active(&mut self, segment: &Self::Seg);
}

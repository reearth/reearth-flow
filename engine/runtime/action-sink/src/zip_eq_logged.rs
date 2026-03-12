use tracing::error;

pub trait ZipEqLoggedExt: Iterator + Sized {
    /// Zips two iterators. If one ends before the other, an error is logged
    /// instead of panicking.
    fn zip_eq_logged<I>(self, other: I) -> ZipEqLogged<Self, I::IntoIter>
    where
        I: IntoIterator;
}

impl<T: Iterator> ZipEqLoggedExt for T {
    fn zip_eq_logged<I>(self, other: I) -> ZipEqLogged<Self, I::IntoIter>
    where
        I: IntoIterator,
    {
        ZipEqLogged {
            iter_a: self,
            iter_b: other.into_iter(),
        }
    }
}

pub struct ZipEqLogged<A, B> {
    iter_a: A,
    iter_b: B,
}

impl<A, B> Iterator for ZipEqLogged<A, B>
where
    A: Iterator,
    B: Iterator,
{
    type Item = (A::Item, B::Item);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match (self.iter_a.next(), self.iter_b.next()) {
            (Some(a), Some(b)) => Some((a, b)),
            (None, None) => None,
            (Some(_), None) => {
                error!("ZipEqLogged mismatch: Left iterator was longer than right.");
                None
            }
            (None, Some(_)) => {
                error!("ZipEqLogged mismatch: Right iterator was longer than left.");
                None
            }
        }
    }

    // Hinting for optimization
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (a_low, a_high) = self.iter_a.size_hint();
        let (b_low, b_high) = self.iter_b.size_hint();

        let low = std::cmp::min(a_low, b_low);
        let high = match (a_high, b_high) {
            (Some(ah), Some(bh)) => Some(std::cmp::min(ah, bh)),
            _ => None,
        };
        (low, high)
    }
}

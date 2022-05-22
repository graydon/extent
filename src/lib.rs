//! This is an alternative to `std::ops::{Range,RangeInclusive}`, avoiding the
//! quirks of those types (non-`Copy`, inability to produce empty inclusive
//! ranges without extra bool, directly implementing `Iterator`, etc). See
//! https://ridiculousfish.com/blog/posts/least-favorite-rust-type.html and
//! https://kaylynn.gay/blog/post/rust_ranges_and_suffering for its litany of
//! sins.
//!
//! I was coincidentally writing this type in one of my own crates today when I
//! saw the second post go by, so I figured I'd split it out and post it as a
//! crate others can use. It has some quirks and limitations of its own but it
//! suits my needs better than the builtins. Here are the choices I made:
//!
//!   1. `Extent` represents an inclusive range of numbers, where number is
//!      `N:PrimInt` from the (fairly standard) num-traits crate. It is
//!      inclusive because (at least the most obvious representations of)
//!      exclusive ranges can't represent the maximum number of a number-type,
//!      which in my experience one fairly often needs to represent!
//!
//!   2. `Extent` uses exactly 2 numbers and no extra flags or wasted space.
//!
//!   3. `Extent` can represent empty ranges. Empty ranges are represented with
//!      a normalized form of `{lo=1, hi=0}`. This is the only case for which
//!      `lo > hi` and is only constructable via the static function `empty` or
//!      the IO-oriented function `new_unchecked`. Typical accessors for
//!      endpoints `lo()` and `hi()` return an `Option<N>` with `None` in the
//!      empty case. If you want the raw form (eg. for doing IO) you can call
//!      `lo_unchecked()` or `hi_unchecked()`, which are marked unsafe as they
//!      do not reflect the significant `lo <= hi` invariant.
//!
//!   4. All nonempty cases have `lo <= hi` enforced in `new`. If you pass `hi >
//!      lo` to `new`, the values are swapped (i.e. you can construct from
//!      either order of points; they get stored in order). If you are
//!      constructing from raw IO values you can do `new_unchecked` which will
//!      not swap, only normalize unordered ranges to `empty()`, and is also
//!      unsafe.
//!
//!   5. `Extent` implements `Copy` (and everything else standard).
//!
//!   6. `Extent` does not implement `Iterator`, but it has an `iter` method
//!      that copies `Extent` into `ExtentIter`, which does implement
//!      `Iterator`.
//!
//!   7. There is also an `ExtentRevIter` that counts down.
//!
//!   8. Some basic set-like operators are provided (union, intersection,
//!      contains) but nothing too fancy.
//!
//! Patches are welcome to enrich this further, though I will try to keep it
//! fairly simple and focused on the use-case of number-ranges, not "arbitrary
//! thing ranges".

use std::{
    borrow::Borrow,
    ops::{Range, RangeInclusive},
};

use num_traits::PrimInt;

#[derive(Copy, Clone, Debug, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Extent<N: PrimInt> {
    lo: N,
    hi: N,
}

impl<N: PrimInt> Default for Extent<N> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<N: PrimInt> Extent<N> {
    pub fn lo(&self) -> Option<N> {
        if self.is_empty() {
            None
        } else {
            Some(self.lo)
        }
    }

    pub fn hi(&self) -> Option<N> {
        if self.is_empty() {
            None
        } else {
            Some(self.hi)
        }
    }

    pub unsafe fn lo_unchecked(&self) -> N {
        self.lo
    }

    pub unsafe fn hi_unchecked(&self) -> N {
        self.hi
    }

    pub fn len(&self) -> N {
        if self.is_empty() {
            N::zero()
        } else {
            N::one() + (self.hi - self.lo)
        }
    }

    pub fn empty() -> Self {
        Self {
            lo: N::one(),
            hi: N::zero(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.lo > self.hi
    }

    pub fn new<T: Borrow<N>, U: Borrow<N>>(lo: T, hi: U) -> Self {
        let lo: N = *lo.borrow();
        let hi: N = *hi.borrow();
        Self {
            lo: lo.min(hi),
            hi: hi.max(lo),
        }
    }

    pub unsafe fn new_unchecked<T: Borrow<N>, U: Borrow<N>>(lo: T, hi: U) -> Self {
        let lo: N = *lo.borrow();
        let hi: N = *hi.borrow();
        if lo > hi {
            Self::empty()
        } else {
            Self { lo, hi }
        }
    }

    pub fn union<S: Borrow<Self>>(&self, other: S) -> Self {
        if self.is_empty() {
            *other.borrow()
        } else if other.borrow().is_empty() {
            self.clone()
        } else {
            let other = *other.borrow();
            Self::new(self.lo.min(other.lo), self.hi.max(other.hi))
        }
    }

    pub fn intersect<S: Borrow<Self>>(&self, other: S) -> Self {
        if self.is_empty() || other.borrow().is_empty() {
            Extent::empty()
        } else {
            let other = *other.borrow();
            Self::new(&self.lo.max(other.lo), &self.hi.min(other.hi))
        }
    }

    pub fn contains<T: Borrow<N>>(&self, n: T) -> bool {
        let n = *n.borrow();
        self.lo <= n && n <= self.hi
    }

    pub fn iter(&self) -> ExtentIter<N> {
        ExtentIter(*self)
    }
}

#[derive(Clone, Debug, Default)]
pub struct ExtentIter<N: PrimInt>(Extent<N>);

impl<N: PrimInt> Iterator for ExtentIter<N> {
    type Item = N;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            None
        } else {
            let v = self.0.lo;
            self.0.lo = self.0.lo + N::one();
            Some(v)
        }
    }
}

impl<N: PrimInt> ExtentIter<N> {
    pub fn rev(self) -> ExtentRevIter<N> {
        ExtentRevIter(self.0)
    }
}

pub struct ExtentRevIter<N: PrimInt>(Extent<N>);

impl<N: PrimInt> Iterator for ExtentRevIter<N> {
    type Item = N;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            None
        } else {
            let v = self.0.hi;
            self.0.hi = self.0.hi - N::one();
            Some(v)
        }
    }
}

// std::ops::Range is an exclusive range. Extent is inclusive,
// so we subtract one from any nonempty std::ops::Range.
impl<N: PrimInt> From<Range<N>> for Extent<N> {
    fn from(r: Range<N>) -> Self {
        if r.is_empty() {
            Self::empty()
        } else {
            Self {
                lo: r.start,
                hi: r.end - N::one(),
            }
        }
    }
}

impl<N: PrimInt> TryFrom<Extent<N>> for Range<N> {
    type Error = &'static str;
    fn try_from(e: Extent<N>) -> Result<Self, Self::Error> {
        if e.is_empty() {
            Ok(Range {
                start: N::zero(),
                end: N::zero(),
            })
        } else if e.hi == N::max_value() {
            Err("Extent.hi is N::max_value(), can't represent as Range")
        } else {
            Ok(Range {
                start: e.lo,
                end: e.hi + N::one(),
            })
        }
    }
}

impl<N: PrimInt> From<RangeInclusive<N>> for Extent<N> {
    fn from(r: RangeInclusive<N>) -> Self {
        if r.is_empty() {
            Self::empty()
        } else {
            Self::new(r.start(), r.end())
        }
    }
}

impl<N: PrimInt> From<Extent<N>> for RangeInclusive<N> {
    fn from(e: Extent<N>) -> Self {
        RangeInclusive::new(e.lo, e.hi)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use core::convert::TryInto;
    use num_traits::PrimInt;
    use std::fmt::Debug;

    fn check_sensible<N: PrimInt + Debug>(a: N, b: N) {
        let e = Extent::new(a, b);
        assert!(e.contains(a));
        assert!(e.contains(b));
        assert!(e.lo() <= e.hi());
        let ri: RangeInclusive<N> = e.clone().into();
        let e2: Extent<N> = ri.into();
        assert_eq!(e, e2);
        match e.try_into() {
            Ok(r) => {
                let r: Range<N> = r;
                let e3: Extent<N> = r.into();
                assert_eq!(e, e3);
            }
            Err(_) => {
                assert_eq!(e.hi, N::max_value())
            }
        }
    }

    fn check_set_ops<N: PrimInt + Debug>(a: N, b: N, c: N) {
        let mut v = [a, b, c];
        v.sort();
        let a = v[0];
        let b = v[1];
        let c = v[2];
        let ab = Extent::from(a..=b);
        let bc = Extent::from(b..=c);
        let ac = Extent::from(a..=c);
        let bb = Extent::from(b..=b);
        let empty: Extent<N> = Extent::empty();
        assert_eq!(ab.union(bc), ac);
        assert_eq!(ab.union(ac), ac);
        assert_eq!(bc.union(ac), ac);
        assert_eq!(ac.union(ab), ac);
        assert_eq!(ac.union(bc), ac);

        assert_eq!(ab.union(empty), ab);
        assert_eq!(empty.union(ab), ab);

        assert_eq!(ab.intersect(bc), bb);
        assert_eq!(ab.intersect(ac), ab);
        assert_eq!(bc.intersect(ac), bc);
        assert_eq!(bb.intersect(ac), bb);
        assert_eq!(bb.intersect(ab), bb);
        assert_eq!(bb.intersect(bc), bb);

        assert_eq!(ab.intersect(empty), empty);
        assert_eq!(empty.intersect(ab), empty);
    }

    #[test]
    fn test_basics() {
        let elts = vec![
            i32::MIN,
            i32::MIN + 1,
            i32::MIN + 2,
            -2,
            -1,
            0,
            1,
            2,
            i32::MAX - 2,
            i32::MAX - 1,
            i32::MAX,
        ];
        for a in elts.iter() {
            for b in elts.iter() {
                check_sensible(*a, *b);
                for c in elts.iter() {
                    check_set_ops(*a, *b, *c);
                }
            }
        }

        let v: Vec<_> = Extent::from(0..=5).iter().collect();
        assert_eq!(v, vec![0, 1, 2, 3, 4, 5]);

        let rv: Vec<_> = Extent::from(0..=5).iter().rev().collect();
        assert_eq!(rv, vec![5, 4, 3, 2, 1, 0]);

        let ev: Vec<u32> = Extent::empty().iter().collect();
        assert_eq!(ev, vec![]);
    }
}

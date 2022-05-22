# extent

This is an alternative to `std::ops::{Range,RangeInclusive}`, avoiding the
quirks of those types (non-`Copy`, inability to produce empty inclusive
ranges without extra bool, directly implementing `Iterator`, etc). See
https://ridiculousfish.com/blog/posts/least-favorite-rust-type.html and
https://kaylynn.gay/blog/post/rust_ranges_and_suffering for its litany of
sins.

I was coincidentally writing this type in one of my own crates today when I
saw the second post go by, so I figured I'd split it out and post it as a
crate others can use. It has some quirks and limitations of its own but it
suits my needs better than the builtins. Here are the choices I made:

  1. `Extent` represents an inclusive range of numbers, where number is
     `N:PrimInt` from the (fairly standard) num-traits crate. It is
     inclusive because (at least the most obvious representations of)
     exclusive ranges can't represent the maximum number of a number-type,
     which in my experience one fairly often needs to represent!

  2. `Extent` uses exactly 2 numbers and no extra flags or wasted space.

  3. `Extent` can represent empty ranges. Empty ranges are represented with
     a normalized form of `{lo=1, hi=0}`. This is the only case for which
     `lo > hi` and is only constructable via the static function `empty` or
     the IO-oriented function `new_unchecked`. Typical accessors for
     endpoints `lo()` and `hi()` return an `Option<N>` with `None` in the
     empty case. If you want the raw form (eg. for doing IO) you can call
     `lo_unchecked()` or `hi_unchecked()`, which are marked unsafe as they
     do not reflect the significant `lo <= hi` invariant.

  4. All nonempty cases have `lo <= hi` enforced in `new`. If you pass `lo >
     hi` to `new`, the values are swapped (i.e. you can construct from
     either order of points; they get stored in order). If you are
     constructing from raw IO values you can do `new_unchecked` which will
     not swap, only normalize unordered ranges to `empty()`, and is also
     unsafe.

  5. `Extent` implements `Copy` (and everything else standard).

  6. `Extent` does not implement `Iterator`, but it has an `iter` method
     that copies `Extent` into `ExtentIter`, which does implement
     `Iterator`.

  7. There is also an `ExtentRevIter` that counts down.

  8. Some basic set-like operators are provided (union, intersection,
     contains) but nothing too fancy.

Patches are welcome to enrich this further, though I will try to keep it
fairly simple and focused on the use-case of number-ranges, not "arbitrary
thing ranges".

License: MIT OR Apache-2.0

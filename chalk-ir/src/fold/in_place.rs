//! Subroutines to help implementers of `Fold` avoid unnecessary heap allocations.

use std::marker::PhantomData;
use std::{mem, ptr};

fn is_zst<T>() -> bool {
    mem::size_of::<T>() == 0
}

fn is_layout_identical<T, U>() -> bool {
    mem::size_of::<T>() == mem::size_of::<U>() && mem::align_of::<T>() == mem::align_of::<U>()
}

/// Maps a `Box<T>` to a `Box<U>`, reusing the underlying storage if possible.
pub(super) fn fallible_map_box<T, U, E>(
    b: Box<T>,
    map: impl FnOnce(T) -> Result<U, E>,
) -> Result<Box<U>, E> {
    // This optimization is only valid when `T` and `U` have the same size/alignment and is not
    // useful for ZSTs.
    if !is_layout_identical::<T, U>() || is_zst::<T>() {
        return map(*b).map(Box::new);
    }

    let raw = Box::into_raw(b);
    unsafe {
        let val = ptr::read(raw);

        // Box<T> -> Box<MaybeUninit<U>>
        let mut raw: Box<mem::MaybeUninit<U>> = Box::from_raw(raw.cast());

        // If `map` panics or returns an error, `raw` will free the memory associated with `b`, but
        // not drop the boxed value itself since it is wrapped in `MaybeUninit`. This is what we
        // want since the boxed value was moved into `map`.
        let mapped_val = map(val)?;
        ptr::write(raw.as_mut_ptr(), mapped_val);

        // Box<MaybeUninit<U>> -> Box<U>
        Ok(Box::from_raw(Box::into_raw(raw).cast()))
    }
}

/// Maps a `Vec<T>` to a `Vec<U>`, reusing the underlying storage if possible.
pub(super) fn fallible_map_vec<T, U, E>(
    vec: Vec<T>,
    mut map: impl FnMut(T) -> Result<U, E>,
) -> Result<Vec<U>, E> {
    // This optimization is only valid when `T` and `U` have the same size/alignment and is not
    // useful for ZSTs.
    if !is_layout_identical::<T, U>() || is_zst::<T>() {
        return vec.into_iter().map(map).collect();
    }

    let mut vec = VecMappedInPlace::<T, U>::new(vec);

    unsafe {
        for i in 0..vec.len {
            let place = vec.ptr.add(i);
            let val = ptr::read(place);

            // Set `map_in_progress` so the drop impl for `VecMappedInPlace` can handle the other
            // elements correctly in case `map` panics or returns an error.
            vec.map_in_progress = i;
            let mapped_val = map(val)?;

            ptr::write(place as *mut U, mapped_val);
        }

        Ok(vec.finish())
    }
}

/// Takes ownership of a `Vec` that is being mapped in place.
struct VecMappedInPlace<T, U> {
    ptr: *mut T,
    len: usize,
    cap: usize,

    map_in_progress: usize,
    _elem_tys: PhantomData<(T, U)>,
}

impl<T, U> VecMappedInPlace<T, U> {
    fn new(mut vec: Vec<T>) -> Self {
        assert!(is_layout_identical::<T, U>());

        // FIXME: This is just `Vec::into_raw_parts`. Use that instead when it is stabilized.
        let ptr = vec.as_mut_ptr();
        let len = vec.len();
        let cap = vec.capacity();
        mem::forget(vec);

        VecMappedInPlace {
            ptr,
            len,
            cap,

            map_in_progress: 0,
            _elem_tys: PhantomData,
        }
    }

    unsafe fn finish(self) -> Vec<U> {
        let this = mem::ManuallyDrop::new(self);
        Vec::from_raw_parts(this.ptr as *mut U, this.len, this.cap)
    }
}

/// `VecMappedInPlace` drops everything but the element that was passed to `map` when it panicked or
/// returned an error. Everything before that index in the vector has type `U` (it has been mapped)
/// and everything after it has type `T` (it has not been mapped).
///
/// ```text
///  mapped
///  |      not yet mapped
///  |----| |-----|
/// [UUUU UxTT TTTT]
///        ^
///    `map_in_progress` (not dropped)
/// ```
impl<T, U> Drop for VecMappedInPlace<T, U> {
    fn drop(&mut self) {
        // Drop mapped elements (of type `U`)
        for i in 0..self.map_in_progress {
            unsafe {
                ptr::drop_in_place(self.ptr.add(i) as *mut U);
            }
        }

        // Drop unmapped elements (of type `T`)
        for i in (self.map_in_progress + 1)..self.len {
            unsafe {
                ptr::drop_in_place(self.ptr.add(i));
            }
        }

        // Free the underlying storage for the `Vec`.
        // `len` is 0 because the elements were handled above.
        unsafe {
            Vec::from_raw_parts(self.ptr, 0, self.cap);
        }
    }
}

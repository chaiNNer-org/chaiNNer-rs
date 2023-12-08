use std::mem::ManuallyDrop;

/// Splits the slice into a slice of `N`-element arrays,
/// starting at the beginning of the slice,
/// and a remainder slice with length strictly less than `N`.
///
/// # Panics
///
/// Panics if `N` is 0. This check will most probably get changed to a compile time
/// error before this method gets stabilized.
#[inline]
pub fn slice_as_chunks<T, const N: usize>(s: &[T]) -> (&[[T; N]], &[T]) {
    assert_ne!(N, 0);
    let len = s.len() / N;
    let (multiple_of_n, remainder) = s.split_at(len * N);
    // SAFETY: We already panicked for zero, and ensured by construction
    // that the length of the subslice is a multiple of N.
    let array_slice = unsafe { as_chunks_unchecked(multiple_of_n) };
    (array_slice, remainder)
}

/// Splits the slice into a slice of `N`-element arrays,
/// assuming that there's no remainder.
///
/// # Safety
///
/// This may only be called when
/// - The slice splits exactly into `N`-element chunks (aka `self.len() % N == 0`).
/// - `N != 0`.
#[inline]
unsafe fn as_chunks_unchecked<T, const N: usize>(s: &[T]) -> &[[T; N]] {
    let this = s;
    // SAFETY: Caller must guarantee that `N` is nonzero and exactly divides the slice length
    let new_len = {
        assert!(
            N != 0 && this.len() % N == 0,
            "slice::as_chunks_unchecked requires `N != 0` and the slice to split exactly into `N`-element chunks"
        );
        s.len() / N
    };
    // SAFETY: We cast a slice of `new_len * N` elements into
    // a slice of `new_len` many `N` elements chunks.
    unsafe { std::slice::from_raw_parts(s.as_ptr().cast(), new_len) }
}

/// Splits the slice into a slice of `N`-element arrays,
/// starting at the beginning of the slice,
/// and a remainder slice with length strictly less than `N`.
///
/// # Panics
///
/// Panics if `N` is 0. This check will most probably get changed to a compile time
/// error before this method gets stabilized.
#[inline]
pub fn slice_as_chunks_mut<T, const N: usize>(s: &mut [T]) -> (&mut [[T; N]], &mut [T]) {
    assert_ne!(N, 0);
    let len = s.len() / N;
    let (multiple_of_n, remainder) = s.split_at_mut(len * N);
    // SAFETY: We already panicked for zero, and ensured by construction
    // that the length of the subslice is a multiple of N.
    let array_slice = unsafe { as_chunks_unchecked_mut(multiple_of_n) };
    (array_slice, remainder)
}

/// Splits the slice into a slice of `N`-element arrays,
/// assuming that there's no remainder.
///
/// # Safety
///
/// This may only be called when
/// - The slice splits exactly into `N`-element chunks (aka `self.len() % N == 0`).
/// - `N != 0`.
#[inline]
unsafe fn as_chunks_unchecked_mut<T, const N: usize>(s: &mut [T]) -> &mut [[T; N]] {
    let this = &*s;
    // SAFETY: Caller must guarantee that `N` is nonzero and exactly divides the slice length
    let new_len = {
        assert!(
            N != 0 && this.len() % N == 0,
            "slice::as_chunks_unchecked_mut requires `N != 0` and the slice to split exactly into `N`-element chunks",
        );
        this.len() / N
    };
    // SAFETY: We cast a slice of `new_len * N` elements into
    // a slice of `new_len` many `N` elements chunks.
    unsafe { std::slice::from_raw_parts_mut(s.as_mut_ptr().cast(), new_len) }
}

/// Takes a `Vec<[T; N]>` and flattens it into a `Vec<T>`.
///
/// # Panics
///
/// Panics if the length of the resulting vector would overflow a `usize`.
///
/// This is only possible when flattening a vector of arrays of zero-sized
/// types, and thus tends to be irrelevant in practice. If
/// `size_of::<T>() > 0`, this will never panic.
pub fn vec_into_flattened<T, const N: usize>(s: Vec<[T; N]>) -> Vec<T> {
    let (ptr, len, cap) = into_raw_parts(s);
    let (new_len, new_cap) = if std::mem::size_of::<T>() == 0 {
        (len.checked_mul(N).expect("vec len overflow"), usize::MAX)
    } else {
        // SAFETY:
        // - `cap * N` cannot overflow because the allocation is already in
        // the address space.
        // - Each `[T; N]` has `N` valid elements, so there are `len * N`
        // valid elements in the allocation.
        (len * N, cap * N)
    };
    // SAFETY:
    // - `ptr` was allocated by `self`
    // - `ptr` is well-aligned because `[T; N]` has the same alignment as `T`.
    // - `new_cap` refers to the same sized allocation as `cap` because
    // `new_cap * size_of::<T>()` == `cap * size_of::<[T; N]>()`
    // - `len` <= `cap`, so `len * N` <= `cap * N`.
    unsafe { Vec::<T>::from_raw_parts(ptr.cast(), new_len, new_cap) }
}

/// Takes a `Vec<T>` and chunks it into a `Vec<[T; N]>`.
///
/// # Panics
///
/// If the length of the vector is not a multiple of `N`.
pub fn vec_into_chunks<T, const N: usize>(mut s: Vec<T>) -> Vec<[T; N]> {
    assert!(
        s.len() % N == 0,
        "vec_into_chunks: len must be a multiple of N"
    );
    s.shrink_to_fit();

    let (ptr, len, cap) = into_raw_parts(s);

    let (new_len, new_cap) = if std::mem::size_of::<T>() == 0 {
        (len / N, usize::MAX)
    } else {
        // SAFETY:
        // - `cap * N` cannot overflow because the allocation is already in
        // the address space.
        // - Each `[T; N]` has `N` valid elements, so there are `len * N`
        // valid elements in the allocation.
        (len / N, cap / N)
    };
    // SAFETY:
    // - `ptr` was allocated by `self`
    // - `ptr` is well-aligned because `[T; N]` has the same alignment as `T`.
    // - `new_cap` refers to the same sized allocation as `cap` because
    // `new_cap * size_of::<T>()` == `cap * size_of::<[T; N]>()`
    // - `len` <= `cap`, so `len * N` <= `cap * N`.
    unsafe { Vec::<[T; N]>::from_raw_parts(ptr.cast(), new_len, new_cap) }
}

/// Decomposes a `Vec<T>` into its raw components.
///
/// Returns the raw pointer to the underlying data, the length of
/// the vector (in elements), and the allocated capacity of the
/// data (in elements). These are the same arguments in the same
/// order as the arguments to [`from_raw_parts`].
///
/// After calling this function, the caller is responsible for the
/// memory previously managed by the `Vec`. The only way to do
/// this is to convert the raw pointer, length, and capacity back
/// into a `Vec` with the [`from_raw_parts`] function, allowing
/// the destructor to perform the cleanup.
fn into_raw_parts<T>(s: Vec<T>) -> (*mut T, usize, usize) {
    let mut me = ManuallyDrop::new(s);
    (me.as_mut_ptr(), me.len(), me.capacity())
}

/// Takes a `Vec<T>` and tries to transmute it into a `Vec<U>`.
///
/// If the transmute is not possible, the original vector is returned.
///
/// # Safety
///
/// This operation only safe if `T` can be transmuted into `U`.
pub unsafe fn vec_try_transmute<T, U>(s: Vec<T>) -> Result<Vec<U>, Vec<T>> {
    if std::mem::size_of::<T>() != std::mem::size_of::<U>() {
        // not the same size
        return Err(s);
    }

    let (ptr, len, cap) = into_raw_parts(s);

    // check alignment
    if ptr as usize % std::mem::align_of::<U>() != 0 {
        // alignment doesn't work out
        return Err(unsafe { Vec::from_raw_parts(ptr.cast(), len, cap) });
    }

    Ok(unsafe { Vec::from_raw_parts(ptr.cast(), len, cap) })
}

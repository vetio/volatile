#![cfg_attr(feature = "const_fn", feature(const_fn))]

//! Provides wrapper types `Volatile`, `ReadOnly`, `WriteOnly`, `ReadWrite`, which wrap any copy-able type and allows for
//! volatile memory access to wrapped value. Volatile memory accesses are never optimized away by
//! the compiler, and are useful in many low-level systems programming and concurrent contexts.
//!
//! The wrapper types *do not* enforce any atomicity guarantees; to also get atomicity, consider
//! looking at the `Atomic` wrapper type found in `libcore` or `libstd`.
//!
//! These wrappers do not depend on the standard library and never panic.
//!
//! # Dealing with Volatile Pointers
//!
//! Frequently, one may have to deal with volatile pointers, eg, writes to specific memory
//! locations. The canonical way to solve this is to cast the pointer to a volatile wrapper
//! directly, eg:
//!
//! ```rust
//! use volatile::Volatile;
//!
//! let mut_ptr = 0xFEE00000 as *mut u32;
//!
//! let volatile_ptr = mut_ptr as *mut Volatile<u32>;
//! ```
//!
//! and then perform operations on the pointer as usual in a volatile way. This method works as all
//! of the volatile wrapper types are the same size as their contained values.
#![no_std]

use access::{ReadWrite, Readable, Writable};
use core::{marker::PhantomData, ptr};

pub mod access;
pub mod foo;
pub mod reference;
pub mod reference_mut;
#[cfg(test)]
mod tests;

/// A wrapper type around a volatile variable, which allows for volatile reads and writes
/// to the contained value. The stored type needs to be `Copy`, as volatile reads and writes
/// take and return copies of the value.
///
/// The size of this struct is the same as the size of the contained type.
#[derive(Debug, Default)]
#[repr(transparent)]
pub struct Volatile<T, A = ReadWrite> {
    value: T,
    access: PhantomData<A>,
}

impl<T> Volatile<T> {
    /// Construct a new volatile instance wrapping the given value.
    ///
    /// ```rust
    /// use volatile::Volatile;
    ///
    /// let value = Volatile::new(0u32);
    /// ```
    ///
    /// # Panics
    ///
    /// This method never panics.
    pub const fn new(value: T) -> Volatile<T> {
        Volatile {
            value,
            access: PhantomData,
        }
    }
}

impl<T: Copy, A: Readable> Volatile<T, A> {
    /// Performs a volatile read of the contained value, returning a copy
    /// of the read value. Volatile reads are guaranteed not to be optimized
    /// away by the compiler, but by themselves do not have atomic ordering
    /// guarantees. To also get atomicity, consider looking at the `Atomic` wrapper type.
    ///
    /// ```rust
    /// use volatile::Volatile;
    ///
    /// let value = Volatile::new(42u32);
    ///
    /// assert_eq!(value.read(), 42u32);
    /// ```
    ///
    /// # Panics
    ///
    /// This method never panics.
    pub fn read(&self) -> T {
        // UNSAFE: Safe, as we know that our internal value exists.
        unsafe { ptr::read_volatile(&self.value) }
    }
}

impl<T: Copy, A: Writable> Volatile<T, A> {
    /// Performs a volatile write, setting the contained value to the given value `value`. Volatile
    /// writes are guaranteed to not be optimized away by the compiler, but by themselves do not
    /// have atomic ordering guarantees. To also get atomicity, consider looking at the `Atomic`
    /// wrapper type.
    ///
    /// ```rust
    /// use volatile::Volatile;
    ///
    /// let mut value = Volatile::new(0u32);
    ///
    /// value.write(42u32);
    ///
    /// assert_eq!(value.read(), 42u32);
    /// ```
    ///
    /// # Panics
    ///
    /// This method never panics.
    pub fn write(&mut self, value: T) {
        // UNSAFE: Safe, as we know that our internal value exists.
        unsafe { ptr::write_volatile(&mut self.value, value) };
    }
}

impl<T: Copy, A: Readable + Writable> Volatile<T, A> {
    /// Performs a volatile read of the contained value, passes a mutable reference to it to the
    /// function `f`, and then performs a volatile write of the (potentially updated) value back to
    /// the contained value.
    ///
    /// ```rust
    /// use volatile::Volatile;
    ///
    /// let mut value = Volatile::new(21u32);
    ///
    /// value.update(|val_ref| *val_ref *= 2);
    ///
    /// assert_eq!(value.read(), 42u32);
    /// ```
    ///
    /// # Panics
    ///
    /// Ths method never panics.
    pub fn update<F>(&mut self, f: F)
    where
        F: FnOnce(&mut T),
    {
        let mut value = self.read();
        f(&mut value);
        self.write(value);
    }
}

impl<T: Copy, A: Readable> Clone for Volatile<T, A> {
    fn clone(&self) -> Self {
        Volatile {
            value: self.read(),
            access: self.access,
        }
    }
}

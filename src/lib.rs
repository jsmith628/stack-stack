#![doc = include_str!("../README.md")]
#![no_std]

use core::iter::*;
use core::mem::*;
use core::ops::*;
use core::slice::*;
use core::borrow::*;
use core::hash::*;
use core::ptr::copy_nonoverlapping;
use core::fmt::{Debug, Formatter, Result as FmtResult};

///
/// Utility macro for creating a stack from values
/// 
/// # Panics
/// Panics if the capacity provided is less than the quantity of values
/// 
/// # Examples
/// 
/// ```rust
/// # use stack_stack::{Stack, stack};
/// let s1 = stack![6,2,8,3,1; 10];
/// assert_eq!(s1, [6,2,8,3,1]);
/// assert_eq!(s1.capacity(), 10);
/// 
/// let s2 = stack![3; 4; 5];
/// assert_eq!(s2, [3,3,3,3]);
/// assert_eq!(s2.capacity(), 5);
/// 
/// ```
/// 
#[macro_export]
macro_rules! stack {

    ($elem:expr; $n:expr; $cap:expr) => {
        {
            let mut stack = Stack::with_capacity::<$cap>();
            stack.resize($n, $elem);
            stack
        }
    };

    ($($x:expr),+ $(,)?; $cap:expr) => {
        {
            let vals = [$($x),*];
            let mut stack = Stack::with_capacity::<$cap>();
            if stack.extend_from_slice(&vals).is_err() {
                panic!(
                    "Attempted to create a stack of len {}, but the capacity was {}",
                    vals.len(), $cap
                )
            }
            stack
        }
    }

}

///
/// A basic fixed-capacity stack stored statically
/// 
/// The design of its methods is based pretty closely on `Vec`, with
/// the primary difference being that [`push`](Self::push()) returns an [`Option`]
/// containing the pushed value if `self` is at capacity.
/// 
pub struct Stack<T, const N:usize> {
    len: usize,
    data: [MaybeUninit<T>; N]
}

impl<T:Clone, const N:usize> Clone for Stack<T, N> {
    fn clone(&self) -> Self {
        let mut new = Stack::new();
        while new.len() < self.len() {
            new.push(self[new.len()].clone()).ok();
        }
        new
    }
}

impl<T, const N:usize> Drop for Stack<T, N> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T, const N:usize> Deref for Stack<T,N> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, const N:usize> DerefMut for Stack<T,N> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T, const N:usize> Default for Stack<T,N> {
    fn default() -> Self { Self::new() }
}

impl<T, const N:usize> From<[T;N]> for Stack<T,N> {
    fn from(array: [T;N]) -> Self { Self::from_array(array) }
}

impl<T, const N:usize> AsRef<[T]> for Stack<T,N> {
    fn as_ref(&self) -> &[T] { self.as_slice() }
}

impl<T, const N:usize> AsMut<[T]> for Stack<T,N> {
    fn as_mut(&mut self) -> &mut [T] { self.as_mut_slice() }
}

impl<T, const N:usize> Borrow<[T]> for Stack<T,N> {
    fn borrow(&self) -> &[T] { self.as_slice() }
}

impl<T, const N:usize> BorrowMut<[T]> for Stack<T,N> {
    fn borrow_mut(&mut self) -> &mut [T] { self.as_mut_slice() }
}

impl<T, I:SliceIndex<[T]>, const N:usize> Index<I> for Stack<T,N> {
    type Output = I::Output;
    fn index(&self, i:I) -> &Self::Output {
        &self.as_slice()[i]
    }
}

impl<T, I:SliceIndex<[T]>, const N:usize> IndexMut<I> for Stack<T,N> {
    fn index_mut(&mut self, i:I) -> &mut Self::Output {
        &mut self.as_mut_slice()[i]
    }
}

impl<T:Eq, const N:usize> Eq for Stack<T,N> {}

impl<T:PartialEq<U>, U, const N:usize, const M:usize> PartialEq<Stack<U,M>> for Stack<T,N> {
    fn eq(&self, other: &Stack<U,M>) -> bool { self.as_slice().eq(other.as_slice()) }
    fn ne(&self, other: &Stack<U,M>) -> bool { self.as_slice().ne(other.as_slice()) }
}

impl<T:PartialEq<U>, U, const N:usize, const M:usize> PartialEq<[U;M]> for Stack<T,N> {
    fn eq(&self, other: &[U;M]) -> bool { self.as_slice().eq(other) }
    fn ne(&self, other: &[U;M]) -> bool { self.as_slice().ne(other) }
}

impl<T:PartialEq<U>, U, const N:usize, const M:usize> PartialEq<Stack<U,M>> for [T;N] {
    fn eq(&self, other: &Stack<U,M>) -> bool { self.eq(other.as_slice()) }
    fn ne(&self, other: &Stack<U,M>) -> bool { self.ne(other.as_slice()) }
}

impl<T:PartialEq<U>, U, const N:usize> PartialEq<[U]> for Stack<T,N> {
    fn eq(&self, other: &[U]) -> bool { self.as_slice().eq(other) }
    fn ne(&self, other: &[U]) -> bool { self.as_slice().ne(other) }
}

impl<T:PartialEq<U>, U, const N:usize> PartialEq<&[U]> for Stack<T,N> {
    fn eq(&self, other: &&[U]) -> bool { self.as_slice().eq(*other) }
    fn ne(&self, other: &&[U]) -> bool { self.as_slice().ne(*other) }
}

impl<T:PartialEq<U>, U, const N:usize> PartialEq<&mut [U]> for Stack<T,N> {
    fn eq(&self, other: &&mut [U]) -> bool { self.as_slice().eq(*other) }
    fn ne(&self, other: &&mut [U]) -> bool { self.as_slice().ne(*other) }
}

impl<T:PartialEq<U>, U, const N:usize> PartialEq<Stack<U,N>> for [T] {
    fn eq(&self, other: &Stack<U,N>) -> bool { self.eq(other.as_slice()) }
    fn ne(&self, other: &Stack<U,N>) -> bool { self.ne(other.as_slice()) }
}

impl<T:PartialEq<U>, U, const N:usize> PartialEq<Stack<U,N>> for &[T] {
    fn eq(&self, other: &Stack<U,N>) -> bool { (**self).eq(other) }
    fn ne(&self, other: &Stack<U,N>) -> bool { (**self).ne(other) }
}

impl<T:PartialEq<U>, U, const N:usize> PartialEq<Stack<U,N>> for &mut[T] {
    fn eq(&self, other: &Stack<U,N>) -> bool { (**self).eq(other) }
    fn ne(&self, other: &Stack<U,N>) -> bool { (**self).ne(other) }
}

impl<T:Hash, const N:usize> Hash for Stack<T,N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state);
    }
}

impl<T:Debug, const N:usize> Debug for Stack<T,N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(self.as_slice(), f)
    }
}

impl<T> Stack<T, 0> {


    ///
    /// Creates a stack with a given capacity
    /// 
    /// ```
    /// # use stack_stack::Stack;
    /// let mut s = Stack::with_capacity::<2>();
    /// assert_eq!(s.len(), 0);
    /// assert_eq!(s.capacity(), 2);
    /// 
    /// assert!(!s.is_full());
    /// s.push(1);
    /// assert!(!s.is_full());
    /// s.push(2);
    /// assert!(s.is_full());
    /// 
    /// ```
    /// 
    pub const fn with_capacity<const N:usize>() -> Stack<T,N> {
        Stack::new()
    }
}

impl<T, const N:usize> Stack<T, N> {

    ///
    /// Creates an empty stack
    /// 
    /// Tends to be more clunky to use than [`Self::with_capacity()`], as type
    /// annotations are usually required in order to specify the capacity
    /// 
    /// # Examples
    /// ```
    /// # use stack_stack::Stack;
    /// let mut stack: Stack<i32, 10> = Stack::new();
    /// ```
    /// 
    pub const fn new() -> Self {
        Self { len: 0, data: unsafe { MaybeUninit::uninit().assume_init() } }
    }

    ///
    /// Creates a stack and fills it with the values of an array
    /// 
    /// Note that this will set the capacity to the size of the array,
    /// and thus the stack will be completely full once initialized
    /// 
    /// # Examples
    /// ```
    /// # use stack_stack::Stack;
    /// let mut s = Stack::from_array([6,2,8]);
    /// assert_eq!(s, [6,2,8]);
    /// 
    /// //the stack starts out full
    /// assert_eq!(s.len(), s.capacity());
    /// assert!(s.is_full());
    /// ```
    /// 
    pub fn from_array(array: [T;N]) -> Self {
        Self { len:array.len(), data: array.map(|t| MaybeUninit::new(t)) }
    }

    ///
    /// Creates a stack and initializes it to the values of an array up to the given length
    /// 
    /// Useful for initializing a stack with values while still leaving some capacity
    /// 
    /// # Panics
    /// Panics if the length is greater than the size of the array
    /// 
    /// # Examples
    /// ```
    /// # use stack_stack::Stack;
    /// let s = Stack::using_array([6,2,8,3,1], 3);
    /// assert_eq!(s, [6,2,8]);
    /// assert_eq!(s.capacity(), 5);
    /// ```
    /// 
    pub fn using_array(array: [T;N], len: usize) -> Self {
        if len > N {
            panic!("Attempted to create stack with len {len}, but the capacity was {N}");
        }
        unsafe { Self::using_array_unchecked(array, len) }
    }

    ///
    /// Same as [`Self::using_array()`] but doesn't check if `len` is greater than `N`
    /// 
    /// # Safety
    /// Caller must guarrantee that `len` is less than or equal to `N`
    /// 
    pub unsafe fn using_array_unchecked(array: [T;N], len: usize) -> Self {
        let mut array = Self::from_array(array);
        array.truncate(len);
        array
    }

    ///
    /// Constructs a `Stack` using the given buffer and sets the length directly
    /// 
    /// # Safety
    /// Caller must guarrantee that the first `len` values in `buf` are properly
    /// initialized and that `len` is less than or equal to `N`
    /// 
    /// # Examples
    /// ```
    /// # use stack_stack::Stack;
    /// use core::mem::MaybeUninit;
    ///  
    /// let s = unsafe {
    ///     Stack::from_raw_parts(
    ///         [MaybeUninit::new(6), MaybeUninit::new(2), MaybeUninit::new(8),
    ///          MaybeUninit::uninit(), MaybeUninit::uninit(), MaybeUninit::uninit()], 3
    ///     )
    /// };
    /// 
    /// assert_eq!(s, [6,2,8]);
    /// assert_eq!(s.capacity(), 6);
    /// ```
    /// 
    pub const unsafe fn from_raw_parts(buf: [MaybeUninit<T>; N], len: usize) -> Self {
        Self { len, data: buf }
    }

    /// The quantity of values in the stack
    pub const fn len(&self) -> usize { self.len }

    /// The total quantity of values that this stack can hold.
    /// Equivalent to `N`
    pub const fn capacity(&self) -> usize { N }
    
    /// `true` when this stack contains no elements
    pub const fn is_empty(&self) -> bool { self.len() == 0 }

    /// `true` when this stack contains as many elements as the capacity
    pub const fn is_full(&self) -> bool { self.len() >= self.capacity() }

    /// Returns a pointer to the data in the stack
    pub const fn as_ptr(&self) -> *const T { self.data.as_ptr() as *const _ }

    /// Returns a mutable pointer to the data in the stack
    pub fn as_mut_ptr(&mut self) -> *mut T { self.data.as_mut_ptr() as *mut _ }

    /// Returns a slice of the data in the stac
    pub const fn as_slice(&self) -> &[T] {
        unsafe { from_raw_parts(self.data.as_ptr() as *const _, self.len) }
    }

    /// Returns a mutable slice of the data in the stack
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { from_raw_parts_mut(self.data.as_mut_ptr() as *mut _, self.len) }
    }

    ///
    /// Inserts an element to the end of the stack
    /// 
    /// If this stack is full, it returns the function argument in a `Some()`
    /// value and leaves the stack unchanged
    /// 
    /// # Examples
    /// ```
    /// # use stack_stack::Stack;
    /// let mut s = Stack::with_capacity::<3>();
    /// 
    /// assert_eq!(s.push(6), Ok(()));
    /// assert_eq!(s.push(2), Ok(()));
    /// assert_eq!(s.push(8), Ok(()));
    /// assert_eq!(s, [6, 2, 8]);
    /// 
    /// assert_eq!(s.push(3), Err(3));
    /// assert_eq!(s.push(1), Err(1));
    /// assert_eq!(s, [6, 2, 8])
    /// ```
    /// 
    /// If confident that there will be no overflow, the `#[must_use]` warnings
    /// can be ergonomically ignored by postfixing [`Result::ok()`]
    /// 
    /// ```
    /// # use stack_stack::Stack;
    /// let mut s = Stack::with_capacity::<3>();
    /// s.push(6).ok();
    /// s.push(2).ok();
    /// s.push(8).ok();
    /// assert_eq!(s, [6, 2, 8]);
    /// ```
    /// 
    /// 
    pub fn push(&mut self, x:T) -> Result<(),T> {
        if self.is_full() { return Err(x); }
        self.data[self.len] = MaybeUninit::new(x);
        self.len += 1;
        Ok(())
    }

    ///
    /// Removes the last element from the stack and returns it
    /// 
    /// If this stack is empty, `None` is returned instead
    /// 
    /// # Examples
    /// ```
    /// # use stack_stack::{Stack, stack};
    /// let mut s = stack![6,2,8; 3];
    /// 
    /// assert_eq!(s.pop(), Some(8));
    /// assert_eq!(s, [6, 2]);
    /// 
    /// assert_eq!(s.pop(), Some(2));
    /// assert_eq!(s.pop(), Some(6));
    /// assert_eq!(s.pop(), None);
    /// 
    /// assert_eq!(s, []);
    /// 
    /// ```
    /// 
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() { return None; }
        self.len -= 1;
        unsafe { Some(self.data[self.len].assume_init_read()) }
    }

    ///
    /// Copies the contents of this stack into another stack of a different
    /// capacity
    /// 
    /// If the new capacity is larger, then the resulting stack should
    /// be equal to this one except with extra storage space.
    /// 
    /// If the new capacity is smaller, then items will be truncated from
    /// the end to fit the new size
    /// 
    /// # Examples
    /// ```
    /// # use stack_stack::{Stack, stack};
    /// let s1 = stack![6, 2, 8; 3];
    /// let s2 = s1.clone().resize_capacity::<10>();
    /// 
    /// assert_eq!(s1, s2);
    /// assert_eq!(s1.capacity(), 3);
    /// assert_eq!(s2.capacity(), 10);
    /// 
    /// let s3 = s1.clone().resize_capacity::<2>();
    /// assert_ne!(s1, s3);
    /// assert_eq!(s3, [6, 2]);
    /// assert_eq!(s3.capacity(), 2);
    /// 
    /// ```
    /// 
    pub fn resize_capacity<const M: usize>(self) -> Stack<T,M> {
        let mut new = Stack::new();
        for x in self {
            if new.push(x).is_err() { break; } //stop early if M < N
        }
        new
    }

    // pub fn into_vec() -> 
    // pub fn into_boxed_slice() 

    ///
    /// Removes all elements after the given length
    /// 
    /// If `len` is greater than `Self::len()`, then the stack remains unchanged
    /// 
    /// # Examples
    /// ```
    /// # use stack_stack::{Stack, stack};
    /// let mut s1 = stack![6, 2, 8, 3, 1; 5];
    /// s1.truncate(3);
    /// assert_eq!(s1, [6, 2, 8]);
    /// ```
    /// 
    pub fn truncate(&mut self, len: usize) {
        let target = self.len().min(len);
        if needs_drop::<T>() {
            while self.len() > target { self.pop(); }
        } else {
            unsafe { self.set_len(target) }
        }
    }

    ///
    /// Directly sets the length of the stack without changing the backing array
    /// 
    /// # Safety
    /// Caller must guarrantee the following:
    /// - `len` is no greater than the capacity
    /// - if `len` is larger than the current length, that any new elements are
    ///   properly initialized
    /// - if `len` is smaller than the current length, that all elements after
    ///   the new length are either properly dropped or that leaks are acceptable
    /// 
    /// # Examples
    /// Useful to read data in from FFI apis. See `Vec::set_len()` for
    /// examples
    /// 
    pub unsafe fn set_len(&mut self, len: usize) {
        self.len = len;
    }

    fn check_bounds(&self, index: usize, op:&str) {
        //TODO: fill in error message
        if index >= self.len() {
            panic!("Attempted to {op} item at {index}, but the len was {}", self.len());
        }
    }

    fn check_capacity(&self, size: usize, op:&str) {
        if size > self.capacity() {
            panic!("Attempted to {op} to {size}, but the capacity is {}", self.capacity())
        }
    }

    ///
    /// Quickly removes and returns the element at `index` by swapping it with
    /// the last element in the stack
    /// 
    /// This operation is quick and O(1), as it does not need to shift over any
    /// other elements. However, it does change the ordering of the stack which
    /// may be unacceptable for some applications.
    /// 
    /// # Examples
    /// ```
    /// # use stack_stack::{Stack, stack};
    /// let mut s1 = stack![1, 2, 3, 4, 5; 5];
    /// s1.swap_remove(1);
    /// assert_eq!(s1, [1, 5, 3, 4]);
    /// ```
    /// 
    pub fn swap_remove(&mut self, index: usize) -> T {
        self.check_bounds(index, "remove");
        unsafe {
            let ret = self.data[index].assume_init_read();
            self.len -= 1;
            copy_nonoverlapping(
                self.data[self.len].as_ptr(),
                self.data[index].as_mut_ptr(),
                1
            );
            ret
        }
    }

    ///
    /// Adds an element to the stack at an index
    /// 
    /// If the stack is full, then the value is still inserted, but the last
    /// element of the stack is removed and returned.
    /// 
    /// # Examples
    /// ```
    /// # use stack_stack::{Stack, stack};
    /// let mut s1 = stack![1, 2, 3, 4, 5; 6];
    /// 
    /// s1.insert(2, 10);
    /// assert_eq!(s1, [1, 2, 10, 3, 4, 5]);
    /// 
    /// assert_eq!(s1.insert(2, 10), Some(5));
    /// assert_eq!(s1, [1, 2, 10, 10, 3, 4]);
    /// 
    /// ```
    /// 
    pub fn insert(&mut self, index: usize, element: T) -> Option<T> {
        self.check_bounds(index, "insert");
        let mut temp = MaybeUninit::new(element);
        for i in index..self.len() {
            swap(&mut self.data[i], &mut temp);
        }

        if self.is_full() {
            //return the overflow
            Some(unsafe { temp.assume_init() })
        } else {
            self.data[self.len] = temp;
            self.len += 1;
            None
        }

    }

    ///
    /// Removes and returns the element at a given index
    /// 
    /// # Panics
    /// Panics if the index is greater than the size of the stack
    /// 
    /// # Examples
    /// ```
    /// # use stack_stack::{Stack, stack};
    /// let mut s1 = stack![1, 2, 3, 4, 5; 5];
    /// assert_eq!(s1.remove(2), 3);
    /// assert_eq!(s1, [1, 2, 4, 5]);
    /// ```
    /// 
    pub fn remove(&mut self, index: usize) -> T {
        self.check_bounds(index, "remove");
        unsafe {
            let ret = self.data[index].assume_init_read();
            for i in index+1..self.len() {
                copy_nonoverlapping(
                    self.data[i].as_ptr(), self.data[i-1].as_mut_ptr(), 1
                )
            }
            self.len -= 1;
            ret
        }
    }

    ///
    /// Removes all elements from the stack
    /// 
    /// # Examples
    /// ```
    /// # use stack_stack::{Stack, stack};
    /// let mut s1 = stack![1, 2, 3, 4, 5; 5];
    /// s1.clear();
    /// assert_eq!(s1, []);
    /// ```
    /// 
    pub fn clear(&mut self) {
        if needs_drop::<T>() {
            for i in 0..self.len {
                unsafe { self.data[i].assume_init_drop(); }
            }
        }
        self.len = 0;
    }

    ///
    /// Resizes the stack in place to the given length, filling with `x` when needed
    /// 
    /// If the new length is less than the current one, the stack is truncated
    /// to the new size, else, the stack is grown to the new size, cloning `x`
    /// in to fill the new space as needed.
    /// 
    /// # Panics
    /// Panics if the new length is greater than the capacity
    /// 
    /// # Examples
    /// ```
    /// # use stack_stack::{Stack, stack};
    /// let mut s1 = stack![6, 2, 8; 5];
    /// s1.resize(5, 10);
    /// assert_eq!(s1, [6,2,8,10,10]);
    /// 
    /// let mut s2 = stack![6, 2, 8, 3, 1; 5];
    /// s2.resize(3, 10);
    /// assert_eq!(s2, [6, 2, 8]);
    /// 
    /// ```
    pub fn resize(&mut self, new_len:usize, x:T) where T:Clone {
        self.resize_with(new_len, || x.clone())
    }

    ///
    /// Resizes the stack in place to the given length
    /// 
    /// If the new length is less than the current one, the stack is truncated
    /// to the new size, else, the stack is grown to the new size, using the
    /// given function to create values to fill into the new space as needed.
    ///
    /// # Examples
    /// ```
    /// # use stack_stack::{Stack, stack};
    /// let mut s1 = stack![6, 2, 8; 5];
    /// s1.resize_with(5, || 10);
    /// assert_eq!(s1, [6,2,8,10,10]);
    /// 
    /// let mut s2 = stack![6, 2, 8, 3, 1; 5];
    /// s2.resize_with(3, || 10);
    /// assert_eq!(s2, [6, 2, 8]);
    /// 
    /// ```
    pub fn resize_with<F:FnMut()->T>(&mut self, new_len:usize, mut f:F) {
        self.check_capacity(new_len, "resize");
        if new_len < self.len() {
            self.truncate(new_len);
        } else {
            while self.len() < new_len {
                self.push(f()).ok();
            }
        }
    }

    ///
    /// Appends the stack with values from a slice
    /// 
    /// If extending would take the stack over-capacity, then
    /// as many values as possible are pushed in and a sub-slice of
    /// the remaining elements is returned.
    /// 
    /// # Examples
    /// ```
    /// # use stack_stack::{Stack, stack};
    /// let mut s1 = stack![6, 2, 8; 5];
    /// assert_eq!(s1.extend_from_slice(&[3,1]), Ok(()));
    /// assert_eq!(s1, [6,2,8,3,1]);
    /// 
    /// 
    /// let mut s2 = stack![6, 2, 8; 5];
    /// assert_eq!(s2.extend_from_slice(&[3,1,8,5]), Err(&[8,5] as &[_]));
    /// assert_eq!(s2, [6,2,8,3,1]);
    /// 
    /// ```
    /// 
    pub fn extend_from_slice<'a>(&mut self, mut other:&'a[T]) -> Result<(),&'a[T]>
    where T:Clone
    {
        while let Some((first, rest)) = other.split_first() {
            self.push(first.clone()).map_err(|_| other)?;
            other = rest;
        }
        Ok(())
    }


    ///
    /// Appends the stack with values from an iterator
    /// 
    /// If extending would take the stack over-capacity, then
    /// as many values as possible are pushed in and the iterator
    /// of the remaining elements is returned
    /// 
    /// # Examples
    /// ```
    /// # use stack_stack::{Stack, stack};
    /// let mut s1 = stack![9,9,9,9; 10];
    /// assert_eq!(s1.extend_from_iter(0..3), Ok(()));
    /// assert_eq!(s1, [9,9,9,9,0,1,2]);
    /// 
    /// 
    /// let mut s1 = stack![9,9,9,9; 7];
    /// assert_eq!(s1.extend_from_iter(0..10), Err(3..10)); //only 3 elements are appended
    /// assert_eq!(s1, [9,9,9,9,0,1,2]);
    /// 
    /// ```
    /// 
    pub fn extend_from_iter<I:Iterator<Item=T>>(&mut self, mut iter:I) -> Result<(), I> {
        loop {
            if self.is_full() {
                return Err(iter);
            } else if let Some(x) = iter.next() {
                self.push(x).ok();
            } else {
                return Ok(());
            }
        }
    }

}

impl<T,const N:usize> IntoIterator for Stack<T,N> {
    type Item = T;
    type IntoIter = IntoIter<T,N>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter { index: 0, stack: self }
    }
}

/// An iterator over the values of a [`Stack`]
pub struct IntoIter<T, const N:usize> {
    index: usize,
    stack: Stack<T,N>
}

impl<T, const N:usize> IntoIter<T,N> {
    fn remaining(&self) -> usize {
        self.stack.len()-self.index
    }
}

impl<T, const N:usize> Iterator for IntoIter<T,N> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.stack.len() { return None; }
        unsafe {
            let i = self.index;
            self.index += 1;
            Some(self.stack.data[i].assume_init_read())
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining();
        (remaining, Some(remaining))
    }

    fn count(self) -> usize { self.remaining() }

}

impl<T, const N:usize> DoubleEndedIterator for IntoIter<T,N> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.stack.pop()
    }
}

impl<T, const N:usize> ExactSizeIterator for IntoIter<T,N> {}
// unsafe impl<T, const N:usize> TrustedLen for IntoIter<T,N> {}


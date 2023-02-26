#![no_std]

use core::iter::*;
use core::mem::*;
use core::ops::*;
use core::slice::*;
use core::borrow::*;
use core::hash::*;
use core::ptr::copy_nonoverlapping;

pub struct StackVec<T, const N:usize> {
    len: usize,
    data: [MaybeUninit<T>; N]
}

impl<T:Clone, const N:usize> Clone for StackVec<T, N> {
    fn clone(&self) -> Self {
        let mut new = StackVec::new();
        while new.len() < self.len() {
            new.push(self[new.len()].clone());
        }
        new
    }
}

impl<T, const N:usize> Drop for StackVec<T, N> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T, const N:usize> Deref for StackVec<T,N> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe { from_raw_parts(self.data.as_ptr() as *const _, self.len) }
    }
}

impl<T, const N:usize> DerefMut for StackVec<T,N> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { from_raw_parts_mut(self.data.as_mut_ptr() as *mut _, self.len) }
    }
}

impl<T, const N:usize> Default for StackVec<T,N> {
    fn default() -> Self { Self::new() }
}

impl<T, const N:usize> From<[T;N]> for StackVec<T,N> {
    fn from(array: [T;N]) -> Self { Self::from_array(array) }
}

impl<T, const N:usize> AsRef<[T]> for StackVec<T,N> {
    fn as_ref(&self) -> &[T] { self.as_slice() }
}

impl<T, const N:usize> AsMut<[T]> for StackVec<T,N> {
    fn as_mut(&mut self) -> &mut [T] { self.as_mut_slice() }
}

impl<T, const N:usize> Borrow<[T]> for StackVec<T,N> {
    fn borrow(&self) -> &[T] { self.as_slice() }
}

impl<T, const N:usize> BorrowMut<[T]> for StackVec<T,N> {
    fn borrow_mut(&mut self) -> &mut [T] { self.as_mut_slice() }
}

impl<T> StackVec<T, 0> {
    pub fn with_capacity<const N:usize>() -> StackVec<T,N> {
        StackVec::new()
    }
}

impl<T, I:SliceIndex<[T]>, const N:usize> Index<I> for StackVec<T,N> {
    type Output = I::Output;
    fn index(&self, i:I) -> &Self::Output {
        &self.as_slice()[i]
    }
}

impl<T, I:SliceIndex<[T]>, const N:usize> IndexMut<I> for StackVec<T,N> {
    fn index_mut(&mut self, i:I) -> &mut Self::Output {
        &mut self.as_mut_slice()[i]
    }
}

impl<T:Eq, const N:usize> Eq for StackVec<T,N> {}

impl<T:PartialEq<U>, U, const N:usize, const M:usize> PartialEq<StackVec<U,M>> for StackVec<T,N> {
    fn eq(&self, other: &StackVec<U,M>) -> bool { self.as_slice().eq(other.as_slice()) }
    fn ne(&self, other: &StackVec<U,M>) -> bool { self.as_slice().ne(other.as_slice()) }
}

impl<T:PartialEq<U>, U, const N:usize, const M:usize> PartialEq<[U;M]> for StackVec<T,N> {
    fn eq(&self, other: &[U;M]) -> bool { self.as_slice().eq(other) }
    fn ne(&self, other: &[U;M]) -> bool { self.as_slice().ne(other) }
}

impl<T:PartialEq<U>, U, const N:usize, const M:usize> PartialEq<StackVec<U,M>> for [T;N] {
    fn eq(&self, other: &StackVec<U,M>) -> bool { self.eq(other.as_slice()) }
    fn ne(&self, other: &StackVec<U,M>) -> bool { self.ne(other.as_slice()) }
}

impl<T:PartialEq<U>, U, const N:usize> PartialEq<[U]> for StackVec<T,N> {
    fn eq(&self, other: &[U]) -> bool { self.as_slice().eq(other) }
    fn ne(&self, other: &[U]) -> bool { self.as_slice().ne(other) }
}

impl<T:PartialEq<U>, U, const N:usize> PartialEq<&[U]> for StackVec<T,N> {
    fn eq(&self, other: &&[U]) -> bool { self.as_slice().eq(*other) }
    fn ne(&self, other: &&[U]) -> bool { self.as_slice().ne(*other) }
}

impl<T:PartialEq<U>, U, const N:usize> PartialEq<&mut [U]> for StackVec<T,N> {
    fn eq(&self, other: &&mut [U]) -> bool { self.as_slice().eq(*other) }
    fn ne(&self, other: &&mut [U]) -> bool { self.as_slice().ne(*other) }
}

impl<T:PartialEq<U>, U, const N:usize> PartialEq<StackVec<U,N>> for [T] {
    fn eq(&self, other: &StackVec<U,N>) -> bool { self.eq(other.as_slice()) }
    fn ne(&self, other: &StackVec<U,N>) -> bool { self.ne(other.as_slice()) }
}

impl<T:PartialEq<U>, U, const N:usize> PartialEq<StackVec<U,N>> for &[T] {
    fn eq(&self, other: &StackVec<U,N>) -> bool { (**self).eq(other) }
    fn ne(&self, other: &StackVec<U,N>) -> bool { (**self).ne(other) }
}

impl<T:PartialEq<U>, U, const N:usize> PartialEq<StackVec<U,N>> for &mut[T] {
    fn eq(&self, other: &StackVec<U,N>) -> bool { (**self).eq(other) }
    fn ne(&self, other: &StackVec<U,N>) -> bool { (**self).ne(other) }
}

impl<T:Hash, const N:usize> Hash for StackVec<T,N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state);
    }
}

impl<T, const N:usize> StackVec<T, N> {

    pub fn new() -> Self {
        Self { len: 0, data: unsafe { MaybeUninit::uninit().assume_init() } }
    }

    pub fn from_array(array: [T;N]) -> Self {
        Self { len:array.len(), data: array.map(|t| MaybeUninit::new(t)) }
    }

    pub fn using_array(array: [T;N], len: usize) -> Self {
        if len > N { panic!("tried to create stack larger than capacity"); }
        unsafe { Self::using_array_unchecked(array, len) }
    }

    pub unsafe fn using_array_unchecked(array: [T;N], len: usize) -> Self {
        let mut array = Self::from_array(array);
        while array.len() > len { array.pop(); }
        array
    }

    pub unsafe fn from_raw_parts(buf: [MaybeUninit<T>; N], len: usize) -> Self {
        Self { len, data: buf }
    }

    pub fn len(&self) -> usize { self.len }
    pub fn capacity(&self) -> usize { N }
    
    pub fn is_empty(&self) -> bool { self.len() == 0 }
    pub fn is_full(&self) -> bool { self.len() >= self.capacity() }

    pub fn as_ptr(&self) -> *const T { self.data.as_ptr() as *const _ }
    pub fn as_mut_ptr(&mut self) -> *mut T { self.data.as_mut_ptr() as *mut _ }

    pub fn as_slice(&self) -> &[T] { self }
    pub fn as_mut_slice(&mut self) -> &mut [T] { self }

    pub fn push(&mut self, x:T) -> Option<T> {
        if self.is_full() { return Some(x); }

        self.data[self.len] = MaybeUninit::new(x);
        self.len += 1;
        None
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() { return None; }
        unsafe { Some(self.data[self.len-1].assume_init_read()) }
    }

    pub fn resize_capacity<const M: usize>(mut self) -> StackVec<T,M> {
        let mut new = StackVec::new();
        while let Some(x) = self.pop() {
            if new.push(x).is_some() { break; } //stop early if M < N
        }
        new
    }

    // pub fn into_vec() -> 
    // pub fn into_boxed_slice() 

    pub fn truncate(&mut self, len: usize) {
        let target = self.len().min(len);
        if needs_drop::<T>() {
            while self.len() > target { self.pop(); }
        } else {
            unsafe { self.set_len(target) }
        }
    }

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

    pub fn clear(&mut self) {
        if needs_drop::<T>() {
            for i in 0..self.len {
                unsafe { self.data[i].assume_init_drop(); }
            }
        }
        self.len = 0;
    }

    pub fn resize(&mut self, new_len:usize, x:T) where T:Clone {
        self.resize_with(new_len, || x.clone())
    }

    pub fn resize_with<F:FnMut()->T>(&mut self, new_len:usize, mut f:F) {
        self.check_capacity(new_len, "resize");
        if new_len < self.len() {
            self.truncate(new_len);
        } else {
            while self.len() < new_len {
                self.push(f());
            }
        }
    }

    pub fn extend_from_slice<'a>(&mut self, mut other:&'a[T]) -> &'a[T] where T:Clone {
        while other.len() > 0 && !self.is_full() {
            self.push(other[0].clone());
            other = &other[1..];
        }
        other
    }

    pub fn extend_from_iter<I:Iterator<Item=T>>(&mut self, mut iter:I) -> I {
        while !self.is_full() {
            if let Some(x) = iter.next() {
                self.push(x);
            } else {
                break;
            }
        }
        iter
    }

}

impl<T,const N:usize> IntoIterator for StackVec<T,N> {
    type Item = T;
    type IntoIter = IntoIter<T,N>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter { index: 0, stack: self }
    }
}

pub struct IntoIter<T, const N:usize> {
    index: usize,
    stack: StackVec<T,N>
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


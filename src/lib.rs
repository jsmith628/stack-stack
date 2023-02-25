#![no_std]

use core::mem::*;
use core::ops::*;
use core::ptr::copy_nonoverlapping;
use core::slice::*;

pub struct StackVec<T, const N:usize> {
    len: usize,
    data: [MaybeUninit<T>; N]
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

impl<T> StackVec<T, 0> {
    pub fn with_capacity<const N:usize>() -> StackVec<T,N> {
        StackVec::new()
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
        let target = self.len().saturating_sub(len);
        while self.len() > target { self.pop(); }
    }

    pub unsafe fn set_len(&mut self, len: usize) {
        self.len = len;
    }

    fn check_bounds(&self, index: usize, op:&str) {
        //TODO: fill in error message
        if index >= self.len() {
            panic!("Attemted to {op} item at {index}, but the len was {}", self.len());
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

}
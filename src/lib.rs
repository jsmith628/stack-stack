#![no_std]

use core::mem::*;
use core::ops::*;
use core::slice::*;

pub struct StackVec<T, const N:usize> {
    len: usize,
    data: [MaybeUninit<T>; N]
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
    pub fn with_capacity<const M:usize>() -> StackVec<T,M> {
        StackVec::new()
    }
}

impl<T, const N:usize> StackVec<T, N> {

    pub fn new() -> Self {
        Self { len: 0, data: unsafe { MaybeUninit::uninit().assume_init() } }
    }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len() == 0 }
    pub fn is_full(&self) -> bool { self.len() >= N }

    pub fn as_ptr(&self) -> *const T { self.data.as_ptr() as *const _ }
    pub fn as_mut_ptr(&mut self) -> *mut T { self.data.as_mut_ptr() as *mut _ }

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

}
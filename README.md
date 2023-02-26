
A simple crate for a stack-allocated stack. Useful for when you want a small
`Vec` of items with a known size bound and want to avoid dynamic allocation.

# Design

`Stack` implements a basic fixed-size, stack-allocated, FIFO data structure.

Uses `const` generics in order to make the typing more ergonomic.

To account for overflows, any method that increases stack size returns a `Result`
containing any values over capacity inside the `Err` variant. There is **no**
dynamic allocation whatsoever, even when going over-capacity.

# Note

Obviously this is similar to [`smallvec`](https://github.com/servo/rust-smallvec),
and frankly, you should probably just use `smallvec`. The
devs did great work. I just wanted something a little closer to my design
preferences, and it was a fun weekend project.

# Example

```rust
use stack_stack::{Stack, stack};

//Manual creation
let mut s1 = Stack::with_capacity::<5>();

//Pushing returns a result
assert_eq!(s1.push(6), Ok(()));
assert_eq!(s1.push(2), Ok(()));
assert_eq!(s1.push(8), Ok(()));
assert_eq!(s1, [6,2,8]);

//We can ergonomically ignore the #[must_use] warning if needed with `Result::ok()`
s1.push(3).ok();
s1.push(1).ok();
assert_eq!(s1, [6,2,8,3,1]);

//Overflows return return the extra value(s) in a `Result::Err()`
assert_eq!(s1.push(101), Err(101));

//Creation using a list of values and a capacity
let s2 = stack![6,2,8,3,1; 10];
assert_eq!(s2, [6,2,8,3,1]);
assert_eq!(s2.capacity(), 10);

//Repeating a value of `3` 4x times with a capacity of 5
let s3 = stack![3; 4; 5];
assert_eq!(s3, [3,3,3,3]);
assert_eq!(s3.len(), 4);
assert_eq!(s3.capacity(), 5);

```


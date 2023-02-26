
A simple crate for a stack-allocated stack. Useful for when you want a small
`Vec` of items with a known upper bound.

Obviously this is similar to [`smallvec`](https://github.com/servo/rust-smallvec),
the main distinguishing aspect being to have no heap allocation and to use
modern `const` generics. Frankly, you should just use `smallvec`, it's great
work. I just wanted something a little closer to my design preferences, and
it was a fun weekend project.

# Example

```rust
use stack_stack::{Stack, stack};

//Manual creation
let mut s1 = Stack::with_capacity::<5>();
s1.push(6);
s1.push(2);
s1.push(8);
s1.push(3);
s1.push(1);
assert_eq!(s1, [6,2,8,3,1]);

//overflows are returned as options
assert_eq!(s1.push(101), Some(101));

//From macro invocations
let s2 = stack![6,2,8,3,1; 10];
assert_eq!(s2, [6,2,8,3,1]);
assert_eq!(s2.capacity(), 10);

let s3 = stack![3; 4; 4];
assert_eq!(s3, [3,3,3,3]);

```


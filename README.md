
A simple crate for a stack-allocated stack. Useful for when you want a small
list of items with a known upper bound.

Obviously this is similar to [`smallvec`](https://github.com/servo/rust-smallvec),
the main distinguishing aspect being to have no heap allocation and to use
modern `const` generics. Frankly, you should just use `smallvec`, it's great
work. I just wanted something a little closer to my design preferences, and
it was a fun weekend project.

# Example

```rust
use stack_stack::Stack;

//Manual creation
let mut s = Stack::with_capacity::<5>();
s.push(6);
s.push(2);
s.push(8);
s.push(3);
s.push(1);
assert_eq!(s, [6,2,8,3,1]);

//overflows are returned as options
assert_eq!(s.push(101), Some(101));


```


# maybe_box

`maybe_box` is a small Rust library for storing arbitrary data in a
pointer-sized piece of memory, only allocating if necessary.

## Example

```rust
// Wrap a bool into a MaybeBox.
// Because a bool is small enough to fit into the size of a pointer, this
// will not do any allocation.
let mb = MaybeBox::new(true);

// Extract the data back out again.
let my_bool = mb.into_inner();
assert_eq!(my_bool, true);

// Wrap a String into a MaybeBox
// Because a String is too big to fit into the size of a pointer, this
// *will* do allocation.
let mb = MaybeBox::new(String::from("hello"));

// We can unpack the MaybeBox to see whether it was boxed or not.
match mb.unpack() {
    maybe_box::Unpacked::Inline(_) => panic!("String wasn't boxed?!"),
    maybe_box::Unpacked::Boxed(b) => {
        // Unbox our string...
        let my_string = *b;

        // ... and get the String that we boxed.
        assert_eq!(&*my_string, "hello");
    },
};
```


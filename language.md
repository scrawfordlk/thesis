# Minimal Rust Subset

## Language Surface

### Types

- `u64` (unsigned 64-bit integer) or `usize` (unsigned, platform dependant size)
  - `u8` might be necessary for byte handling
- `bool` (necessary for `if` and `while`)

#### Pointers

- `&T`, `&mut T` (borrow checked references)
- `*mut T` (raw pointers)

### Control flow

- `if`
- `while`
- `fn` (functions)
- `match` (pattern matching)

### Operators

- Arithmetic: `+`, `-`, `*`, `/`
- Comparison: `<`, `<=`, `==`, `>`, `>=`
- Memory: `*` (deref), `&`, `&mut`
- Cast: `as` (necessary for e.g. pointer arithmetic)

### Literals

- integer literals
- string literals (immediately casted into raw pointers) (might require `u8` type due to alignment)

```rust
let s: *mut u8 = "Hello World" as *const str as *mut u8; // s is raw pointer to 'H'
```

The double cast is necessary, because Rust only allows casting string literals into `*const str` and not `*mut u8`, but does allow casting arbitrary raw pointers into each other.

The alternative would be to support the type `&str`, which is the type of a string literal:

```rust
let s: &str = "Hello World";
```

But a `&str` is treated as a slice, meaning that you would need to use methods to e.g. get the length:

```rust
let length: u64 = "some string".len() as u64;
```

### Data structures

- Tuple enums (without methods, requires `match`)

### Other included features

- Borrow checker (including lifetime annotations)
- Destructuring (for enums)
- `unsafe` blocks

### Other ideas

- Having methods on enums would improve readability with low complexity, though they aren't necessary

## Entrypoint

Currently still uses `u8` (unsigned 8-bit integers) for alignment reasons.

```rust
#![no_main]
#[unsafe(no_mangle)]
fn main(argc: u64, argv: *mut *mut u8) -> u64 {}
```

## Memory cleanup (open design choice)

### Option A: no `Drop` support

- Only manual cleanup functions (`free_*`).
- Smallest compiler surface.
- Easy to leak memory on early return/error paths.

### Option B: full trait-based `Drop`

- Standard Rust semantics.
- But implies broader trait machinery (undesired complexity).

### Option C (middle ground): special-cased `impl Drop for T`

- Keep only this syntax as a built-in language feature.
- `rustc` bootstrap treats it as normal `Drop`.
- Subset compiler treats it as dedicated cleanup syntax (no general trait system).

```rust
enum Buffer {
    Buffer(*mut u64, u64), // ptr, len
}

impl Drop for Buffer {
    fn drop(&mut self) {
        // special-cased by subset compiler
        // free underlying allocation
    }
}
```

## Cut features

- General structs and struct methods (tuple enums only)
- Full trait system and generics
- Full `Drop` trait machinery (optional: special-cased `impl Drop for T` only)
- Additional numeric types in core language model (`i*`, `f*`, etc.)
- Type inference
- Macro systems/procedural macros in the core subset
- Closures, iterators, async/await
- Standard-library strings/collections (`String`, `Vec`, `HashMap`)

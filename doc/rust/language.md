# Language Description

## Top-level

The only top-level items are:

- `fn`

  ```rust
  fn my_function() { ... }
  ```

- `enum` with tuple variants

  ```rust
  enum MyEnum {
    VariantA,
    VariantB(usize),
  }
  ```

No structs, impl blocks, traits, modules, macros.

## Types

- `usize`
- `u8`
- `char`
- `&str`
- user-defined tuple enums
- references: `&T`, `&mut T`
- raw mutable pointers: `*mut T`
- at most one generic type parameter (per function/enum)

## Literals and comments

You can use:

- decimal integer literals (`usize` or `u8`)
- char literals (`char`)
- string literals (`&str`)

There are only line comments:

```rust
// comment
```

## Variables

Type inference is not supported. Hence, all types need to be written explicitly:

```rust
let s: &str = "Hello World";
let mut y: usize = 2;
```

Assignments (for mutable variables) are straightforward:

```rust
y = y + 1;
*ptr = 42;
```

## Operators

- arithmetic: `+`, `-`, `*`, `/`, `%`
- comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`
- unary operator: `-`, `*`, borrow (`&`, `&mut`)
- cast: `as`

## Control flow

- `if` / `else` (`if`)

  ```rust
  let x: char = if b { 'a' } else { 'b' };
  ```

- `while`

  ```rust
  let mut i = 0;
  while i < 10 {
    i = i + 1;
  }
  ```

- `match`

  ```rust
  let message: char = match my_enum {
    MyEnum::VariantA => 'A',
    MyEnum::VariantB(value) => value as char,
  }
  ```

- `return`

  ```rust
  fn f(a: usize) -> usize {
    if a < 0 {
      return 0;
    }
    ...
  }
  ```

# Importless Sealed Traits

Per-method visibility gates on a public trait — callers import only the trait, nothing else.

For the usual sealed-trait patterns, see Predrag's [Definitive Guide to Sealed Traits in Rust](https://predr.ag/blog/definitive-guide-to-sealed-traits-in-rust/).

## What this solves

Other sealed-method patterns work, but they leak into call sites: you end up importing a helper trait *and* a marker token alongside the thing you actually wanted to use. Bring in the wrong set and the method disappears with a confusing error.

Here, one `use` is enough. The methods that resolve are exactly the ones the call site is allowed to reach — no second trait, no marker type, no token in scope.

```rust
use testing::other::{A, W};

W.in_public();   // ok from anywhere
W.in_crate();    // ok inside the defining crate
W.in_other();    // ok inside crate::other
W.in_private();  // ok inside the defining module
```

Each line stops compiling once you cross the corresponding boundary — same import, different resolution.

## How it works

The trait takes a generic witness parameter, and each method bounds it on a different marker:

```rust
pub trait A<V> {
    fn in_public(&self)  where Self: VisibleInPub<V> {}
    fn in_crate(&self)   where Self: VisibleInCrate<V> {}
    fn in_other(&self)   where Self: VisibleInOther<V> {}
    fn in_private(&self) where Self: VisibleInPrivate<V> {}
}
```

Each marker's visibility matches the boundary it guards:

- `pub struct Pub`
- `pub(crate) struct Crate`
- `pub(in crate::other) struct Other`
- `struct Private`

The compiler has to resolve `V` to a concrete marker to check the bound, and *naming* that marker obeys visibility. So the gate enforces itself — the caller never has to mention the marker, but the compiler does on their behalf and fails if the name isn't reachable.

## Why the generic parameter matters

Drop `<V>` and write a plain bound like `Self: VisibleInCrate` — it breaks. Trait resolution doesn't care whether a trait name is visible; if the impl exists, the bound is satisfied. A private helper trait alone won't block anything.

The generic witness is what forces a concrete type to be named, and naming is the only thing visibility actually governs.

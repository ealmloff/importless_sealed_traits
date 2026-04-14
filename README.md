# Importless Sealed Traits

A variation on the sealed-trait pattern that seals **per-method at specific module boundaries** without requiring downstream callers to import any extra "sealed" trait.

For background on the conventional techniques, see Predrag's [Definitive Guide to Sealed Traits in Rust](https://predr.ag/blog/definitive-guide-to-sealed-traits-in-rust/).

## The trick

Instead of sealing the whole trait with a supertrait bound like `trait MyTrait: sealed::Sealed`, the trait carries a generic "visibility witness" parameter, and each method has a where-clause referencing a marker type whose visibility matches the intended boundary:

```rust
pub trait A<V> {
    fn in_public(&self)  where Self: VisibleInPub<V>     { }
    fn in_crate(&self)   where Self: VisibleInCrate<V>   { }
    fn in_other(&self)   where Self: VisibleInOther<V>   { }
    fn in_private(&self) where Self: VisibleInPrivate<V> { }
}
```

Each `VisibleInX` is a private helper trait, implemented only for a marker struct whose visibility is `pub`, `pub(crate)`, `pub(in ::path)`, or private. Rust's method resolution needs the bound to be satisfiable at the call site, and satisfying it requires the marker struct to be **nameable** from the caller's module. So a call like `W.in_crate()` only compiles inside the defining crate, `W.in_other()` only inside `crate::other`, and so on — even though the trait itself is fully public.

## How this compares to the classic patterns

The blog enumerates three main approaches. Each has a different tradeoff around imports, extensibility, and granularity:

| Pattern | Seals trait? | Seals individual methods? | Downstream needs to import a sealing trait? | Multiple boundaries (crate / module / private) in one trait? |
|---|---|---|---|---|
| **Supertrait seal** (`trait Foo: sealed::Sealed`) | Yes | No | No (but can't implement) | No |
| **Signature seal** (private type in method signature) | No — trait implementable, method uncallable | Yes, per method | Sometimes (type must be nameable) | Awkward |
| **Generic-arg seal** (private type as generic param) | Yes | Per impl/method | Yes, the marker must be in scope | Partially |
| **This crate (importless, per-method)** | Methods are individually gated; trait itself is open | **Yes, at arbitrary `pub(...)` granularity** | **No — only the public trait is imported** | **Yes, all four levels in a single trait** |

### Key differences

- **No import tax on callers.** Classic generic-argument sealing (e.g. the `Token`/`Sealer` style) requires users to `use` the sealing type before methods become callable. Here, callers just `use crate::other::A;` and the method set they see is automatically whatever their module's visibility allows.
- **Per-method, per-scope granularity.** A single trait can expose different methods to different visibility tiers (`pub`, `pub(crate)`, `pub(in path)`, private) simultaneously. The conventional patterns generally seal the whole trait (or require a separate trait per tier).
- **Error messages are about unresolved names**, not unsatisfied trait bounds — users see "cannot find type `Crate` in scope" style diagnostics, which can be less actionable than a direct "trait is sealed" error from a supertrait seal.
- **Downstream `impl` is still open.** Like the signature-seal variant, this does not prevent foreign implementations of the trait; it only restricts which methods can be *called* from a given scope. If you need to forbid foreign impls, combine with a supertrait seal.
- **Relies on `private_bounds`.** The methods reference private traits in their where-clauses, so the impl requires `#[allow(private_bounds)]`. This is stable but unusual.

## Why the generic witness is load-bearing

A natural simplification is to drop the `<V>` parameter and just give each sealer trait the visibility you want:

```rust
pub(crate) trait VisibleInCrate {}
impl VisibleInCrate for W {}

pub trait A {
    fn in_crate(&self) where Self: VisibleInCrate {}
}
```

**This does not seal anything.** Trait resolution ignores visibility: if an `impl` exists, the bound is satisfied regardless of whether the sealer trait is nameable at the call site. A downstream crate that can't even see `VisibleInCrate` will still happily call `W.in_crate()` (verified — it compiles).

Visibility polices *naming*, not trait resolution. The generic `<V>` works because it forces the caller into a position where they must name a type (the marker struct) — and that naming *is* subject to visibility. Without the generic, there is no name for the visibility check to reject.

## Demo

- `src/lib.rs` — defines the trait and exercises each boundary from inside and outside the private module.
- `inner/src/main.rs` — a downstream crate that can only see `in_public`.

Uncomment the gated lines in either file to see the compiler reject out-of-scope method calls.

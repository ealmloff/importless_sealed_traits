//! Downstream smoke tests for the `testing` crate's sealed trait.
//!
//! Only `in_public` is reachable from here:
//!
//! ```
//! use testing::other::{A, W};
//! W.in_public();
//! ```
//!
//! Each of the other methods is rejected, demonstrating the seal:
//!
//! ```compile_fail
//! use testing::other::{A, W};
//! W.in_crate();
//! ```
//!
//! ```compile_fail
//! use testing::other::{A, W};
//! W.in_other();
//! ```
//!
//! ```compile_fail
//! use testing::other::{A, W};
//! W.in_private();
//! ```

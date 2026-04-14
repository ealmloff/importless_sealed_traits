//! Importless sealed traits — per-method visibility gates without requiring
//! downstream callers to import a sealing type.
//!
//! From a downstream crate, only [`other::A::in_public`] is callable:
//!
//! ```
//! use testing::other::{A, W};
//! W.in_public();
//! ```
//!
//! All other methods are rejected:
//!
//! ```compile_fail
//! use testing::other::{A, W};
//! W.in_crate();
//! ```

use crate::other::{A, W};

fn test() {
    // fails because private is not in scope
    // W.in_private();
    // fails because other is not in scope
    //  W.in_other();
    W.in_crate();
    W.in_public();
}

/// Holds the public surface of the sealed trait.
///
/// Downstream can reach `in_public`:
///
/// ```
/// use testing::other::{A, W};
/// W.in_public();
/// ```
///
/// but not the inner-scoped methods:
///
/// ```compile_fail
/// use testing::other::{A, W};
/// W.in_private();
/// ```
///
/// ```compile_fail
/// use testing::other::{A, W};
/// W.in_other();
/// ```
///
/// ```compile_fail
/// use testing::other::{A, W};
/// W.in_crate();
/// ```
pub mod other {
    pub use crate::other::private::{A, W};

    pub fn test() {
        // Passes because other could be in scope, but private is not
        W.in_other();
        W.in_public();
    }

    mod private {
        fn test() {
            W.in_private();
        }

        trait VisibleInPrivate<T> {}
        impl VisibleInPrivate<Private> for W {}
        struct Private;

        trait VisibleInCrate<T> {}
        impl VisibleInCrate<Crate> for W {}
        pub(crate) struct Crate;

        trait VisibleInOther<T> {}
        impl VisibleInOther<Other> for W {}
        pub(in crate::other) struct Other;

        trait VisibleInPub<T> {}
        impl VisibleInPub<Pub> for W {}
        pub struct Pub;

        /// The witness type that carries the sealed impls.
        ///
        /// ```
        /// use testing::other::{A, W};
        /// let _ = W;
        /// W.in_public();
        /// ```
        pub struct W;

        /// Sealed trait with per-method visibility gates.
        ///
        /// The [`A::in_public`] method is callable from any crate; the other
        /// methods are gated to progressively narrower scopes.
        ///
        /// ```
        /// use testing::other::{A, W};
        /// W.in_public();
        /// ```
        ///
        /// ```compile_fail
        /// use testing::other::{A, W};
        /// W.in_crate();
        /// ```
        pub trait A<Visiblity> {
            /// Callable from anywhere the trait is in scope.
            ///
            /// ```
            /// use testing::other::{A, W};
            /// W.in_public();
            /// ```
            #[allow(private_bounds)]
            fn in_public(&self)
            where
                Self: VisibleInPub<Visiblity>,
            {
            }

            /// Callable only inside the defining private module.
            ///
            /// ```compile_fail
            /// use testing::other::{A, W};
            /// W.in_private();
            /// ```
            #[allow(private_bounds)]
            fn in_private(&self)
            where
                Self: VisibleInPrivate<Visiblity>,
            {
            }

            /// Callable only inside the defining crate.
            ///
            /// ```compile_fail
            /// use testing::other::{A, W};
            /// W.in_crate();
            /// ```
            #[allow(private_bounds)]
            fn in_crate(&self)
            where
                Self: VisibleInCrate<Visiblity>,
            {
            }

            /// Callable only inside `crate::other`.
            ///
            /// ```compile_fail
            /// use testing::other::{A, W};
            /// W.in_other();
            /// ```
            #[allow(private_bounds)]
            fn in_other(&self)
            where
                Self: VisibleInOther<Visiblity>,
            {
            }
        }

        impl<Visiblity> A<Visiblity> for W {}
    }
}

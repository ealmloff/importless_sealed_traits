use crate::other::{A, W};

fn test() {
    // fails because private is not in scope
    // W.in_private();
    // fails because other is not in scope
    //  W.in_other();
    W.in_crate();
    W.in_public();
}

/// Downstream seal: `in_public` is callable, everything else is not.
///
/// ```
/// use testing::other::{A, W};
/// W.in_public();
/// ```
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

        pub struct W;

        pub trait A<Visiblity> {
            #[allow(private_bounds)]
            fn in_public(&self)
            where
                Self: VisibleInPub<Visiblity>,
            {
            }

            #[allow(private_bounds)]
            fn in_private(&self)
            where
                Self: VisibleInPrivate<Visiblity>,
            {
            }

            #[allow(private_bounds)]
            fn in_crate(&self)
            where
                Self: VisibleInCrate<Visiblity>,
            {
            }

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

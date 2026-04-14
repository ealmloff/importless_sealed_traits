use testing::other::{A, W};

fn main() {
    W.in_public();
    // fails because crate is not in scope
    // W.in_crate();
    // fails because private is not in scope
    // W.in_private();
    // fails because other is not in scope
    //  W.in_other();
}

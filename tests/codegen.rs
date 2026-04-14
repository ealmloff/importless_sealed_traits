//! Generates synthetic crates that reproduce the sealing machinery and
//! exercise every (scope, method) combination, invoking `rustc` directly and
//! asserting the outcome matches the expected visibility table.
//!
//! Scopes correspond to the three internal tiers of the real `testing` crate:
//!   - `private` — inside the module that defines the markers
//!   - `other`   — inside `crate::outer` (analog of `crate::other`)
//!   - `root`    — at the generated crate root
//!
//! The fourth tier (downstream) is already covered by `compile_fail` doctests
//! in `src/lib.rs` and `inner/src/lib.rs`.

use std::process::Command;

const MACHINERY: &str = r#"
    trait VisibleInPrivate<T> {}
    impl VisibleInPrivate<Private> for W {}
    struct Private;

    trait VisibleInCrate<T> {}
    impl VisibleInCrate<Crate> for W {}
    pub(crate) struct Crate;

    trait VisibleInOther<T> {}
    impl VisibleInOther<Other> for W {}
    pub(in crate::outer) struct Other;

    trait VisibleInPub<T> {}
    impl VisibleInPub<Pub> for W {}
    pub struct Pub;

    pub struct W;

    pub trait A<V> {
        #[allow(private_bounds)] fn in_public(&self) where Self: VisibleInPub<V> {}
        #[allow(private_bounds)] fn in_private(&self) where Self: VisibleInPrivate<V> {}
        #[allow(private_bounds)] fn in_crate(&self) where Self: VisibleInCrate<V> {}
        #[allow(private_bounds)] fn in_other(&self) where Self: VisibleInOther<V> {}
    }
    impl<V> A<V> for W {}
"#;

#[derive(Copy, Clone, Debug)]
enum Scope {
    Private,
    Other,
    Root,
    Downstream,
}

fn lib_source(priv_body: &str, other_body: &str) -> String {
    format!(
        r#"
#![allow(dead_code, unused_imports)]
pub mod outer {{
    pub use self::inner::{{A, W}};
    fn in_other_scope() {{ {other_body} }}
    pub mod inner {{
        {MACHINERY}
        fn in_private_scope() {{ {priv_body} }}
    }}
}}
"#
    )
}

fn single_crate_source(scope: Scope, method: &str) -> String {
    let call = format!("W.{method}();");
    let (priv_body, other_body, root_body) = match scope {
        Scope::Private => (call.as_str(), "", ""),
        Scope::Other => ("", call.as_str(), ""),
        Scope::Root => ("", "", call.as_str()),
        Scope::Downstream => unreachable!(),
    };
    let lib = lib_source(priv_body, other_body);
    format!("{lib}\nuse crate::outer::{{A, W}};\nfn in_root_scope() {{ {root_body} }}\nfn main() {{}}\n")
}

fn downstream_consumer(method: &str) -> String {
    format!(
        r#"
use sealed_lib::outer::{{A, W}};
fn main() {{ W.{method}(); }}
"#
    )
}

fn workdir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("sealed_codegen_{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

const EDITIONS: &[&str] = &["2018", "2021", "2024"];

fn try_compile(name: &str, edition: &str, src: &str) -> (bool, String) {
    let dir = workdir(name);
    let src_path = dir.join("src.rs");
    let out_path = dir.join("out");
    std::fs::write(&src_path, src).unwrap();
    let output = Command::new("rustc")
        .arg(format!("--edition={edition}"))
        .arg("--crate-type=bin")
        .arg("-o")
        .arg(&out_path)
        .arg(&src_path)
        .output()
        .expect("failed to invoke rustc");
    (output.status.success(), String::from_utf8_lossy(&output.stderr).to_string())
}

fn try_compile_downstream(name: &str, edition: &str, method: &str) -> (bool, String) {
    let dir = workdir(name);
    let lib_path = dir.join("sealed_lib.rs");
    let consumer_path = dir.join("consumer.rs");
    std::fs::write(&lib_path, lib_source("", "")).unwrap();
    std::fs::write(&consumer_path, downstream_consumer(method)).unwrap();

    let lib_out = Command::new("rustc")
        .arg(format!("--edition={edition}"))
        .arg("--crate-type=rlib")
        .arg("--crate-name=sealed_lib")
        .arg("-o")
        .arg(dir.join("libsealed_lib.rlib"))
        .arg(&lib_path)
        .output()
        .expect("rustc (lib)");
    if !lib_out.status.success() {
        return (false, format!("lib build failed:\n{}", String::from_utf8_lossy(&lib_out.stderr)));
    }

    let output = Command::new("rustc")
        .arg(format!("--edition={edition}"))
        .arg("--crate-type=bin")
        .arg("--extern")
        .arg(format!("sealed_lib={}", dir.join("libsealed_lib.rlib").display()))
        .arg("-o")
        .arg(dir.join("consumer"))
        .arg(&consumer_path)
        .output()
        .expect("rustc (consumer)");
    (output.status.success(), String::from_utf8_lossy(&output.stderr).to_string())
}

#[test]
fn visibility_matrix() {
    // (scope, method) -> should compile
    let cases: &[(Scope, &str, bool)] = &[
        (Scope::Private, "in_public", true),
        (Scope::Private, "in_crate", true),
        (Scope::Private, "in_other", true),
        (Scope::Private, "in_private", true),
        (Scope::Other, "in_public", true),
        (Scope::Other, "in_crate", true),
        (Scope::Other, "in_other", true),
        (Scope::Other, "in_private", false),
        (Scope::Root, "in_public", true),
        (Scope::Root, "in_crate", true),
        (Scope::Root, "in_other", false),
        (Scope::Root, "in_private", false),
    ];

    let cases: Vec<_> = cases.iter().copied().chain([
        (Scope::Downstream, "in_public", true),
        (Scope::Downstream, "in_crate", false),
        (Scope::Downstream, "in_other", false),
        (Scope::Downstream, "in_private", false),
    ]).collect();

    let mut failures = Vec::new();
    for edition in EDITIONS {
        for &(scope, method, expected) in &cases {
            let name = format!("{edition}_{scope:?}_{method}").to_lowercase();
            let (ok, stderr) = match scope {
                Scope::Downstream => try_compile_downstream(&name, edition, method),
                _ => try_compile(&name, edition, &single_crate_source(scope, method)),
            };
            let src = match scope {
                Scope::Downstream => downstream_consumer(method),
                _ => single_crate_source(scope, method),
            };
            if ok != expected {
                failures.push(format!(
                    "edition={edition} scope={scope:?} method={method}: expected compile={expected}, got={ok}\nstderr:\n{stderr}\nsource:\n{src}"
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} case(s) failed:\n\n{}",
        failures.len(),
        failures.join("\n---\n")
    );
}

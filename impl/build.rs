use rustc_version::{version_meta, Channel};

#[cfg(not(feature = "testing-helpers"))]
fn detect_nightly() {}

#[cfg(feature = "testing-helpers")]
fn detect_nightly() {
    if version_meta().unwrap().channel == Channel::Nightly {
        println!("cargo:rustc-cfg=nightly");
    }
}

fn detect_error_generic_member_access() {
    // Error backtraces require the unstable generic member access API
    if version_meta().unwrap().channel == Channel::Nightly {
        println!("cargo:rustc-cfg=error_generic_member_access");
    }
}

fn main() {
    detect_nightly();
    detect_error_generic_member_access();
}

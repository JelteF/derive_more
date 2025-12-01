use rustc_version::{version_meta, Channel};

fn detect_error_generic_member_access() {
    // Error backtraces require `feature(error_generic_member_access)`.
    if version_meta().unwrap().channel == Channel::Nightly {
        println!("cargo:rustc-cfg=error_generic_member_access");
    }
}

#[cfg(not(feature = "testing-helpers"))]
fn detect_nightly() {}

#[cfg(feature = "testing-helpers")]
fn detect_nightly() {
    if version_meta().unwrap().channel == Channel::Nightly {
        println!("cargo:rustc-cfg=nightly");
    }
}

fn main() {
    detect_error_generic_member_access();
    detect_nightly();
}

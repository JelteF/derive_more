#[cfg(feature = "testing-helpers")]
extern crate rustc_version;

#[cfg(not(feature = "testing-helpers"))]
fn detect_nightly() {}

#[cfg(feature = "testing-helpers")]
fn detect_nightly() {
    use rustc_version::{version_meta, Channel};
    if version_meta().unwrap().channel == Channel::Nightly {
        println!("cargo:rustc-cfg=feature=\"nightly\"");
    }
}

#[cfg(not(feature = "unwrap"))]
fn detect_track_caller() {}
/// Detect availability of the `#[track_caller]` attribute for
/// use in derived panicking methods like `.unwrap_*()`.
#[cfg(feature = "unwrap")]
fn detect_track_caller() {
    use rustc_version::version_meta;
    if version_meta().unwrap().semver.minor >= 46 {
        println!("cargo:rustc-cfg=feature=\"track-caller\"");
    }
}

fn main() {
    detect_nightly();
    detect_track_caller();
}

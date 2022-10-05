#[cfg(not(feature = "testing-helpers"))]
fn detect_nightly() {}

#[cfg(feature = "testing-helpers")]
fn detect_nightly() {
    use rustc_version::{version_meta, Channel};

    if version_meta().unwrap().channel == Channel::Nightly {
        println!("cargo:rustc-cfg=feature=\"nightly\"");
    }
}

fn main() {
    detect_nightly();
}

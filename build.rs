extern crate rustc_version;

use rustc_version::{version_meta, Channel};

fn main() {
    if version_meta().channel == Channel::Nightly {
        println!("cargo:rustc-cfg=feature=\"nightly\"");
    }
}

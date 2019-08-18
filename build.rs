extern crate rustc_version;

#[cfg(feature = "generate-parsing-rs")]
extern crate peg;

use rustc_version::{version_meta, Channel};

#[cfg(not(feature = "generate-parsing-rs"))]
fn main() {
    if version_meta().unwrap().channel == Channel::Nightly {
        println!("cargo:rustc-cfg=feature=\"nightly\"");
    }
}

#[cfg(feature = "generate-parsing-rs")]
fn main() {
    if version_meta().unwrap().channel == Channel::Nightly {
        println!("cargo:rustc-cfg=feature=\"nightly\"");
    }

    let contents = match ::std::fs::read_to_string("src/parsing.rustpeg") {
        Ok(contents) => contents,
        Err(e) => panic!("{}", e),
    };

    let compiled = match ::peg::compile(&contents) {
        Ok(compiled) => compiled,
        Err(e) => panic!("{}", e),
    };

    if let Err(e) = ::std::fs::write("src/parsing.rs", compiled) {
        panic!("{}", e);
    }
}

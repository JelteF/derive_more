#[cfg(feature = "testing-helpers")]
extern crate rustc_version;

#[cfg(feature = "generate-parsing-rs")]
extern crate peg;


#[cfg(not(feature = "generate-parsing-rs"))]
fn generate_peg() {}
#[cfg(feature = "generate-parsing-rs")]
fn generate_peg() {
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
    generate_peg();
}

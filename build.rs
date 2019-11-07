#[cfg(feature = "generate-parsing-rs")]
extern crate peg;

#[cfg(not(feature = "generate-parsing-rs"))]
fn main() {}

#[cfg(feature = "generate-parsing-rs")]
fn main() {
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

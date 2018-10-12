#![allow(dead_code, unused_imports, unused_variables)]
#[macro_use]
extern crate derive_more;

use std::path::PathBuf;

// Here just to make sure that this doesn't conflict with
// the derives in some way
use std::fmt::Binary;

#[derive(Display)]
struct MyInt(i32);

#[derive(Display)]
#[display(fmt = "({}, {})", x, y)]
struct Point2D {
    x: i32,
    y: i32,
}

#[derive(Display)]
enum E {
    Uint(u32),
    #[display(fmt = "I am B {:b}", i)]
    Binary {
        i: i8,
    },
    #[cfg(feature = "nightly")]
    #[display(fmt = "I am C {}", "_0.display()")]
    Path(PathBuf),
}

#[derive(Display)]
#[display(fmt = "Java EE")]
enum EE {
    A,
    B,
}

#[derive(Display)]
#[display(fmt = "Hello there!")]
union U {
    i: u32,
}

#[derive(Octal)]
#[octal(fmt = "7")]
struct S;

#[derive(UpperHex)]
#[upper_hex(fmt = "UpperHex")]
struct UH;

#[derive(Display)]
struct Unit;

#[derive(Display)]
struct UnitStruct {}

#[derive(Display)]
enum EmptyEnum {}

#[test]
fn check_display() {
    assert_eq!(MyInt(-2).to_string(), "-2");
    assert_eq!(Point2D { x: 3, y: 4 }.to_string(), "(3, 4)");
    assert_eq!(E::Uint(2).to_string(), "2");
    assert_eq!(E::Binary { i: -2 }.to_string(), "I am B 11111110");
    #[cfg(feature = "nightly")]
    assert_eq!(E::Path("abc".into()).to_string(), "I am C abc");
    assert_eq!(EE::A.to_string(), "Java EE");
    assert_eq!(EE::B.to_string(), "Java EE");
    assert_eq!(U { i: 2 }.to_string(), "Hello there!");
    assert_eq!(format!("{:o}", S), "7");
    assert_eq!(format!("{:X}", UH), "UpperHex");
    assert_eq!(Unit.to_string(), "Unit");
    assert_eq!(UnitStruct {}.to_string(), "UnitStruct");
}

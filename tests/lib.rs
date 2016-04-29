#![feature(rustc_private, custom_derive, plugin)]
#![plugin(derive_more)]

extern crate syntax;
use syntax::codemap;

#[derive(Eq, PartialEq, Debug)]
#[derive(Add)]
struct MyInt(u32, i32);

impl <T> ::std::ops::Mul<T> for MyInt where
    T: Copy + Into<u32> + Into<i32> {

    type Output = MyInt;
    fn mul(self, rhs: T) -> MyInt {
        MyInt(Into::<u32>::into(rhs).mul(self.0), Into::<i32>::into(rhs).mul(self.1))
    }
}



#[derive(Eq, PartialEq, Debug)]
#[derive(Add, Mul)]
struct MyStruct{x:u32, y:u32}

enum MyEnum{
    Int(i32),
    Uint(u32),
}

impl <u32> ::std::ops::Mul<u32> for MyInt where
    type Output = MyInt;
    fn mul(self, rhs: u32) -> MyInt {
        match self {
            UInt(x) => UInt(x * rhs),
            _ => unreachable!();
        }
    }
}

impl <i32> ::std::ops::Mul<i32> for MyInt where
    type Output = MyInt;
    fn mul(self, rhs: i32) -> MyInt {
        match self {
            Int(x) => Int(x * rhs),
            _ => unreachable!();
        }
    }
}

#[test]
fn main() {
    assert_eq!(MyEnum::Int(5) * 8, (50, 60));
    assert_eq!(MyStruct{x:5, y:6} * 10, MyStruct{x:50, y:60});
}

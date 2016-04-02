# derive_more
Rust derive macros for some common traits for general types.

The traits that can be derived currently are `From` and infix arithmetic traits
(`Add`, `Sub`, `Mul`, `Div`, `Rem`, `BitAnd`, `BitOr`, `BitXor`).

## Installation

Add this to `Cargo.toml`:

```toml
[dependencies]
derive_more = "*"
```

And this to to top of your Rust file:
```
#![feature(rustc_private, custom_derive, plugin)]
#![plugin(derive_more)]
```

## Explanation of traits
This is a basic explanation of how the traits will be implemented, but the
example code below might do a better job explaining.

The arithmetic traits currently works for structs and enums.
For structs they simply do the respective operation on each pair of fields
separately.
For enums it will generate code that returns a `Result`, as adding different
enum options together will not work. When adding the same ones together the same
approach is done as for structs. Adding two unit types will result in an error.

The `From` trait only works for tuple structs with one element (newtypes) or
enums containing these tuple structs. The types wrapped by these tuple structs
can than simply be converted by using the `.into()` method.

For enums no from code will be generated for types that occur multiple times
since this would be ambiguous.

## Example usage
It can simply be used like this:

```rust
#![feature(rustc_private, custom_derive, plugin)]
#![plugin(derive_more)]

#[derive(From, Add)]
struct MyInt(i32);

#[derive(Add)]
struct NormalStruct{int1: u64, int2: u64}

#[derive(From, Add)]
enum MyIntEnum{
    Int(i32),
    Bool(bool),
    UnsignedOne(u32),
    UnsignedTwo(u32),
    Nothing,
}

```


The resulting code that will be compiled will look like this:

```rust
struct MyInt(i32);
impl ::std::convert::From<i32> for MyInt {
    fn from(a: i32) -> MyInt { MyInt(a) }
}
impl ::std::ops::Add for MyInt {
    type Output = MyInt;
    fn add(self, rhs: MyInt) -> MyInt {
        MyInt(self.0.add(rhs.0))
    }
}


struct NormalStruct{int1: u64, int2: u64}

impl ::std::ops::Add for NormalStruct {
    type Output = NormalStruct;
    fn add(self, rhs: NormalStruct) -> NormalStruct {
        NormalStruct{int1: self.int1.add(rhs.int1),
                     int2: self.int2.add(rhs.int2),}
    }
}


enum MyIntEnum {
    SmallInt(i32),
    BigInt(i64),
    TwoInts(i32, i32),
    UnsignedOne(u32),
    UnsignedTwo(u32),
    Nothing,
}

impl ::std::convert::From<i32> for MyIntEnum {
    fn from(a: i32) -> MyIntEnum { MyIntEnum::SmallInt(a) }
}

impl ::std::convert::From<i64> for MyIntEnum {
    fn from(a: i64) -> MyIntEnum { MyIntEnum::BigInt(a) }
}

impl ::std::ops::Add for MyIntEnum {
    type Output = Result<MyIntEnum, &'static str>;

    fn add(self, rhs: MyIntEnum) -> Result<MyIntEnum, &'static str> {
        match (self, rhs) {
            (MyIntEnum::SmallInt(__l_0), MyIntEnum::SmallInt(__r_0)) => Ok(MyIntEnum::SmallInt(__l_0.add(__r_0))),
            (MyIntEnum::BigInt(__l_0), MyIntEnum::BigInt(__r_0)) => Ok(MyIntEnum::BigInt(__l_0.add(__r_0))),

            (MyIntEnum::TwoInts(__l_0, __l_1),
             MyIntEnum::TwoInts(__r_0, __r_1)) => Ok(MyIntEnum::TwoInts(__l_0.add(__r_0), __l_1.add(__r_1))),

            (MyIntEnum::UnsignedOne(__l_0), MyIntEnum::UnsignedOne(__r_0)) => Ok(MyIntEnum::UnsignedOne(__l_0.add(__r_0))),
            (MyIntEnum::UnsignedTwo(__l_0), MyIntEnum::UnsignedTwo(__r_0)) => Ok(MyIntEnum::UnsignedTwo(__l_0.add(__r_0))),

            (MyIntEnum::Nothing, MyIntEnum::Nothing) => Err("Cannot add unit types together"),
            _ => Err("Trying to add mismatched enum types"),
        }
    }
}

```

Because of this and Rust its built in type inference the following can be done
now:

```rust
fn main() {
    let my_enum_val = (MyIntEnum::SmallInt(5) + 6.into()).unwrap();

    // To do this Eq and PartialEq also need to be derived
    if en_enum_val == 5.into() {
        println!("The content of my_enum_val is 5")
    }
}
```

For more usage examples look at the [tests](https://github.com/JelteF/derive_more/blob/master/tests/lib.rs).


## Licence

MIT

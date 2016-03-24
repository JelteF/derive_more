# derive_from
Rust derive(From) macro

It only works for tuple structs with one element (newtypes) or enums containing
these tuple structs. The types wrapped by these tuple structs can than simply be
converted by using the `.into()` method.

For enums no from code will be generated for types that occur multiple times
since this would be ambiguous.

It can simply be used like this:

```rust
#[derive(From)]
struct MyInt(i32);

#[derive(From)]
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


enum MyIntEnum {
    Int(i32),
    Bool(bool),
    UnsignedOne(u32),
    UnsignedTwo(u32),
    Nothing,
}
impl ::std::convert::From<i32> for MyIntEnum {
    fn from(a: i32) -> MyIntEnum { MyIntEnum::Int(a) }
}
impl ::std::convert::From<bool> for MyIntEnum {
    fn from(a: bool) -> MyIntEnum { MyIntEnum::Bool(a) }
}
```

Because of this and Rust its built in type inference the following can be done
now:

```rust
fn main() {
    let my_enum_val = MyIntEnum::Int(5);

    if en_enum_val == 5.into() {
        println!("The content of my_enum_val is 5")
    }
}
```

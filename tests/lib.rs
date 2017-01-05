#[macro_use]
extern crate derive_more;

#[derive(From)]
//#[derive(Eq, PartialEq, Debug)]
// #[derive(Add)]
struct MyInt(i32);

// #[derive(Add)]
// #[derive(Eq, PartialEq, Debug)]
// struct MyUInt(u64, u64);
// 
// #[derive(Add, Sub, Mul, Div, Rem, BitAnd, BitOr, BitXor, Shr, Shl)]
// #[derive(Eq, PartialEq, Debug, Clone, Copy)]
// struct NormalStruct{int1: u64, int2: u64}
// 
// #[derive(From)]
// //#[derive(Eq, PartialEq, Debug)]
// struct NestedInt(MyInt);
// 
// #[derive(From)]
// //#[derive(Eq, PartialEq, Debug)]
// struct MySpan(codemap::Span);
// 
// #[derive(From)]
// //#[derive(Eq, PartialEq, Debug)]
// struct MySpan2(::syntax::codemap::Span);
// 
// #[derive(Eq, PartialEq, Debug)]
// #[derive(From)]
// #[derive(Add,Sub)]
// enum SimpleMyIntEnum{
//     Int(i32),
//     UnsignedOne(u32),
//     UnsignedTwo(u32),
// }
// 
// #[derive(Eq, PartialEq, Debug)]
// #[derive(From)]
// #[derive(Add)]
// enum SimpleEnum{
//     Int(i32),
//     Ints(i32, i32),
//     LabeledInts{a: i32, b: i32},
//     SomeUnit,
// }
// 
// #[derive(Eq, PartialEq, Debug)]
// #[derive(From, Add)]
// enum MyIntEnum{
//     SmallInt(i32),
//     BigInt(i64),
//     TwoInts(i32, i32),
//     UnsignedOne(u32),
//     UnsignedTwo(u32),
//     Nothing,
// }
// 
// #[derive(Eq, PartialEq, Debug)]
// #[derive(Add, Mul)]
// struct DoubleUInt(u32, u32);
// 
// #[derive(Eq, PartialEq, Debug)]
// #[derive(Add, Mul)]
// struct DoubleUIntStruct{x:u32, y:u32}


#[test]
fn main() {
    // assert_eq!(MyInt(5), 5.into());
    // assert_eq!(MyIntEnum::SmallInt(5), 5.into());

    // assert_eq!(MyInt(4) + MyInt(1), 5.into());
    // assert_eq!(MyUInt(4, 5) + MyUInt(1, 2), MyUInt(5, 7));
    // let s1 = NormalStruct{int1: 1, int2: 2};
    // let s2 = NormalStruct{int1: 2, int2: 3};
    // let s3 = NormalStruct{int1: 3, int2: 5};
    // assert_eq!(s1 + s2, s3);
    // assert_eq!(s3 - s2, s1);
    // assert_eq!((SimpleMyIntEnum::Int(6) + 5.into()).unwrap(), 11.into());
    // assert_eq!((SimpleMyIntEnum::Int(6) - 5.into()).unwrap(), 1.into());
    // assert_eq!((SimpleEnum::Ints(6, 5) + SimpleEnum::Ints(1, 4)).unwrap(), SimpleEnum::Ints(7, 9));
    // assert_eq!((SimpleEnum::LabeledInts{a: 6, b: 5} + SimpleEnum::LabeledInts{a: 1, b: 4}).unwrap(),
    //            SimpleEnum::LabeledInts{a: 7, b: 9});

    // let my_enum_val = (MyIntEnum::SmallInt(5) + 6.into()).unwrap();

    // assert_eq!(DoubleUInt(5, 6) * 10, DoubleUInt(50, 60));
    // assert_eq!(DoubleUIntStruct{x:5, y:6} * 10, DoubleUIntStruct{x:50, y:60});
}

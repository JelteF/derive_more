#![allow(dead_code)]

#[macro_use]
extern crate derive_more;

use std::convert::{TryFrom, TryInto};

// Ensure that the TryFrom macro is hygenic and doesn't break when `Result` has
// been redefined.
type Result = ();

#[derive(Clone, Copy, TryInto, Eq, PartialEq, Debug)]
#[try_into(owned, ref, ref_mut)]
enum MixedInts {
    SmallInt(i32),
    NamedBigInt {
        int: i64,
    },
    UnsignedWithIgnoredField(#[try_into(ignore)] bool, i64),
    NamedUnsignedWithIgnnoredField {
        #[try_into(ignore)]
        useless: bool,
        x: i64,
    },
    TwoSmallInts(i32, i32),
    NamedBigInts {
        x: i64,
        y: i64,
    },
    Unsigned(u32),
    NamedUnsigned {
        x: u32,
    },
    Unit,
    #[try_into(ignore)]
    Unit2,
}

#[test]
fn test_try_into() {
    let mut i = MixedInts::SmallInt(42);
    assert_eq!(Ok(42i32), i.try_into());
    assert_eq!(Ok(&42i32), (&i).try_into());
    assert_eq!(Ok(&mut 42i32), (&mut i).try_into());
    assert_eq!(i64::try_from(i), Err(i),);
    assert_eq!(<(i32, i32)>::try_from(i), Err(i),);
    assert_eq!(<(i64, i64)>::try_from(i), Err(i),);
    assert_eq!(u32::try_from(i), Err(i));
    assert_eq!(<()>::try_from(i), Err(i));

    let mut i = MixedInts::NamedBigInt { int: 42 };
    assert_eq!(i32::try_from(i), Err(i));
    assert_eq!(Ok(42i64), i.try_into());
    assert_eq!(Ok(&42i64), (&i).try_into());
    assert_eq!(Ok(&mut 42i64), (&mut i).try_into());
    assert_eq!(<(i32, i32)>::try_from(i), Err(i));
    assert_eq!(<(i64, i64)>::try_from(i), Err(i));
    assert_eq!(u32::try_from(i), Err(i));
    assert_eq!(<()>::try_from(i), Err(i));

    let mut i = MixedInts::TwoSmallInts(42, 64);
    assert_eq!(i32::try_from(i), Err(i),);
    assert_eq!(i64::try_from(i), Err(i));
    assert_eq!(Ok((42i32, 64i32)), i.try_into());
    assert_eq!(Ok((&42i32, &64i32)), (&i).try_into());
    assert_eq!(Ok((&mut 42i32, &mut 64i32)), (&mut i).try_into());
    assert_eq!(<(i64, i64)>::try_from(i), Err(i));
    assert_eq!(u32::try_from(i), Err(i));
    assert_eq!(<()>::try_from(i), Err(i));

    let mut i = MixedInts::NamedBigInts { x: 42, y: 64 };
    assert_eq!(i32::try_from(i), Err(i));
    assert_eq!(i64::try_from(i), Err(i));
    assert_eq!(<(i32, i32)>::try_from(i), Err(i));
    assert_eq!(Ok((42i64, 64i64)), i.try_into());
    assert_eq!(Ok((&42i64, &64i64)), (&i).try_into());
    assert_eq!(Ok((&mut 42i64, &mut 64i64)), (&mut i).try_into());
    assert_eq!(u32::try_from(i), Err(i));
    assert_eq!(<()>::try_from(i), Err(i));

    let mut i = MixedInts::Unsigned(42);
    assert_eq!(i32::try_from(i), Err(i));
    assert_eq!(i64::try_from(i), Err(i));
    assert_eq!(<(i32, i32)>::try_from(i), Err(i));
    assert_eq!(<(i64, i64)>::try_from(i), Err(i));
    assert_eq!(Ok(42u32), i.try_into());
    assert_eq!(Ok(&42u32), (&i).try_into());
    assert_eq!(Ok(&mut 42u32), (&mut i).try_into());
    assert_eq!(<()>::try_from(i), Err(i));

    let mut i = MixedInts::NamedUnsigned { x: 42 };
    assert_eq!(i32::try_from(i), Err(i));
    assert_eq!(i64::try_from(i), Err(i),);
    assert_eq!(i64::try_from(i), Err(i),);
    assert_eq!(<(i32, i32)>::try_from(i), Err(i),);
    assert_eq!(<(i64, i64)>::try_from(i), Err(i),);
    assert_eq!(Ok(42u32), i.try_into());
    assert_eq!(Ok(&42u32), (&i).try_into());
    assert_eq!(Ok(&mut 42u32), (&mut i).try_into());
    assert_eq!(<()>::try_from(i), Err(i));

    let i = MixedInts::Unit;
    assert_eq!(i32::try_from(i), Err(i),);
    assert_eq!(i64::try_from(i), Err(i),);
    assert_eq!(<(i32, i32)>::try_from(i), Err(i),);
    assert_eq!(<(i64, i64)>::try_from(i), Err(i));
    assert_eq!(u32::try_from(i), Err(i));
    assert_eq!(Ok(()), i.try_into());
}

#[derive(TryInto, Debug)]
#[try_into(owned, ref, ref_mut)]
enum Something {
    A(A),
    B(B),
}

#[derive(Debug)]
struct A {
    value: u8,
    valid: bool,
}

#[derive(Debug)]
struct B {
    value: u64,
    valid: bool,
}

trait IsValid {
    fn is_valid(self) -> bool;
}

impl IsValid for A {
    fn is_valid(self) -> bool {
        self.valid
    }
}

impl IsValid for B {
    fn is_valid(self) -> bool {
        self.valid
    }
}

impl IsValid for Something {
    fn is_valid(self: Something) -> bool {
        match A::try_from(self) {
            Ok(a) => a.is_valid(),
            Err(x) => match B::try_from(x) {
                Ok(b) => b.is_valid(),
                Err(_) => false,
            },
        }
    }
}

#[test]
fn test_try_from_chain() {
    let a = Something::A(A {
        value: 1,
        valid: true,
    });
    let b = Something::B(B {
        value: 2,
        valid: true,
    });
    let c = Something::A(A {
        value: 3,
        valid: false,
    });
    assert!(a.is_valid());
    assert!(b.is_valid());
    assert!(!c.is_valid());
}

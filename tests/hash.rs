#![cfg_attr(not(feature = "std"), no_std)]

mod hash_utils;
use hash_utils::do_hash;

pub mod utils {
    pub fn alternate_u32_hash_function<H: core::hash::Hasher>(
        value: &u32,
        state: &mut H,
    ) {
        state.write_u32(42);
        state.write_u32(*value)
    }
}

mod structs {
    mod single_field {
        use crate::do_hash;
        use derive_more::Hash;

        #[derive(Hash)]
        struct Tuple(i32);

        #[derive(Hash)]
        struct Struct {
            field: i32,
        }

        #[derive(Hash)]
        struct StructSkipped {
            #[hash(skip)]
            _skipped: i32,
        }

        #[derive(Hash)]
        #[hash(skip)]
        struct StructFullySkipped {
            _skipped: i32,
        }

        #[test]
        fn assert() {
            assert_eq!(do_hash(&Tuple(42)), do_hash(&42));
            assert_eq!(do_hash(&Struct { field: 42 }), do_hash(&42));
            assert_eq!(do_hash(&StructSkipped { _skipped: 42 }), do_hash(&()));
            assert_eq!(do_hash(&StructFullySkipped { _skipped: 42 }), do_hash(&()));
        }
    }

    mod multi_field {
        use super::super::utils;
        use crate::do_hash;
        use derive_more::Hash;

        #[derive(Hash)]
        struct MultiTuple(i32, &'static str, bool);

        #[derive(Hash)]
        struct MultiStruct {
            a: i32,
            b: &'static str,
            c: bool,
        }

        #[derive(Hash)]
        struct StructWithAlternateHashFunction {
            #[hash(with(utils::alternate_u32_hash_function))]
            a: u32,
            b: &'static str,
            c: bool,
        }

        #[derive(Hash)]
        struct MixedSkip {
            field1: i32,
            #[hash(skip)]
            _skipped: &'static str,
            field2: bool,
        }

        #[test]
        fn assert() {
            assert_eq!(
                do_hash(&MultiTuple(42, "test", true)),
                do_hash(&(42, "test", true))
            );
            assert_eq!(
                do_hash(&MultiStruct {
                    a: 42,
                    b: "test",
                    c: true
                }),
                do_hash(&(42, "test", true))
            );
            assert_eq!(
                do_hash(&StructWithAlternateHashFunction {
                    a: 42,
                    b: "test",
                    c: true
                }),
                do_hash(&(42, 42, "test", true))
            );
            assert_eq!(
                do_hash(&MixedSkip {
                    field1: 42,
                    _skipped: "ignored",
                    field2: true
                }),
                do_hash(&(42, true))
            );
        }
    }

    mod generics {
        use crate::do_hash;
        use derive_more::Hash;

        trait SomeTraitWithTypes {
            type TraitType;
        }

        #[derive(Hash)]
        struct GenericStruct<T: SomeTraitWithTypes, U> {
            a: T::TraitType,
            b: U,
        }

        // this struct doesn't implement `Hash` but implements `SomeTraitWithTypes` with `TraitType = i32`
        // this means that `GenericStruct<SomeTraitWithTypesImpl>` should be hashable as well
        struct SomeTraitWithTypesImpl;
        impl SomeTraitWithTypes for SomeTraitWithTypesImpl {
            type TraitType = i32;
        }

        #[test]
        fn assert() {
            let instance: GenericStruct<SomeTraitWithTypesImpl, _> =
                GenericStruct { a: 42, b: "test" };
            assert_eq!(do_hash(&instance), do_hash(&(42, "test")));
        }
    }
}

mod enums {
    use super::utils;
    use crate::do_hash;
    use derive_more::Hash;

    #[derive(Hash)]
    enum SimpleEnum {
        A,
        B,
        C,
    }

    #[derive(Hash)]
    enum TupleEnum {
        A(i32),
        B(&'static str, bool),
        C,
    }

    #[derive(Hash)]
    enum StructEnum {
        A { x: i32 },
        B { y: &'static str, z: bool },
        C,
    }

    #[derive(Hash)]
    enum WithAndSkip {
        A {
            field: i32,
            #[hash(skip)]
            _skipped: &'static str,
        },
        B(
            #[hash(with(utils::alternate_u32_hash_function))] u32,
            #[hash(skip)]
            #[allow(unused)]
            &'static str,
        ),
        #[hash(skip)]
        #[allow(unused)]
        C(i32),
    }

    #[test]
    fn assert() {
        assert_eq!(
            do_hash(&SimpleEnum::A),
            do_hash(&core::mem::discriminant(&SimpleEnum::A))
        );
        assert_eq!(
            do_hash(&SimpleEnum::B),
            do_hash(&core::mem::discriminant(&SimpleEnum::B))
        );
        assert_eq!(
            do_hash(&SimpleEnum::C),
            do_hash(&core::mem::discriminant(&SimpleEnum::C))
        );
        let ta = TupleEnum::A(42);
        let tb = TupleEnum::B("test", true);
        assert_eq!(do_hash(&ta), do_hash(&(core::mem::discriminant(&ta), 42)));
        assert_eq!(
            do_hash(&tb),
            do_hash(&(core::mem::discriminant(&tb), "test", true))
        );
        let tc = TupleEnum::C;
        assert_eq!(do_hash(&tc), do_hash(&core::mem::discriminant(&tc)));

        let sa = StructEnum::A { x: 42 };
        assert_eq!(do_hash(&sa), do_hash(&(core::mem::discriminant(&sa), 42)));

        let sb = StructEnum::B { y: "test", z: true };
        assert_eq!(
            do_hash(&sb),
            do_hash(&(core::mem::discriminant(&sb), "test", true))
        );

        let sc = StructEnum::C;
        assert_eq!(do_hash(&sc), do_hash(&core::mem::discriminant(&sc)));

        let wa = WithAndSkip::A {
            field: 42,
            _skipped: "ignored",
        };
        assert_eq!(do_hash(&wa), do_hash(&(core::mem::discriminant(&wa), 42)));

        let wb = WithAndSkip::B(42, "ignored");
        assert_eq!(
            do_hash(&wb),
            do_hash(&(core::mem::discriminant(&wb), 42, 42))
        );

        let wc = WithAndSkip::C(42);
        assert_eq!(do_hash(&wc), do_hash(&core::mem::discriminant(&wc)));
    }
}

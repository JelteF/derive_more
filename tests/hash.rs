#![cfg_attr(not(feature = "std"), no_std)]

fn do_hash<T: std::hash::Hash>(t: &T) -> u64 {
    use std::hash::{DefaultHasher, Hasher};
    let mut hasher = DefaultHasher::default();
    t.hash(&mut hasher);
    hasher.finish()
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
        use crate::do_hash;
        use derive_more::Hash;

        #[derive(Hash)]
        struct MultiTuple(i32, String, bool);

        #[derive(Hash)]
        struct MultiStruct {
            a: i32,
            b: String,
            c: bool,
        }

        #[derive(Hash)]
        struct MixedSkip {
            field1: i32,
            #[hash(skip)]
            _skipped: String,
            field2: bool,
        }

        #[test]
        fn assert() {
            assert_eq!(
                do_hash(&MultiTuple(42, "test".to_string(), true)),
                do_hash(&(42, "test".to_string(), true))
            );
            assert_eq!(
                do_hash(&MultiStruct {
                    a: 42,
                    b: "test".to_string(),
                    c: true
                }),
                do_hash(&(42, "test".to_string(), true))
            );
            assert_eq!(
                do_hash(&MixedSkip {
                    field1: 42,
                    _skipped: "ignored".to_string(),
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
            let instance: GenericStruct<SomeTraitWithTypesImpl, _> = GenericStruct {
                a: 42,
                b: "test".to_string(),
            };
            assert_eq!(do_hash(&instance), do_hash(&(42, "test".to_string())));
        }
    }
}

mod enums {
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
        B(String, bool),
        C,
    }

    #[derive(Hash)]
    enum StructEnum {
        A { x: i32 },
        B { y: String, z: bool },
        C,
    }

    #[derive(Hash)]
    enum WithSkip {
        A {
            field: i32,
            #[hash(skip)]
            _skipped: String,
        },
        B(
            i32,
            #[hash(skip)]
            #[allow(unused)]
            String,
        ),
        #[hash(skip)]
        #[allow(unused)]
        C(i32),
    }

    #[test]
    fn assert() {
        assert_eq!(
            do_hash(&SimpleEnum::A),
            do_hash(&std::mem::discriminant(&SimpleEnum::A))
        );
        assert_eq!(
            do_hash(&SimpleEnum::B),
            do_hash(&std::mem::discriminant(&SimpleEnum::B))
        );
        assert_eq!(
            do_hash(&SimpleEnum::C),
            do_hash(&std::mem::discriminant(&SimpleEnum::C))
        );
        let ta = TupleEnum::A(42);
        let tb = TupleEnum::B("test".to_string(), true);
        assert_eq!(do_hash(&ta), do_hash(&(std::mem::discriminant(&ta), 42)));
        assert_eq!(
            do_hash(&tb),
            do_hash(&(std::mem::discriminant(&tb), "test".to_string(), true))
        );
        let tc = TupleEnum::C;
        assert_eq!(do_hash(&tc), do_hash(&std::mem::discriminant(&tc)));

        let sa = StructEnum::A { x: 42 };
        assert_eq!(do_hash(&sa), do_hash(&(std::mem::discriminant(&sa), 42)));

        let sb = StructEnum::B {
            y: "test".to_string(),
            z: true,
        };
        assert_eq!(
            do_hash(&sb),
            do_hash(&(std::mem::discriminant(&sb), "test".to_string(), true))
        );

        let sc = StructEnum::C;
        assert_eq!(do_hash(&sc), do_hash(&std::mem::discriminant(&sc)));

        let wa = WithSkip::A {
            field: 42,
            _skipped: "ignored".to_string(),
        };
        assert_eq!(do_hash(&wa), do_hash(&(std::mem::discriminant(&wa), 42)));

        let wb = WithSkip::B(42, "ignored".to_string());
        assert_eq!(do_hash(&wb), do_hash(&(std::mem::discriminant(&wb), 42)));

        let wc = WithSkip::C(42);
        assert_eq!(do_hash(&wc), do_hash(&std::mem::discriminant(&wc)));

    }
}
#![cfg_attr(not(feature = "std"), no_std)]

mod structs {

    use derive_more::Clone;
    #[test]
    fn reference_to_generic_type() {
        struct NotClone;

        #[derive(Clone)]
        struct ReferenceToGenericType<'a, T>(&'a T);

        let should_be_clonable = ReferenceToGenericType(&NotClone);
        let clone = should_be_clonable.clone();

        assert!(core::ptr::eq(should_be_clonable.0, clone.0))
    }

    #[test]
    fn manual_bound() {
        trait Foo {
            type Bar;
        }

        #[derive(Clone)]
        #[clone(bound(T::Bar: Clone))]
        struct Baz<T: Foo>(T::Bar);

        // intentionally not clone
        struct FooImpl;

        impl Foo for FooImpl {
            type Bar = i32;
        }

        let baz: Baz<FooImpl> = Baz(42);
        let clone = baz.clone();
        assert_eq!(baz.0, clone.0);
    }
}

mod enums {
    use derive_more::Clone;

    #[test]
    fn simple_enum() {
        #[derive(Clone, PartialEq, Debug)]
        enum Simple {
            A,
            B,
            C,
        }

        let a = Simple::A;
        let clone = a.clone();
        assert_eq!(a, clone);
    }

    #[test]
    fn enum_with_data() {
        #[derive(Clone, PartialEq, Debug)]
        enum WithData {
            Unit,
            Tuple(i32, &'static str),
            Struct { x: i32, y: i32 },
        }

        let tuple = WithData::Tuple(42, "hello");
        let clone = tuple.clone();
        assert_eq!(tuple, clone);

        let struct_variant = WithData::Struct { x: 1, y: 2 };
        let clone = struct_variant.clone();
        assert_eq!(struct_variant, clone);
    }

    #[test]
    fn reference_to_generic_type() {
        struct NotClone;

        #[derive(Clone)]
        enum ReferenceToGenericType<'a, T> {
            Ref(&'a T),
            None,
        }

        let should_be_clonable = ReferenceToGenericType::Ref(&NotClone);
        let clone = should_be_clonable.clone();

        match (&should_be_clonable, &clone) {
            (ReferenceToGenericType::Ref(a), ReferenceToGenericType::Ref(b)) => {
                assert!(core::ptr::eq(*a, *b));
            }
            _ => panic!("unexpected variant"),
        }
    }

    #[test]
    fn manual_bound() {
        trait Foo {
            type Bar;
        }

        #[derive(Clone)]
        #[clone(bound(T::Bar: Clone))]
        enum Baz<T: Foo> {
            Value(T::Bar),
            Empty,
        }

        struct FooImpl;

        impl Foo for FooImpl {
            type Bar = i32;
        }

        let baz: Baz<FooImpl> = Baz::Value(42);
        let clone = baz.clone();
        match (baz, clone) {
            (Baz::Value(a), Baz::Value(b)) => assert_eq!(a, b),
            _ => panic!("unexpected variant"),
        }
    }
}

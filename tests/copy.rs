#![cfg_attr(not(feature = "std"), no_std)]

mod structs {
    use derive_more::{Clone, Copy};

    #[test]
    fn simple_struct() {
        #[derive(Clone, Copy, PartialEq, Debug)]
        struct Simple(i32);

        let a = Simple(42);
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn reference_to_generic_type() {
        struct NotCopy;

        #[derive(Clone, Copy)]
        struct ReferenceToGenericType<'a, T>(&'a T);

        let not_copy = NotCopy;
        let should_be_copyable = ReferenceToGenericType(&not_copy);
        let copy = should_be_copyable;

        assert!(core::ptr::eq(should_be_copyable.0, copy.0))
    }

    #[test]
    fn manual_bound() {
        trait Foo {
            type Bar;
        }

        #[derive(Clone, Copy)]
        #[copy(bound(T::Bar: Copy))]
        #[clone(bound(T::Bar: Clone))]
        struct Baz<T: Foo>(T::Bar);

        struct FooImpl;

        impl Foo for FooImpl {
            type Bar = i32;
        }

        let baz: Baz<FooImpl> = Baz(42);
        let copy = baz;
        assert_eq!(baz.0, copy.0);
    }

    #[test]
    fn multiple_fields() {
        #[derive(Clone, Copy, PartialEq, Debug)]
        struct MultipleFields {
            x: i32,
            y: i64,
            z: u8,
        }

        let a = MultipleFields { x: 1, y: 2, z: 3 };
        let b = a;
        assert_eq!(a, b);
    }
}

mod enums {
    use derive_more::{Clone, Copy};

    #[test]
    fn simple_enum() {
        #[derive(Clone, Copy, PartialEq, Debug)]
        enum Simple {
            A,
            B,
            C,
        }

        let a = Simple::A;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn enum_with_data() {
        #[derive(Clone, Copy, PartialEq, Debug)]
        enum WithData {
            Unit,
            Tuple(i32, i64),
            Struct { x: i32, y: i32 },
        }

        let tuple = WithData::Tuple(42, 100);
        let copy = tuple;
        assert_eq!(tuple, copy);

        let struct_variant = WithData::Struct { x: 1, y: 2 };
        let copy = struct_variant;
        assert_eq!(struct_variant, copy);
    }

    #[test]
    fn reference_to_generic_type() {
        struct NotCopy;

        #[derive(Clone, Copy)]
        enum ReferenceToGenericType<'a, T> {
            Ref(&'a T),
            None,
        }

        let not_copy = NotCopy;
        let should_be_copyable = ReferenceToGenericType::Ref(&not_copy);
        let copy = should_be_copyable;

        match (&should_be_copyable, &copy) {
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

        #[derive(Clone, Copy)]
        #[copy(bound(T::Bar: Copy))]
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
        let copy = baz;
        match (baz, copy) {
            (Baz::Value(a), Baz::Value(b)) => assert_eq!(a, b),
            _ => panic!("unexpected variant"),
        }
    }
}

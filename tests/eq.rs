#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)] // some code is tested for type checking only

mod structs {
    mod structural {
        use derive_more::{Eq, __private::AssertParamIsEq};

        #[test]
        fn unit() {
            #[derive(Eq, PartialEq)]
            struct Baz;

            let _: AssertParamIsEq<Baz>;
        }

        #[test]
        fn empty_tuple() {
            #[derive(Eq, PartialEq)]
            struct Foo();

            let _: AssertParamIsEq<Foo>;
        }

        #[test]
        fn empty_struct() {
            #[derive(Eq, PartialEq)]
            struct Bar {}

            let _: AssertParamIsEq<Bar>;
        }

        #[test]
        fn multi_field_tuple() {
            #[derive(Eq, PartialEq)]
            struct Foo(bool, i32);

            let _: AssertParamIsEq<Foo>;
        }

        #[test]
        fn multi_field_struct() {
            #[derive(Eq, PartialEq)]
            struct Bar {
                b: bool,
                i: i32,
            }

            let _: AssertParamIsEq<Bar>;
        }

        mod generic {
            use derive_more::{Eq, PartialEq, __private::AssertParamIsEq};

            trait Some {
                type Assoc;
            }

            impl<T> Some for T {
                type Assoc = bool;
            }

            #[test]
            fn multi_field_tuple() {
                #[derive(Eq, PartialEq)]
                struct Foo<A: Some, B>(A::Assoc, B);

                let _: AssertParamIsEq<Foo<f32, ()>>;
            }

            #[test]
            fn multi_field_struct() {
                #[derive(Eq, PartialEq)]
                struct Bar<A, B: Some> {
                    b: B::Assoc,
                    i: A,
                }

                let _: AssertParamIsEq<Bar<u8, f32>>;
            }

            #[test]
            fn lifetime() {
                #[derive(Eq, PartialEq)]
                struct Foo<'a>(&'a str, i32);

                #[derive(Eq, PartialEq)]
                struct Bar<'a> {
                    b: Foo<'a>,
                    i: i32,
                }

                let _: AssertParamIsEq<Foo>;
                let _: AssertParamIsEq<Bar>;
            }

            #[test]
            fn const_param() {
                #[derive(Eq, PartialEq)]
                struct Baz<const N: usize>;

                #[derive(Eq, PartialEq)]
                struct Foo<const N: usize>([i32; N], i8);

                #[derive(Eq, PartialEq)]
                struct Bar<const N: usize> {
                    b: Foo<N>,
                    i: Baz<N>,
                }

                let _: AssertParamIsEq<Baz<1>>;
                let _: AssertParamIsEq<Foo<2>>;
                let _: AssertParamIsEq<Bar<3>>;
            }

            #[test]
            fn mixed() {
                #[derive(Eq, PartialEq)]
                struct Foo<'a, T, const N: usize>([&'a T; N]);

                let _: AssertParamIsEq<Foo<i32, 1>>;
            }
        }
    }
}

mod enums {
    mod structural {
        use derive_more::{Eq, __private::AssertParamIsEq};

        #[test]
        fn empty() {
            #[derive(Eq, PartialEq)]
            enum E {}

            let _: AssertParamIsEq<E>;
        }

        #[test]
        fn single_variant_unit() {
            #[derive(Eq, PartialEq)]
            enum E {
                Baz,
            }

            let _: AssertParamIsEq<E>;
        }

        #[test]
        fn single_variant_empty_tuple() {
            #[derive(Eq, PartialEq)]
            enum E {
                Foo(),
            }

            let _: AssertParamIsEq<E>;
        }

        #[test]
        fn single_variant_empty_struct() {
            #[derive(Eq, PartialEq)]
            enum E {
                Bar {},
            }

            let _: AssertParamIsEq<E>;
        }

        #[test]
        fn single_variant_multi_field_tuple() {
            #[derive(Eq, PartialEq)]
            enum E {
                Foo(bool, i32),
            }

            let _: AssertParamIsEq<E>;
        }

        #[test]
        fn single_variant_multi_field_struct() {
            #[derive(Eq, PartialEq)]
            enum E {
                Bar { b: bool, i: i32 },
            }

            let _: AssertParamIsEq<E>;
        }

        #[test]
        fn multi_variant_empty_field() {
            #[derive(Eq, PartialEq)]
            enum E {
                Foo(),
                Bar {},
                Baz,
            }

            let _: AssertParamIsEq<E>;
        }

        #[test]
        fn multi_variant_multi_field() {
            #[derive(Eq, PartialEq)]
            enum E {
                Foo(bool, i32),
                Bar { b: bool, i: i32 },
            }

            let _: AssertParamIsEq<E>;
        }

        #[test]
        fn multi_variant_empty_and_multi_field() {
            #[derive(Eq, PartialEq)]
            enum E {
                Foo(bool, i32),
                Baz,
            }

            let _: AssertParamIsEq<E>;
        }

        mod generic {
            use derive_more::{Eq, PartialEq, __private::AssertParamIsEq};

            trait Some {
                type Assoc;
            }

            impl<T> Some for T {
                type Assoc = bool;
            }

            #[test]
            fn single_variant_multi_field_tuple() {
                #[derive(Eq, PartialEq)]
                enum E<A: Some, B> {
                    Foo(A::Assoc, B),
                }

                let _: AssertParamIsEq<E<f32, ()>>;
            }

            #[test]
            fn single_variant_multi_field_struct() {
                #[derive(Eq, PartialEq)]
                enum E<A, B: Some> {
                    Bar { b: B::Assoc, i: A },
                }

                let _: AssertParamIsEq<E<&'static str, f64>>;
            }

            #[test]
            fn multi_variant_empty_and_multi_field() {
                #[derive(Eq, PartialEq)]
                enum E<A, B: Some> {
                    Foo(B::Assoc, A),
                    Bar { b: B::Assoc, i: A },
                    Baz,
                }

                let _: AssertParamIsEq<E<i64, f64>>;
            }

            #[test]
            fn lifetime() {
                #[derive(Eq, PartialEq)]
                enum E1<'a> {
                    Foo(&'a str, i32),
                }

                #[derive(Eq, PartialEq)]
                enum E2<'a> {
                    Bar { b: E1<'a>, i: i32 },
                }

                let _: AssertParamIsEq<E1>;
                let _: AssertParamIsEq<E2>;
            }

            #[test]
            fn const_param() {
                #[derive(Eq, PartialEq)]
                enum E3<const N: usize> {
                    Baz,
                }

                #[derive(Eq, PartialEq)]
                enum E1<const N: usize> {
                    Foo([i32; N], i8),
                }

                #[derive(Eq, PartialEq)]
                enum E2<const N: usize> {
                    Bar { b: E1<N>, i: E3<N> },
                }

                let _: AssertParamIsEq<E3<1>>;
                let _: AssertParamIsEq<E1<2>>;
                let _: AssertParamIsEq<E2<0>>;
            }

            #[test]
            fn mixed() {
                #[derive(Eq, PartialEq)]
                enum E<'a, A, B: Some, const N: usize> {
                    Foo([&'a A; N]),
                    Baz([B::Assoc; N]),
                }

                let _: AssertParamIsEq<E<i32, f64, 1>>;
            }
        }
    }
}

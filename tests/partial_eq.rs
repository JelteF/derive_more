#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)] // some code is tested for type checking only

/// Since [`assert_ne!()`] macro in [`core`] doesn't use `$left != $right` comparison, but rather
/// checks as `!($left == $right)`, it should be redefined for tests to consider actual
/// [`PartialEq::ne()`] implementations.
///
/// [`assert_ne!()`]: core::assert_ne
#[macro_export]
macro_rules! assert_ne {
    ($left:expr, $right:expr $(,)?) => {
        assert!($left != $right)
    };
}

mod structs {
    mod structural {
        use derive_more::PartialEq;

        #[test]
        fn unit() {
            #[derive(Debug, PartialEq)]
            struct Baz;

            assert_eq!(Baz, Baz);
        }

        #[test]
        fn empty_tuple() {
            #[derive(Debug, PartialEq)]
            struct Foo();

            assert_eq!(Foo(), Foo());
        }

        #[test]
        fn empty_struct() {
            #[derive(Debug, PartialEq)]
            struct Bar {}

            assert_eq!(Bar {}, Bar {});
        }

        #[test]
        fn multi_field_tuple() {
            #[derive(Debug, PartialEq)]
            struct Foo(bool, i32);

            assert_eq!(Foo(true, 0), Foo(true, 0));
            assert_ne!(Foo(true, 0), Foo(false, 0));
            assert_ne!(Foo(true, 0), Foo(true, 1));
            assert_ne!(Foo(true, 0), Foo(false, 1));
        }

        #[test]
        fn multi_field_struct() {
            #[derive(Debug, PartialEq)]
            struct Bar {
                b: bool,
                i: i32,
            }

            assert_eq!(Bar { b: true, i: 0 }, Bar { b: true, i: 0 });
            assert_ne!(Bar { b: true, i: 0 }, Bar { b: false, i: 0 });
            assert_ne!(Bar { b: true, i: 0 }, Bar { b: true, i: 1 });
            assert_ne!(Bar { b: true, i: 0 }, Bar { b: false, i: 1 });
        }

        mod generic {
            use derive_more::PartialEq;

            trait Some {
                type Assoc;
            }

            impl<T> Some for T {
                type Assoc = bool;
            }

            #[test]
            fn multi_field_tuple() {
                #[derive(Debug, PartialEq)]
                struct Foo<A: Some, B>(A::Assoc, B);

                assert_eq!(Foo::<(), _>(true, 0), Foo(true, 0));
                assert_ne!(Foo::<(), _>(true, 0), Foo(false, 0));
                assert_ne!(Foo::<(), _>(true, 0), Foo(true, 1));
                assert_ne!(Foo::<(), _>(true, 0), Foo(false, 1));
            }

            #[test]
            fn multi_field_struct() {
                #[derive(Debug, PartialEq)]
                struct Bar<A, B: Some> {
                    b: B::Assoc,
                    i: A,
                }

                assert_eq!(Bar::<_, ()> { b: true, i: 0 }, Bar { b: true, i: 0 });
                assert_ne!(Bar::<_, ()> { b: true, i: 0 }, Bar { b: false, i: 0 });
                assert_ne!(Bar::<_, ()> { b: true, i: 0 }, Bar { b: true, i: 1 });
                assert_ne!(Bar::<_, ()> { b: true, i: 0 }, Bar { b: false, i: 1 });
            }

            #[test]
            fn lifetime() {
                #[derive(Debug, PartialEq)]
                struct Foo<'a>(&'a str, i32);

                #[derive(Debug, PartialEq)]
                struct Bar<'a> {
                    b: Foo<'a>,
                    i: i32,
                }

                assert_eq!(Foo("hi", 0), Foo("hi", 0));
                assert_ne!(Foo("hi", 0), Foo("bye", 0));
                assert_ne!(Foo("hi", 0), Foo("hi", 1));
                assert_ne!(Foo("hi", 0), Foo("bye", 1));

                assert_eq!(
                    Bar {
                        b: Foo("hi", 0),
                        i: 0,
                    },
                    Bar {
                        b: Foo("hi", 0),
                        i: 0,
                    },
                );
                assert_ne!(
                    Bar {
                        b: Foo("hi", 0),
                        i: 0,
                    },
                    Bar {
                        b: Foo("bye", 0),
                        i: 0,
                    },
                );
                assert_ne!(
                    Bar {
                        b: Foo("hi", 0),
                        i: 0,
                    },
                    Bar {
                        b: Foo("hi", 0),
                        i: 1,
                    },
                );
                assert_ne!(
                    Bar {
                        b: Foo("hi", 0),
                        i: 0,
                    },
                    Bar {
                        b: Foo("bye", 0),
                        i: 1,
                    },
                );
            }

            #[test]
            fn const_param() {
                #[derive(Debug, PartialEq)]
                struct Baz<const N: usize>;

                #[derive(Debug, PartialEq)]
                struct Foo<const N: usize>([i32; N], i8);

                #[derive(Debug, PartialEq)]
                struct Bar<const N: usize> {
                    b: Foo<N>,
                    i: Baz<N>,
                }

                assert_eq!(Baz::<1>, Baz);

                assert_eq!(Foo([3], 0), Foo([3], 0));
                assert_ne!(Foo([3], 0), Foo([4], 0));
                assert_ne!(Foo([3], 0), Foo([3], 1));
                assert_ne!(Foo([3], 0), Foo([4], 1));

                assert_eq!(
                    Bar {
                        b: Foo([3], 0),
                        i: Baz,
                    },
                    Bar {
                        b: Foo([3], 0),
                        i: Baz,
                    },
                );
                assert_ne!(
                    Bar {
                        b: Foo([3], 0),
                        i: Baz,
                    },
                    Bar {
                        b: Foo([3], 1),
                        i: Baz,
                    },
                );
            }
        }
    }
}

mod enums {
    mod structural {
        use derive_more::PartialEq;

        #[test]
        fn empty() {
            #[derive(Debug, PartialEq)]
            enum E {}
        }

        #[test]
        fn single_variant_unit() {
            #[derive(Debug, PartialEq)]
            enum E {
                Baz,
            }

            assert_eq!(E::Baz, E::Baz);
        }

        #[test]
        fn single_variant_empty_tuple() {
            #[derive(Debug, PartialEq)]
            enum E {
                Foo(),
            }

            assert_eq!(E::Foo(), E::Foo());
        }

        #[test]
        fn single_variant_empty_struct() {
            #[derive(Debug, PartialEq)]
            enum E {
                Bar {},
            }

            assert_eq!(E::Bar {}, E::Bar {});
        }

        #[test]
        fn single_variant_multi_field_tuple() {
            #[derive(Debug, PartialEq)]
            enum E {
                Foo(bool, i32),
            }

            assert_eq!(E::Foo(true, 0), E::Foo(true, 0));
            assert_ne!(E::Foo(true, 0), E::Foo(false, 0));
            assert_ne!(E::Foo(true, 0), E::Foo(true, 1));
            assert_ne!(E::Foo(true, 0), E::Foo(false, 1));
        }

        #[test]
        fn single_variant_multi_field_struct() {
            #[derive(Debug, PartialEq)]
            enum E {
                Bar { b: bool, i: i32 },
            }

            assert_eq!(E::Bar { b: true, i: 0 }, E::Bar { b: true, i: 0 });
            assert_ne!(E::Bar { b: true, i: 0 }, E::Bar { b: false, i: 0 });
            assert_ne!(E::Bar { b: true, i: 0 }, E::Bar { b: true, i: 1 });
            assert_ne!(E::Bar { b: true, i: 0 }, E::Bar { b: false, i: 1 });
        }

        #[test]
        fn multi_variant_empty_field() {
            #[derive(Debug, PartialEq)]
            enum E {
                Foo(),
                Bar {},
                Baz,
            }

            assert_eq!(E::Foo(), E::Foo());

            assert_eq!(E::Bar {}, E::Bar {});

            assert_eq!(E::Baz, E::Baz);

            assert_ne!(E::Foo(), E::Bar {});
            assert_ne!(E::Bar {}, E::Foo());
            assert_ne!(E::Foo(), E::Baz);
            assert_ne!(E::Baz, E::Foo());
            assert_ne!(E::Bar {}, E::Baz);
            assert_ne!(E::Baz, E::Bar {});
        }

        #[test]
        fn multi_variant_multi_field() {
            #[derive(Debug, PartialEq)]
            enum E {
                Foo(bool, i32),
                Bar { b: bool, i: i32 },
            }

            assert_eq!(E::Foo(true, 0), E::Foo(true, 0));
            assert_ne!(E::Foo(true, 0), E::Foo(false, 0));
            assert_ne!(E::Foo(true, 0), E::Foo(true, 1));
            assert_ne!(E::Foo(true, 0), E::Foo(false, 1));

            assert_eq!(E::Bar { b: true, i: 0 }, E::Bar { b: true, i: 0 });
            assert_ne!(E::Bar { b: true, i: 0 }, E::Bar { b: false, i: 0 });
            assert_ne!(E::Bar { b: true, i: 0 }, E::Bar { b: true, i: 1 });
            assert_ne!(E::Bar { b: true, i: 0 }, E::Bar { b: false, i: 1 });

            assert_ne!(E::Foo(true, 0), E::Bar { b: true, i: 0 });
            assert_ne!(E::Bar { b: false, i: 1 }, E::Foo(false, 1));
        }

        #[test]
        fn multi_variant_empty_and_multi_field() {
            #[derive(Debug, PartialEq)]
            enum E {
                Foo(bool, i32),
                Baz,
            }

            assert_eq!(E::Foo(true, 0), E::Foo(true, 0));
            assert_ne!(E::Foo(true, 0), E::Foo(false, 0));
            assert_ne!(E::Foo(true, 0), E::Foo(true, 1));
            assert_ne!(E::Foo(true, 0), E::Foo(false, 1));

            assert_eq!(E::Baz, E::Baz);

            assert_ne!(E::Foo(true, 0), E::Baz);
            assert_ne!(E::Baz, E::Foo(false, 1));
        }

        mod generic {
            use derive_more::PartialEq;

            trait Some {
                type Assoc;
            }

            impl<T> Some for T {
                type Assoc = bool;
            }

            #[test]
            fn single_variant_multi_field_tuple() {
                #[derive(Debug, PartialEq)]
                enum E<A: Some, B> {
                    Foo(A::Assoc, B),
                }

                assert_eq!(E::<u8, _>::Foo(true, 0), E::Foo(true, 0));
                assert_ne!(E::<u8, _>::Foo(true, 0), E::Foo(false, 0));
                assert_ne!(E::<u8, _>::Foo(true, 0), E::Foo(true, 1));
                assert_ne!(E::<u8, _>::Foo(true, 0), E::Foo(false, 1));
            }

            #[test]
            fn single_variant_multi_field_struct() {
                #[derive(Debug, PartialEq)]
                enum E<A, B: Some> {
                    Bar { b: B::Assoc, i: A },
                }

                assert_eq!(E::<_, ()>::Bar { b: true, i: 0 }, E::Bar { b: true, i: 0 });
                assert_ne!(
                    E::<_, ()>::Bar { b: true, i: 0 },
                    E::Bar { b: false, i: 0 }
                );
                assert_ne!(E::<_, ()>::Bar { b: true, i: 0 }, E::Bar { b: true, i: 1 });
                assert_ne!(
                    E::<_, ()>::Bar { b: true, i: 0 },
                    E::Bar { b: false, i: 1 }
                );
            }

            #[test]
            fn multi_variant_empty_and_multi_field() {
                #[derive(Debug, derive_more::PartialEq)]
                enum E<A, B: Some> {
                    Foo(B::Assoc, A),
                    Bar { b: B::Assoc, i: A },
                    Baz,
                }

                assert_eq!(E::<_, ()>::Foo(true, 0), E::Foo(true, 0));
                assert_ne!(E::<_, ()>::Foo(true, 0), E::Foo(false, 0));
                assert_ne!(E::<_, ()>::Foo(true, 0), E::Foo(true, 1));
                assert_ne!(E::<_, ()>::Foo(true, 0), E::Foo(false, 1));

                assert_eq!(E::<_, ()>::Bar { b: true, i: 0 }, E::Bar { b: true, i: 0 });
                assert_ne!(
                    E::<_, ()>::Bar { b: true, i: 0 },
                    E::Bar { b: false, i: 0 }
                );
                assert_ne!(E::<_, ()>::Bar { b: true, i: 0 }, E::Bar { b: true, i: 1 });
                assert_ne!(
                    E::<_, ()>::Bar { b: true, i: 0 },
                    E::Bar { b: false, i: 1 }
                );

                assert_eq!(E::<i32, ()>::Baz, E::Baz);

                assert_ne!(E::<_, ()>::Foo(true, 0), E::Bar { b: true, i: 0 });
                assert_ne!(E::<_, ()>::Bar { b: false, i: 1 }, E::Foo(false, 1));
                assert_ne!(E::<_, ()>::Foo(true, 0), E::Baz);
                assert_ne!(E::<_, ()>::Baz, E::Foo(false, 1));
                assert_ne!(E::<_, ()>::Bar { b: false, i: 1 }, E::Baz);
                assert_ne!(E::<_, ()>::Baz, E::Bar { b: true, i: 0 });
            }

            #[test]
            fn lifetime() {
                #[derive(Debug, PartialEq)]
                enum E1<'a> {
                    Foo(&'a str, i32),
                }

                #[derive(Debug, PartialEq)]
                enum E2<'a> {
                    Bar { b: E1<'a>, i: i32 },
                }

                assert_eq!(E1::Foo("hi", 0), E1::Foo("hi", 0));
                assert_ne!(E1::Foo("hi", 0), E1::Foo("bye", 0));
                assert_ne!(E1::Foo("hi", 0), E1::Foo("hi", 1));
                assert_ne!(E1::Foo("hi", 0), E1::Foo("bye", 1));

                assert_eq!(
                    E2::Bar {
                        b: E1::Foo("hi", 0),
                        i: 0,
                    },
                    E2::Bar {
                        b: E1::Foo("hi", 0),
                        i: 0,
                    },
                );
                assert_ne!(
                    E2::Bar {
                        b: E1::Foo("hi", 0),
                        i: 0,
                    },
                    E2::Bar {
                        b: E1::Foo("bye", 0),
                        i: 0,
                    },
                );
                assert_ne!(
                    E2::Bar {
                        b: E1::Foo("hi", 0),
                        i: 0,
                    },
                    E2::Bar {
                        b: E1::Foo("hi", 0),
                        i: 1,
                    },
                );
                assert_ne!(
                    E2::Bar {
                        b: E1::Foo("hi", 0),
                        i: 0,
                    },
                    E2::Bar {
                        b: E1::Foo("bye", 0),
                        i: 1,
                    },
                );
            }

            #[test]
            fn const_param() {
                #[derive(Debug, PartialEq)]
                enum E3<const N: usize> {
                    Baz,
                }

                #[derive(Debug, PartialEq)]
                enum E1<const N: usize> {
                    Foo([i32; N], i8),
                }

                #[derive(Debug, PartialEq)]
                enum E2<const N: usize> {
                    Bar { b: E1<N>, i: E3<N> },
                }

                assert_eq!(E3::<1>::Baz, E3::Baz);

                assert_eq!(E1::Foo([3], 0), E1::Foo([3], 0));
                assert_ne!(E1::Foo([3], 0), E1::Foo([4], 0));
                assert_ne!(E1::Foo([3], 0), E1::Foo([3], 1));
                assert_ne!(E1::Foo([3], 0), E1::Foo([4], 1));

                assert_eq!(
                    E2::Bar {
                        b: E1::Foo([3], 0),
                        i: E3::Baz,
                    },
                    E2::Bar {
                        b: E1::Foo([3], 0),
                        i: E3::Baz,
                    },
                );
                assert_ne!(
                    E2::Bar {
                        b: E1::Foo([3], 0),
                        i: E3::Baz,
                    },
                    E2::Bar {
                        b: E1::Foo([3], 1),
                        i: E3::Baz,
                    },
                );
            }
        }
    }
}

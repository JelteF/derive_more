#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)] // some code is tested for type checking only

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
            #[derive(Debug, derive_more::PartialEq)]
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
    }
}

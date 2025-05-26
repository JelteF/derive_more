#![cfg_attr(not(feature = "std"), no_std)]

mod structs {
    mod structural {
        use derive_more::PartialEq;

        /*
        #[test]
        fn unit() {
            #[derive(Debug, Eq, FromStr, PartialEq)]
            struct Foo;

            assert_eq!("Foo".parse::<Foo>().unwrap(), Foo);
        }

        #[test]
        fn empty_tuple() {
            #[derive(Debug, Eq, FromStr, PartialEq)]
            struct Bar();

            assert_eq!("Bar".parse::<Bar>().unwrap(), Bar());
        }

        #[test]
        fn empty_struct() {
            #[derive(Debug, Eq, FromStr, PartialEq)]
            struct Baz {}

            assert_eq!("Baz".parse::<Baz>().unwrap(), Baz {});
        }

         */

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
        fn single_variant_multi_field_tuple() {
            #[derive(Debug, derive_more::PartialEq)]
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
        fn multi_field() {
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

        /*
        #[test]
        fn empty() {
            #[derive(Debug, Eq, FromStr, PartialEq)]
            enum Enum {}

            assert_eq!(
                "other".parse::<Enum>().unwrap_err().to_string(),
                "Invalid `Enum` string representation",
            );
        }

        #[test]
        fn unit() {
            #[derive(Debug, Eq, FromStr, PartialEq)]
            enum Enum {
                Foo,
            }

            assert_eq!("Foo".parse::<Enum>().unwrap(), Enum::Foo);
            assert_eq!("FOO".parse::<Enum>().unwrap(), Enum::Foo);
            assert_eq!("foo".parse::<Enum>().unwrap(), Enum::Foo);

            assert_eq!(
                "baz".parse::<Enum>().unwrap_err().to_string(),
                "Invalid `Enum` string representation",
            );
            assert_eq!(
                "other".parse::<Enum>().unwrap_err().to_string(),
                "Invalid `Enum` string representation",
            );
        }

        #[test]
        fn empty_tuple() {
            #[derive(Debug, Eq, FromStr, PartialEq)]
            enum Enum {
                Foo(),
            }

            assert_eq!("Foo".parse::<Enum>().unwrap(), Enum::Foo());
            assert_eq!("FOO".parse::<Enum>().unwrap(), Enum::Foo());
            assert_eq!("foo".parse::<Enum>().unwrap(), Enum::Foo());

            assert_eq!(
                "baz".parse::<Enum>().unwrap_err().to_string(),
                "Invalid `Enum` string representation",
            );
            assert_eq!(
                "other".parse::<Enum>().unwrap_err().to_string(),
                "Invalid `Enum` string representation",
            );
        }

        #[test]
        fn empty_struct() {
            #[derive(Debug, Eq, FromStr, PartialEq)]
            enum Enum {
                Foo {},
            }

            assert_eq!("Foo".parse::<Enum>().unwrap(), Enum::Foo {});
            assert_eq!("FOO".parse::<Enum>().unwrap(), Enum::Foo {});
            assert_eq!("foo".parse::<Enum>().unwrap(), Enum::Foo {});

            assert_eq!(
                "baz".parse::<Enum>().unwrap_err().to_string(),
                "Invalid `Enum` string representation",
            );
            assert_eq!(
                "other".parse::<Enum>().unwrap_err().to_string(),
                "Invalid `Enum` string representation",
            );
        }*/
    }
}

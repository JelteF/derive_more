#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code, clippy::unnecessary_mut_passed)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{borrow::ToOwned, collections::VecDeque, string::String, vec, vec::Vec};

#[cfg(feature = "std")]
use std::collections::VecDeque;

use core::ptr;

use derive_more::AsMut;

mod single_field {
    use super::*;

    mod tuple {
        use super::*;

        #[derive(AsMut)]
        struct Nothing(String);

        #[test]
        fn nothing() {
            let mut item = Nothing("test".to_owned());

            assert!(ptr::eq(item.as_mut(), &mut item.0));
        }

        #[derive(AsMut)]
        #[as_mut(forward)]
        struct Forward(String);

        #[test]
        fn forward() {
            let mut item = Forward("test".to_owned());

            let rf: &mut str = item.as_mut();
            assert!(ptr::eq(rf, item.0.as_mut()));
        }

        #[derive(AsMut)]
        struct Field(#[as_mut] String);

        #[test]
        fn field() {
            let mut item = Field("test".to_owned());

            assert!(ptr::eq(item.as_mut(), &mut item.0));
        }

        #[derive(AsMut)]
        struct FieldForward(#[as_mut(forward)] String);

        #[test]
        fn field_forward() {
            let mut item = FieldForward("test".to_owned());

            let rf: &mut str = item.as_mut();
            assert!(ptr::eq(rf, item.0.as_mut()));
        }

        mod generic {
            use super::*;

            #[derive(AsMut)]
            struct Nothing<T>(T);

            #[test]
            fn nothing() {
                let mut item = Nothing("test".to_owned());

                assert!(ptr::eq(item.as_mut(), &mut item.0));
            }

            #[derive(AsMut)]
            #[as_mut(forward)]
            struct Forward<T>(T);

            #[test]
            fn forward() {
                let mut item = Forward("test".to_owned());

                let rf: &mut str = item.as_mut();
                assert!(ptr::eq(rf, item.0.as_mut()));
            }

            #[derive(AsMut)]
            struct Field<T>(#[as_mut] T);

            #[test]
            fn field() {
                let mut item = Field("test".to_owned());

                assert!(ptr::eq(item.as_mut(), &mut item.0));
            }

            #[derive(AsMut)]
            struct FieldForward<T>(#[as_mut(forward)] T);

            #[test]
            fn field_forward() {
                let mut item = FieldForward("test".to_owned());

                let rf: &mut str = item.as_mut();
                assert!(ptr::eq(rf, item.0.as_mut()));
            }
        }
    }

    mod named {
        use super::*;

        #[derive(AsMut)]
        struct Nothing {
            first: String,
        }

        #[test]
        fn nothing() {
            let mut item = Nothing {
                first: "test".to_owned(),
            };

            assert!(ptr::eq(item.as_mut(), &mut item.first));
        }

        #[derive(AsMut)]
        #[as_mut(forward)]
        struct Forward {
            first: String,
        }

        #[test]
        fn forward() {
            let mut item = Forward {
                first: "test".to_owned(),
            };

            let rf: &mut str = item.as_mut();
            assert!(ptr::eq(rf, item.first.as_mut()));
        }

        #[derive(AsMut)]
        struct Field {
            #[as_mut]
            first: String,
        }

        #[test]
        fn field() {
            let mut item = Field {
                first: "test".to_owned(),
            };

            assert!(ptr::eq(item.as_mut(), &mut item.first));
        }

        #[derive(AsMut)]
        struct FieldForward {
            #[as_mut(forward)]
            first: String,
        }

        #[test]
        fn field_forward() {
            let mut item = FieldForward {
                first: "test".to_owned(),
            };

            let rf: &mut str = item.as_mut();
            assert!(ptr::eq(rf, item.first.as_mut()));
        }

        mod generic {
            use super::*;

            #[derive(AsMut)]
            struct Nothing<T> {
                first: T,
            }

            #[test]
            fn nothing() {
                let mut item = Nothing {
                    first: "test".to_owned(),
                };

                assert!(ptr::eq(item.as_mut(), &mut item.first));
            }

            #[derive(AsMut)]
            #[as_mut(forward)]
            struct Forward<T> {
                first: T,
            }

            #[test]
            fn struct_forward() {
                let mut item = Forward {
                    first: "test".to_owned(),
                };

                let rf: &mut str = item.as_mut();
                assert!(ptr::eq(rf, item.first.as_mut()));
            }

            #[derive(AsMut)]
            struct Field<T> {
                #[as_mut]
                first: T,
            }

            #[test]
            fn field() {
                let mut item = Field {
                    first: "test".to_owned(),
                };

                assert!(ptr::eq(item.as_mut(), &mut item.first));
            }

            #[derive(AsMut)]
            struct FieldForward<T> {
                #[as_mut(forward)]
                first: T,
            }

            #[test]
            fn field_forward() {
                let mut item = FieldForward {
                    first: "test".to_owned(),
                };

                let rf: &mut str = item.as_mut();
                assert!(ptr::eq(rf, item.first.as_mut()));
            }
        }
    }
}

mod multi_field {
    use super::*;

    mod tuple {
        use super::*;

        #[derive(AsMut)]
        struct Nothing(String, i32);

        #[test]
        fn nothing() {
            let mut item = Nothing("test".to_owned(), 0);

            assert!(ptr::eq(item.as_mut(), &mut item.0));
            assert!(ptr::eq(item.as_mut(), &mut item.1));
        }

        #[derive(AsMut)]
        struct Skip(String, i32, #[as_mut(skip)] f64);

        // Asserts that the macro expansion doesn't generate `AsMut` impl for the skipped field, by
        // producing trait implementations conflict error during compilation, if it does.
        impl AsMut<f64> for Skip {
            fn as_mut(&mut self) -> &mut f64 {
                &mut self.2
            }
        }

        #[test]
        fn skip() {
            let mut item = Skip("test".to_owned(), 0, 0.0);

            assert!(ptr::eq(item.as_mut(), &mut item.0));
            assert!(ptr::eq(item.as_mut(), &mut item.1));
        }

        #[derive(AsMut)]
        struct Field(#[as_mut] String, #[as_mut] i32, f64);

        // Asserts that the macro expansion doesn't generate `AsMut` impl for the third field, by
        // producing trait implementations conflict error during compilation, if it does.
        impl AsMut<f64> for Field {
            fn as_mut(&mut self) -> &mut f64 {
                &mut self.2
            }
        }

        #[test]
        fn field() {
            let mut item = Field("test".to_owned(), 0, 0.0);

            assert!(ptr::eq(item.as_mut(), &mut item.0));
            assert!(ptr::eq(item.as_mut(), &mut item.1));
        }

        #[derive(AsMut)]
        struct FieldForward(#[as_mut(forward)] String, i32);

        #[test]
        fn field_forward() {
            let mut item = FieldForward("test".to_owned(), 0);

            let rf: &mut str = item.as_mut();
            assert!(ptr::eq(rf, item.0.as_mut()));
        }

        mod generic {
            use super::*;

            #[derive(AsMut)]
            struct Nothing<T, U>(Vec<T>, VecDeque<U>);

            #[test]
            fn nothing() {
                let mut item = Nothing(vec![1], VecDeque::from([2]));

                assert!(ptr::eq(item.as_mut(), &mut item.0));
                assert!(ptr::eq(item.as_mut(), &mut item.1));
            }

            #[derive(AsMut)]
            struct Skip<T, U, V>(Vec<T>, VecDeque<U>, #[as_mut(skip)] V);

            #[test]
            fn skip() {
                let mut item = Skip(vec![1], VecDeque::from([2]), 0);

                assert!(ptr::eq(item.as_mut(), &mut item.0));
                assert!(ptr::eq(item.as_mut(), &mut item.1));
            }

            #[derive(AsMut)]
            struct Field<T, U, V>(#[as_mut] Vec<T>, #[as_mut] VecDeque<U>, V);

            #[test]
            fn field() {
                let mut item = Field(vec![1], VecDeque::from([2]), 0);

                assert!(ptr::eq(item.as_mut(), &mut item.0));
                assert!(ptr::eq(item.as_mut(), &mut item.1));
            }

            #[derive(AsMut)]
            struct FieldForward<T, U>(#[as_mut(forward)] T, U);

            #[test]
            fn field_forward() {
                let mut item = FieldForward("test".to_owned(), 0);

                let rf: &mut str = item.as_mut();
                assert!(ptr::eq(rf, item.0.as_mut()));
            }
        }
    }

    mod named {
        use super::*;

        #[derive(AsMut)]
        struct Nothing {
            first: String,
            second: i32,
        }

        #[test]
        fn nothing() {
            let mut item = Nothing {
                first: "test".to_owned(),
                second: 0,
            };

            assert!(ptr::eq(item.as_mut(), &mut item.first));
            assert!(ptr::eq(item.as_mut(), &mut item.second));
        }

        #[derive(AsMut)]
        struct Skip {
            first: String,
            second: i32,
            #[as_mut(skip)]
            third: f64,
        }

        // Asserts that the macro expansion doesn't generate `AsMut` impl for the skipped field, by
        // producing trait implementations conflict error during compilation, if it does.
        impl AsMut<f64> for Skip {
            fn as_mut(&mut self) -> &mut f64 {
                &mut self.third
            }
        }

        #[test]
        fn skip() {
            let mut item = Skip {
                first: "test".to_owned(),
                second: 0,
                third: 0.0,
            };

            assert!(ptr::eq(item.as_mut(), &mut item.first));
            assert!(ptr::eq(item.as_mut(), &mut item.second));
        }

        #[derive(AsMut)]
        struct Field {
            #[as_mut]
            first: String,
            #[as_mut]
            second: i32,
            third: f64,
        }

        // Asserts that the macro expansion doesn't generate `AsMut` impl for the `third` field, by
        // producing trait implementations conflict error during compilation, if it does.
        impl AsMut<f64> for Field {
            fn as_mut(&mut self) -> &mut f64 {
                &mut self.third
            }
        }

        #[test]
        fn field() {
            let mut item = Field {
                first: "test".to_owned(),
                second: 0,
                third: 0.0,
            };

            assert!(ptr::eq(item.as_mut(), &mut item.first));
            assert!(ptr::eq(item.as_mut(), &mut item.second));
        }

        #[derive(AsMut)]
        struct FieldForward {
            #[as_mut(forward)]
            first: String,
            second: i32,
        }

        #[test]
        fn field_forward() {
            let mut item = FieldForward {
                first: "test".to_owned(),
                second: 0,
            };

            let rf: &mut str = item.as_mut();
            assert!(ptr::eq(rf, item.first.as_mut()));
        }

        mod generic {
            use super::*;

            #[derive(AsMut)]
            struct Nothing<T, U> {
                first: Vec<T>,
                second: VecDeque<U>,
            }

            #[test]
            fn nothing() {
                let mut item = Nothing {
                    first: vec![1],
                    second: VecDeque::from([2]),
                };

                assert!(ptr::eq(item.as_mut(), &mut item.first));
                assert!(ptr::eq(item.as_mut(), &mut item.second));
            }

            #[derive(AsMut)]
            struct Skip<T, U, V> {
                first: Vec<T>,
                second: VecDeque<U>,
                #[as_mut(skip)]
                third: V,
            }

            #[test]
            fn skip() {
                let mut item = Skip {
                    first: vec![1],
                    second: VecDeque::from([2]),
                    third: 0,
                };

                assert!(ptr::eq(item.as_mut(), &mut item.first));
                assert!(ptr::eq(item.as_mut(), &mut item.second));
            }

            #[derive(AsMut)]
            struct Field<T, U, V> {
                #[as_mut]
                first: Vec<T>,
                #[as_mut]
                second: VecDeque<U>,
                third: V,
            }

            #[test]
            fn field() {
                let mut item = Field {
                    first: vec![1],
                    second: VecDeque::from([2]),
                    third: 0,
                };

                assert!(ptr::eq(item.as_mut(), &mut item.first));
                assert!(ptr::eq(item.as_mut(), &mut item.second));
            }

            #[derive(AsMut)]
            struct FieldForward<T, U> {
                #[as_mut(forward)]
                first: T,
                second: U,
            }

            #[test]
            fn field_forward() {
                let mut item = FieldForward {
                    first: "test".to_owned(),
                    second: 0,
                };

                let rf: &mut str = item.as_mut();
                assert!(ptr::eq(rf, item.first.as_mut()));
            }
        }
    }
}

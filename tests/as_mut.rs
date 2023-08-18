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
        struct NoAttr(String);

        #[test]
        fn no_attr() {
            let mut item = NoAttr("test".to_owned());

            assert!(ptr::eq(item.as_mut(), &mut item.0));
        }

        #[derive(AsMut)]
        #[as_mut(forward)]
        struct StructForward(String);

        #[test]
        fn struct_forward() {
            let mut item = StructForward("test".to_owned());

            let rf: &mut str = item.as_mut();
            assert!(ptr::eq(rf, item.0.as_mut()));
        }

        #[derive(AsMut)]
        struct FieldEmpty(#[as_mut] String);

        #[test]
        fn field_empty() {
            let mut item = FieldEmpty("test".to_owned());

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
            struct NoAttr<T>(T);

            #[test]
            fn no_attr() {
                let mut item = NoAttr("test".to_owned());

                assert!(ptr::eq(item.as_mut(), &mut item.0));
            }

            #[derive(AsMut)]
            #[as_mut(forward)]
            struct StructForward<T>(T);

            #[test]
            fn struct_forward() {
                let mut item = StructForward("test".to_owned());

                let rf: &mut str = item.as_mut();
                assert!(ptr::eq(rf, item.0.as_mut()));
            }

            #[derive(AsMut)]
            struct FieldEmpty<T>(#[as_mut] T);

            #[test]
            fn field_empty() {
                let mut item = FieldEmpty("test".to_owned());

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
        struct NoAttr {
            first: String,
        }

        #[test]
        fn no_attr() {
            let mut item = NoAttr {
                first: "test".to_owned(),
            };

            assert!(ptr::eq(item.as_mut(), &mut item.first));
        }

        #[derive(AsMut)]
        #[as_mut(forward)]
        struct StructForward {
            first: String,
        }

        #[test]
        fn struct_forward() {
            let mut item = StructForward {
                first: "test".to_owned(),
            };

            let rf: &mut str = item.as_mut();
            assert!(ptr::eq(rf, item.first.as_mut()));
        }

        #[derive(AsMut)]
        struct FieldEmpty {
            #[as_mut]
            first: String,
        }

        #[test]
        fn field_empty() {
            let mut item = FieldEmpty {
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
            struct NoAttr<T> {
                first: T,
            }

            #[test]
            fn no_attr() {
                let mut item = NoAttr {
                    first: "test".to_owned(),
                };

                assert!(ptr::eq(item.as_mut(), &mut item.first));
            }

            #[derive(AsMut)]
            #[as_mut(forward)]
            struct StructForward<T> {
                first: T,
            }

            #[test]
            fn struct_forward() {
                let mut item = StructForward {
                    first: "test".to_owned(),
                };

                let rf: &mut str = item.as_mut();
                assert!(ptr::eq(rf, item.first.as_mut()));
            }

            #[derive(AsMut)]
            struct FieldEmpty<T> {
                #[as_mut]
                first: T,
            }

            #[test]
            fn field_empty() {
                let mut item = FieldEmpty {
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
        struct NoAttr(String, i32);

        #[test]
        fn no_attr() {
            let mut item = NoAttr("test".to_owned(), 0);

            assert!(ptr::eq(item.as_mut(), &mut item.0));
            assert!(ptr::eq(item.as_mut(), &mut item.1));
        }

        #[derive(AsMut)]
        struct Skip(String, i32, #[as_mut(skip)] f64);

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
        struct FieldEmpty(#[as_mut] String, #[as_mut] i32, f64);

        impl AsMut<f64> for FieldEmpty {
            fn as_mut(&mut self) -> &mut f64 {
                &mut self.2
            }
        }

        #[test]
        fn field_empty() {
            let mut item = FieldEmpty("test".to_owned(), 0, 0.0);

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
            struct NoAttr<T, U>(Vec<T>, VecDeque<U>);

            #[test]
            fn no_attr() {
                let mut item = NoAttr(vec![1], VecDeque::from([2]));

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
            struct FieldEmpty<T, U, V>(#[as_mut] Vec<T>, #[as_mut] VecDeque<U>, V);

            #[test]
            fn field_empty() {
                let mut item = FieldEmpty(vec![1], VecDeque::from([2]), 0);

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
        struct NoAttr {
            first: String,
            second: i32,
        }

        #[test]
        fn no_attr() {
            let mut item = NoAttr {
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
        struct FieldEmpty {
            #[as_mut]
            first: String,
            #[as_mut]
            second: i32,
            third: f64,
        }

        impl AsMut<f64> for FieldEmpty {
            fn as_mut(&mut self) -> &mut f64 {
                &mut self.third
            }
        }

        #[test]
        fn field_empty() {
            let mut item = FieldEmpty {
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
            struct NoAttr<T, U> {
                first: Vec<T>,
                second: VecDeque<U>,
            }

            #[test]
            fn no_attr() {
                let mut item = NoAttr {
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
            struct FieldEmpty<T, U, V> {
                #[as_mut]
                first: Vec<T>,
                #[as_mut]
                second: VecDeque<U>,
                third: V,
            }

            #[test]
            fn field_empty() {
                let mut item = FieldEmpty {
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

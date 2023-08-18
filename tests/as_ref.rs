#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{borrow::ToOwned, collections::VecDeque, string::String, vec, vec::Vec};

#[cfg(feature = "std")]
use std::collections::VecDeque;

use core::ptr;

use derive_more::AsRef;

mod single_field {
    use super::*;

    mod tuple {
        use super::*;

        #[derive(AsRef)]
        struct Nothing(String);

        #[test]
        fn nothing() {
            let item = Nothing("test".to_owned());

            assert!(ptr::eq(item.as_ref(), &item.0));
        }

        #[derive(AsRef)]
        #[as_ref(forward)]
        struct Forward(String);

        #[test]
        fn forward() {
            let item = Forward("test".to_owned());

            let rf: &str = item.as_ref();
            assert!(ptr::eq(rf, item.0.as_ref()));
        }

        #[derive(AsRef)]
        struct Field(#[as_ref] String);

        #[test]
        fn field() {
            let item = Field("test".to_owned());

            assert!(ptr::eq(item.as_ref(), &item.0));
        }

        #[derive(AsRef)]
        struct FieldForward(#[as_ref(forward)] String);

        #[test]
        fn field_forward() {
            let item = FieldForward("test".to_owned());

            let rf: &str = item.as_ref();
            assert!(ptr::eq(rf, item.0.as_ref()));
        }

        mod generic {
            use super::*;

            #[derive(AsRef)]
            struct Nothing<T>(T);

            #[test]
            fn nothing() {
                let item = Nothing("test".to_owned());

                assert!(ptr::eq(item.as_ref(), &item.0));
            }

            #[derive(AsRef)]
            #[as_ref(forward)]
            struct Forward<T>(T);

            #[test]
            fn forward() {
                let item = Forward("test".to_owned());

                let rf: &str = item.as_ref();
                assert!(ptr::eq(rf, item.0.as_ref()));
            }

            #[derive(AsRef)]
            struct Field<T>(#[as_ref] T);

            #[test]
            fn field() {
                let item = Field("test".to_owned());

                assert!(ptr::eq(item.as_ref(), &item.0));
            }

            #[derive(AsRef)]
            struct FieldForward<T>(#[as_ref(forward)] T);

            #[test]
            fn field_forward() {
                let item = FieldForward("test".to_owned());

                let rf: &str = item.as_ref();
                assert!(ptr::eq(rf, item.0.as_ref()));
            }
        }
    }

    mod named {
        use super::*;

        #[derive(AsRef)]
        struct Nothing {
            first: String,
        }

        #[test]
        fn nothing() {
            let item = Nothing {
                first: "test".to_owned(),
            };

            assert!(ptr::eq(item.as_ref(), &item.first));
        }

        #[derive(AsRef)]
        #[as_ref(forward)]
        struct Forward {
            first: String,
        }

        #[test]
        fn forward() {
            let item = Forward {
                first: "test".to_owned(),
            };

            let rf: &str = item.as_ref();
            assert!(ptr::eq(rf, item.first.as_ref()));
        }

        #[derive(AsRef)]
        struct Field {
            #[as_ref]
            first: String,
        }

        #[test]
        fn field() {
            let item = Field {
                first: "test".to_owned(),
            };

            assert!(ptr::eq(item.as_ref(), &item.first));
        }

        #[derive(AsRef)]
        struct FieldForward {
            #[as_ref(forward)]
            first: String,
        }

        #[test]
        fn field_forward() {
            let item = FieldForward {
                first: "test".to_owned(),
            };

            let rf: &str = item.as_ref();
            assert!(ptr::eq(rf, item.first.as_ref()));
        }

        mod generic {
            use super::*;

            #[derive(AsRef)]
            struct Nothing<T> {
                first: T,
            }

            #[test]
            fn nothing() {
                let item = Nothing {
                    first: "test".to_owned(),
                };

                assert!(ptr::eq(item.as_ref(), &item.first));
            }

            #[derive(AsRef)]
            #[as_ref(forward)]
            struct Forward<T> {
                first: T,
            }

            #[test]
            fn forward() {
                let item = Forward {
                    first: "test".to_owned(),
                };

                let rf: &str = item.as_ref();
                assert!(ptr::eq(rf, item.first.as_ref()));
            }

            #[derive(AsRef)]
            struct Field<T> {
                #[as_ref]
                first: T,
            }

            #[test]
            fn field() {
                let item = Field {
                    first: "test".to_owned(),
                };

                assert!(ptr::eq(item.as_ref(), &item.first));
            }

            #[derive(AsRef)]
            struct FieldForward<T> {
                #[as_ref(forward)]
                first: T,
            }

            #[test]
            fn field_forward() {
                let item = FieldForward {
                    first: "test".to_owned(),
                };

                let rf: &str = item.as_ref();
                assert!(ptr::eq(rf, item.first.as_ref()));
            }
        }
    }
}

mod multi_field {
    use super::*;

    mod tuple {
        use super::*;

        #[derive(AsRef)]
        struct Nothing(String, i32);

        #[test]
        fn nothing() {
            let item = Nothing("test".to_owned(), 0);

            assert!(ptr::eq(item.as_ref(), &item.0));
            assert!(ptr::eq(item.as_ref(), &item.1));
        }

        #[derive(AsRef)]
        struct Skip(String, i32, #[as_ref(skip)] f64);

        // Asserts that the macro expansion doesn't generate `AsRef` impl for the skipped field, by
        // producing trait implementations conflict error during compilation, if it does.
        impl AsRef<f64> for Skip {
            fn as_ref(&self) -> &f64 {
                &self.2
            }
        }

        #[test]
        fn skip() {
            let item = Skip("test".to_owned(), 0, 0.0);

            assert!(ptr::eq(item.as_ref(), &item.0));
            assert!(ptr::eq(item.as_ref(), &item.1));
        }

        #[derive(AsRef)]
        struct Field(#[as_ref] String, #[as_ref] i32, f64);

        // Asserts that the macro expansion doesn't generate `AsRef` impl for the third field, by
        // producing trait implementations conflict error during compilation, if it does.
        impl AsRef<f64> for Field {
            fn as_ref(&self) -> &f64 {
                &self.2
            }
        }

        #[test]
        fn field() {
            let item = Field("test".to_owned(), 0, 0.0);

            assert!(ptr::eq(item.as_ref(), &item.0));
            assert!(ptr::eq(item.as_ref(), &item.1));
        }

        #[derive(AsRef)]
        struct FieldForward(#[as_ref(forward)] String, i32);

        #[test]
        fn field_forward() {
            let item = FieldForward("test".to_owned(), 0);

            let rf: &str = item.as_ref();
            assert!(ptr::eq(rf, item.0.as_ref()));
        }

        mod generic {
            use super::*;

            #[derive(AsRef)]
            struct Nothing<T, U>(Vec<T>, VecDeque<U>);

            #[test]
            fn nothing() {
                let item = Nothing(vec![1], VecDeque::from([2]));

                assert!(ptr::eq(item.as_ref(), &item.0));
                assert!(ptr::eq(item.as_ref(), &item.1));
            }

            #[derive(AsRef)]
            struct Skip<T, U, V>(Vec<T>, VecDeque<U>, #[as_ref(skip)] V);

            #[test]
            fn skip() {
                let item = Skip(vec![1], VecDeque::from([2]), 0);

                assert!(ptr::eq(item.as_ref(), &item.0));
                assert!(ptr::eq(item.as_ref(), &item.1));
            }

            #[derive(AsRef)]
            struct Field<T, U, V>(#[as_ref] Vec<T>, #[as_ref] VecDeque<U>, V);

            #[test]
            fn field() {
                let item = Field(vec![1], VecDeque::from([2]), 0);

                assert!(ptr::eq(item.as_ref(), &item.0));
                assert!(ptr::eq(item.as_ref(), &item.1));
            }

            #[derive(AsRef)]
            struct FieldForward<T, U>(#[as_ref(forward)] T, U);

            #[test]
            fn field_forward() {
                let item = FieldForward("test".to_owned(), 0);

                let rf: &str = item.as_ref();
                assert!(ptr::eq(rf, item.0.as_ref()));
            }
        }
    }

    mod named {
        use super::*;

        #[derive(AsRef)]
        struct Nothing {
            first: String,
            second: i32,
        }

        #[test]
        fn nothing() {
            let item = Nothing {
                first: "test".to_owned(),
                second: 0,
            };

            assert!(ptr::eq(item.as_ref(), &item.first));
            assert!(ptr::eq(item.as_ref(), &item.second));
        }

        #[derive(AsRef)]
        struct Skip {
            first: String,
            second: i32,
            #[as_ref(skip)]
            third: f64,
        }

        // Asserts that the macro expansion doesn't generate `AsRef` impl for the skipped field, by
        // producing trait implementations conflict error during compilation, if it does.
        impl AsRef<f64> for Skip {
            fn as_ref(&self) -> &f64 {
                &self.third
            }
        }

        #[test]
        fn skip() {
            let item = Skip {
                first: "test".to_owned(),
                second: 0,
                third: 0.0,
            };

            assert!(ptr::eq(item.as_ref(), &item.first));
            assert!(ptr::eq(item.as_ref(), &item.second));
        }

        #[derive(AsRef)]
        struct Field {
            #[as_ref]
            first: String,
            #[as_ref]
            second: i32,
            third: f64,
        }

        // Asserts that the macro expansion doesn't generate `AsRef` impl for the `third` field, by
        // producing trait implementations conflict error during compilation, if it does.
        impl AsRef<f64> for Field {
            fn as_ref(&self) -> &f64 {
                &self.third
            }
        }

        #[test]
        fn field() {
            let item = Field {
                first: "test".to_owned(),
                second: 0,
                third: 0.0,
            };

            assert!(ptr::eq(item.as_ref(), &item.first));
            assert!(ptr::eq(item.as_ref(), &item.second));
        }

        #[derive(AsRef)]
        struct FieldForward {
            #[as_ref(forward)]
            first: String,
            second: i32,
        }

        #[test]
        fn field_forward() {
            let item = FieldForward {
                first: "test".to_owned(),
                second: 0,
            };

            let rf: &str = item.as_ref();
            assert!(ptr::eq(rf, item.first.as_ref()));
        }

        mod generic {
            use super::*;

            #[derive(AsRef)]
            struct Nothing<T, U> {
                first: Vec<T>,
                second: VecDeque<U>,
            }

            #[test]
            fn nothing() {
                let item = Nothing {
                    first: vec![1],
                    second: VecDeque::from([2]),
                };

                assert!(ptr::eq(item.as_ref(), &item.first));
                assert!(ptr::eq(item.as_ref(), &item.second));
            }

            #[derive(AsRef)]
            struct Skip<T, U, V> {
                first: Vec<T>,
                second: VecDeque<U>,
                #[as_ref(skip)]
                third: V,
            }

            #[test]
            fn skip() {
                let item = Skip {
                    first: vec![1],
                    second: VecDeque::from([2]),
                    third: 0,
                };

                assert!(ptr::eq(item.as_ref(), &item.first));
                assert!(ptr::eq(item.as_ref(), &item.second));
            }

            #[derive(AsRef)]
            struct Field<T, U, V> {
                #[as_ref]
                first: Vec<T>,
                #[as_ref]
                second: VecDeque<U>,
                third: V,
            }

            #[test]
            fn field() {
                let item = Field {
                    first: vec![1],
                    second: VecDeque::from([2]),
                    third: 0,
                };

                assert!(ptr::eq(item.as_ref(), &item.first));
                assert!(ptr::eq(item.as_ref(), &item.second));
            }

            #[derive(AsRef)]
            struct FieldForward<T, U> {
                #[as_ref(forward)]
                first: T,
                second: U,
            }

            #[test]
            fn field_forward() {
                let item = FieldForward {
                    first: "test".to_owned(),
                    second: 0,
                };

                let rf: &str = item.as_ref();
                assert!(ptr::eq(rf, item.first.as_ref()));
            }
        }
    }
}

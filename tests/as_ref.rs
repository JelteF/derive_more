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

struct Foo(i32, f64, bool);

impl AsRef<i32> for Foo {
    fn as_ref(&self) -> &i32 {
        &self.0
    }
}

impl AsRef<f64> for Foo {
    fn as_ref(&self) -> &f64 {
        &self.1
    }
}

impl AsRef<bool> for Foo {
    fn as_ref(&self) -> &bool {
        &self.2
    }
}

struct Bar<T>(T);

impl AsRef<i32> for Bar<&'static i32> {
    fn as_ref(&self) -> &i32 {
        self.0
    }
}

impl AsRef<[i32]> for Bar<[i32; 0]> {
    fn as_ref(&self) -> &[i32] {
        &self.0
    }
}

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

        #[derive(AsRef)]
        #[as_ref(i32, f64)]
        struct Types(Foo);

        // Asserts that the macro expansion doesn't generate a blanket `AsRef`
        // impl forwarding to the field type, by producing  a trait implementations
        // conflict error during compilation, if it does.
        impl AsRef<bool> for Types {
            fn as_ref(&self) -> &bool {
                self.0.as_ref()
            }
        }

        // Asserts that the macro expansion doesn't generate an `AsRef` impl for
        // the field type, by producing a trait implementations conflict error
        // during compilation, if it does.
        impl AsRef<Foo> for Types {
            fn as_ref(&self) -> &Foo {
                &self.0
            }
        }

        #[test]
        fn types() {
            let item = Types(Foo(1, 2.0, false));

            let rf: &i32 = item.as_ref();
            assert!(ptr::eq(rf, item.0.as_ref()));

            let rf: &f64 = item.as_ref();
            assert!(ptr::eq(rf, item.0.as_ref()));
        }

        #[derive(AsRef)]
        #[as_ref(i32, Foo)]
        struct TypesWithInner(Foo);

        // Asserts that the macro expansion doesn't generate a blanket `AsRef`
        // impl forwarding to the field type, by producing a trait implementations
        // conflict error during compilation, if it does.
        impl AsRef<bool> for TypesWithInner {
            fn as_ref(&self) -> &bool {
                self.0.as_ref()
            }
        }

        #[test]
        fn types_with_inner() {
            let item = TypesWithInner(Foo(1, 2.0, false));

            let rf: &i32 = item.as_ref();
            assert!(ptr::eq(rf, item.0.as_ref()));

            let rf: &Foo = item.as_ref();
            assert!(ptr::eq(rf, &item.0));
        }

        type RenamedFoo = Foo;

        #[derive(AsRef)]
        #[as_ref(i32, RenamedFoo)]
        struct TypesWithRenamedInner(Foo);

        // Asserts that the macro expansion doesn't generate a blanket `AsRef`
        // impl forwarding to the field type, by producing a trait implementations
        // conflict error during compilation, if it does.

        impl AsRef<bool> for TypesWithRenamedInner {
            fn as_ref(&self) -> &bool {
                self.0.as_ref()
            }
        }

        #[test]
        fn types_with_renamed_inner() {
            let item = TypesWithRenamedInner(Foo(1, 2.0, false));

            let rf: &i32 = item.as_ref();
            assert!(ptr::eq(rf, item.0.as_ref()));

            let rf: &Foo = item.as_ref();
            assert!(ptr::eq(rf, &item.0));
        }

        #[derive(AsRef)]
        struct FieldTypes(#[as_ref(i32, f64)] Foo);

        // Asserts that the macro expansion doesn't generate a blanket `AsRef`
        // impl forwarding to the field type, by producing a trait implementations
        // conflict error during compilation, if it does.
        impl AsRef<bool> for FieldTypes {
            fn as_ref(&self) -> &bool {
                self.0.as_ref()
            }
        }

        // Asserts that the macro expansion doesn't generate an `AsRef` impl for
        // the field type, by producing a trait implementations conflict error
        // during compilation, if it does.
        impl AsRef<Foo> for FieldTypes {
            fn as_ref(&self) -> &Foo {
                &self.0
            }
        }

        #[test]
        fn field_types() {
            let item = FieldTypes(Foo(1, 2.0, false));

            let rf: &i32 = item.as_ref();
            assert!(ptr::eq(rf, item.0.as_ref()));

            let rf: &f64 = item.as_ref();
            assert!(ptr::eq(rf, item.0.as_ref()));
        }

        #[derive(AsRef)]
        struct FieldTypesWithInner(#[as_ref(i32, Foo)] Foo);

        // Asserts that the macro expansion doesn't generate a blanket `AsRef`
        // impl forwarding to the field type, by producing a trait implementations
        // conflict error during compilation, if it does.
        impl AsRef<bool> for FieldTypesWithInner {
            fn as_ref(&self) -> &bool {
                self.0.as_ref()
            }
        }

        #[test]
        fn field_types_with_inner() {
            let item = FieldTypesWithInner(Foo(1, 2.0, false));

            let rf: &i32 = item.as_ref();
            assert!(ptr::eq(rf, item.0.as_ref()));

            let rf: &Foo = item.as_ref();
            assert!(ptr::eq(rf, &item.0));
        }

        #[derive(AsRef)]
        struct FieldTypesWithRenamedInner(#[as_ref(i32, RenamedFoo)] Foo);

        // Asserts that the macro expansion doesn't generate a blanket `AsRef`
        // impl forwarding to the field type, by producing a trait implementations
        // conflict error during compilation, if it does.
        impl AsRef<bool> for FieldTypesWithRenamedInner {
            fn as_ref(&self) -> &bool {
                self.0.as_ref()
            }
        }

        #[test]
        fn field_types_with_renamed_inner() {
            let item = FieldTypesWithRenamedInner(Foo(1, 2.0, false));

            let rf: &i32 = item.as_ref();
            assert!(ptr::eq(rf, item.0.as_ref()));

            let rf: &Foo = item.as_ref();
            assert!(ptr::eq(rf, &item.0));
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

            #[derive(AsRef)]
            #[as_ref(i32, f64)]
            struct Types<T>(T);

            // Asserts that the macro expansion doesn't generate a blanket `AsRef`
            // impl forwarding to the field type, by producing a trait implementations
            // conflict error during compilation, if it does.
            impl<T: AsRef<bool>> AsRef<bool> for Types<T> {
                fn as_ref(&self) -> &bool {
                    self.0.as_ref()
                }
            }

            #[test]
            fn types() {
                let item = Types(Foo(1, 2.0, false));

                let rf: &i32 = item.as_ref();
                assert!(ptr::eq(rf, item.0.as_ref()));

                let rf: &f64 = item.as_ref();
                assert!(ptr::eq(rf, item.0.as_ref()));
            }

            #[derive(AsRef)]
            #[as_ref(Vec<T>)]
            struct TypesInner<T>(Vec<T>);

            #[test]
            fn types_inner() {
                let item = TypesInner(vec![1i32]);

                assert!(ptr::eq(item.as_ref(), &item.0));
            }

            #[derive(AsRef)]
            struct FieldTypes<T>(#[as_ref(i32, f64)] T);

            // Asserts that the macro expansion doesn't generate a blanket `AsRef`
            // impl forwarding to the field type, by producing a trait implementations
            // conflict error during compilation, if it does.
            impl<T: AsRef<bool>> AsRef<bool> for FieldTypes<T> {
                fn as_ref(&self) -> &bool {
                    self.0.as_ref()
                }
            }

            #[test]
            fn field_types() {
                let item = FieldTypes(Foo(1, 2.0, false));

                let rf: &i32 = item.as_ref();
                assert!(ptr::eq(rf, item.0.as_ref()));

                let rf: &f64 = item.as_ref();
                assert!(ptr::eq(rf, item.0.as_ref()));
            }

            #[derive(AsRef)]
            struct FieldTypesInner<T>(#[as_ref(Vec<T>)] Vec<T>);

            #[test]
            fn field_types_inner() {
                let item = FieldTypesInner(vec![1i32]);

                assert!(ptr::eq(item.as_ref(), &item.0));
            }

            #[derive(AsRef)]
            #[as_ref(i32)]
            struct Lifetime<'a>(Bar<&'a i32>);

            #[test]
            fn lifetime() {
                let item = Lifetime(Bar(&1));

                assert!(ptr::eq(item.as_ref(), item.0.as_ref()));
            }

            #[derive(AsRef)]
            struct FieldLifetime<'a>(#[as_ref(i32)] Bar<&'a i32>);

            #[test]
            fn field_lifetime() {
                let item = FieldLifetime(Bar(&1));

                assert!(ptr::eq(item.as_ref(), item.0.as_ref()));
            }

            #[derive(AsRef)]
            #[as_ref([i32])]
            struct ConstParam<const N: usize>(Bar<[i32; N]>);

            #[test]
            fn const_param() {
                let item = ConstParam(Bar([]));

                assert!(ptr::eq(item.as_ref(), item.0.as_ref()));
            }

            #[derive(AsRef)]
            struct FieldConstParam<const N: usize>(#[as_ref([i32])] Bar<[i32; N]>);

            #[test]
            fn field_const_param() {
                let item = FieldConstParam(Bar([]));

                assert!(ptr::eq(item.as_ref(), item.0.as_ref()));
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

        #[derive(AsRef)]
        #[as_ref(i32, f64)]
        struct Types {
            first: Foo,
        }

        // Asserts that the macro expansion doesn't generate a blanket `AsRef`
        // impl forwarding to the field type, by producing a trait implementations
        // conflict error during compilation, if it does.
        impl AsRef<bool> for Types {
            fn as_ref(&self) -> &bool {
                self.first.as_ref()
            }
        }

        // Asserts that the macro expansion doesn't generate an `AsRef` impl for
        // the field type, by producing a trait implementations conflict error
        // during compilation, if it does.
        impl AsRef<Foo> for Types {
            fn as_ref(&self) -> &Foo {
                &self.first
            }
        }

        #[test]
        fn types() {
            let item = Types {
                first: Foo(1, 2.0, false),
            };

            let rf: &i32 = item.as_ref();
            assert!(ptr::eq(rf, item.first.as_ref()));

            let rf: &f64 = item.as_ref();
            assert!(ptr::eq(rf, item.first.as_ref()));
        }

        #[derive(AsRef)]
        #[as_ref(i32, Foo)]
        struct TypesWithInner {
            first: Foo,
        }

        // Asserts that the macro expansion doesn't generate a blanket `AsRef`
        // impl forwarding to the field type, by producing a trait implementations
        // conflict error during compilation, if it does.
        impl AsRef<bool> for TypesWithInner {
            fn as_ref(&self) -> &bool {
                self.first.as_ref()
            }
        }

        #[test]
        fn types_with_inner() {
            let item = TypesWithInner {
                first: Foo(1, 2.0, false),
            };

            let rf: &i32 = item.as_ref();
            assert!(ptr::eq(rf, item.first.as_ref()));

            let rf: &Foo = item.as_ref();
            assert!(ptr::eq(rf, &item.first));
        }

        type RenamedFoo = Foo;

        #[derive(AsRef)]
        #[as_ref(i32, RenamedFoo)]
        struct TypesWithRenamedInner {
            first: Foo,
        }

        // Asserts that the macro expansion doesn't generate a blanket `AsRef`
        // impl forwarding to the field type, by producing a trait implementations
        // conflict error during compilation, if it does.
        impl AsRef<bool> for TypesWithRenamedInner {
            fn as_ref(&self) -> &bool {
                self.first.as_ref()
            }
        }

        #[test]
        fn types_with_renamed_inner() {
            let item = TypesWithRenamedInner {
                first: Foo(1, 2.0, false),
            };

            let rf: &i32 = item.as_ref();
            assert!(ptr::eq(rf, item.first.as_ref()));

            let rf: &Foo = item.as_ref();
            assert!(ptr::eq(rf, &item.first));
        }

        #[derive(AsRef)]
        struct FieldTypes {
            #[as_ref(i32, f64)]
            first: Foo,
        }

        // Asserts that the macro expansion doesn't generate a blanket `AsRef`
        // impl forwarding to the field type, by producing a trait implementations
        // conflict error during compilation, if it does.
        impl AsRef<bool> for FieldTypes {
            fn as_ref(&self) -> &bool {
                self.first.as_ref()
            }
        }

        // Asserts that the macro expansion doesn't generate an `AsRef` impl for
        // the field type, by producing a trait implementations conflict error
        // during compilation, if it does.
        impl AsRef<Foo> for FieldTypes {
            fn as_ref(&self) -> &Foo {
                &self.first
            }
        }

        #[test]
        fn field_types() {
            let item = FieldTypes {
                first: Foo(1, 2.0, false),
            };

            let rf: &i32 = item.as_ref();
            assert!(ptr::eq(rf, item.first.as_ref()));

            let rf: &f64 = item.as_ref();
            assert!(ptr::eq(rf, item.first.as_ref()));
        }

        #[derive(AsRef)]
        struct FieldTypesWithInner {
            #[as_ref(i32, Foo)]
            first: Foo,
        }

        // Asserts that the macro expansion doesn't generate a blanket `AsRef`
        // impl forwarding to the field type, by producing a trait implementations
        // conflict error during compilation, if it does.
        impl AsRef<bool> for FieldTypesWithInner {
            fn as_ref(&self) -> &bool {
                self.first.as_ref()
            }
        }

        #[test]
        fn field_types_with_inner() {
            let item = FieldTypesWithInner {
                first: Foo(1, 2.0, false),
            };

            let rf: &i32 = item.as_ref();
            assert!(ptr::eq(rf, item.first.as_ref()));

            let rf: &Foo = item.as_ref();
            assert!(ptr::eq(rf, &item.first));
        }

        #[derive(AsRef)]
        struct FieldTypesWithRenamedInner {
            #[as_ref(i32, RenamedFoo)]
            first: Foo,
        }

        // Asserts that the macro expansion doesn't generate a blanket `AsRef`
        // impl forwarding to the field type, by producing a trait implementations
        // conflict error during compilation, if it does.
        impl AsRef<bool> for FieldTypesWithRenamedInner {
            fn as_ref(&self) -> &bool {
                self.first.as_ref()
            }
        }

        #[test]
        fn field_types_with_renamed_inner() {
            let item = FieldTypesWithRenamedInner {
                first: Foo(1, 2.0, false),
            };

            let rf: &i32 = item.as_ref();
            assert!(ptr::eq(rf, item.first.as_ref()));

            let rf: &Foo = item.as_ref();
            assert!(ptr::eq(rf, &item.first));
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

            #[derive(AsRef)]
            #[as_ref(i32, f64)]
            struct Types<T> {
                first: T,
            }

            // Asserts that the macro expansion doesn't generate a blanket `AsRef`
            // impl forwarding to the field type, by producing a trait implementations
            // conflict error during compilation, if it does.
            impl<T: AsRef<bool>> AsRef<bool> for Types<T> {
                fn as_ref(&self) -> &bool {
                    self.first.as_ref()
                }
            }

            #[test]
            fn types() {
                let item = Types {
                    first: Foo(1, 2.0, false),
                };

                let rf: &i32 = item.as_ref();
                assert!(ptr::eq(rf, item.first.as_ref()));

                let rf: &f64 = item.as_ref();
                assert!(ptr::eq(rf, item.first.as_ref()));
            }

            #[derive(AsRef)]
            #[as_ref(Vec<T>)]
            struct TypesInner<T> {
                first: Vec<T>,
            }

            #[test]
            fn types_inner() {
                let item = TypesInner { first: vec![1i32] };

                assert!(ptr::eq(item.as_ref(), &item.first));
            }

            #[derive(AsRef)]
            struct FieldTypes<T> {
                #[as_ref(i32, f64)]
                first: T,
            }

            // Asserts that the macro expansion doesn't generate a blanket `AsRef`
            // impl forwarding to the field type, by producing a trait implementations
            // conflict error during compilation, if it does.
            impl<T: AsRef<bool>> AsRef<bool> for FieldTypes<T> {
                fn as_ref(&self) -> &bool {
                    self.first.as_ref()
                }
            }

            #[test]
            fn field_types() {
                let item = FieldTypes {
                    first: Foo(1, 2.0, false),
                };

                let rf: &i32 = item.as_ref();
                assert!(ptr::eq(rf, item.first.as_ref()));

                let rf: &f64 = item.as_ref();
                assert!(ptr::eq(rf, item.first.as_ref()));
            }

            #[derive(AsRef)]
            struct FieldTypesInner<T> {
                #[as_ref(Vec<T>)]
                first: Vec<T>,
            }

            #[test]
            fn field_types_inner() {
                let item = FieldTypesInner { first: vec![1i32] };

                assert!(ptr::eq(item.as_ref(), &item.first));
            }

            #[derive(AsRef)]
            #[as_ref(i32)]
            struct Lifetime<'a> {
                first: Bar<&'a i32>,
            }

            #[test]
            fn lifetime() {
                let item = Lifetime { first: Bar(&1) };

                assert!(ptr::eq(item.as_ref(), item.first.as_ref()));
            }

            #[derive(AsRef)]
            struct FieldLifetime<'a> {
                #[as_ref(i32)]
                first: Bar<&'a i32>,
            }

            #[test]
            fn field_lifetime() {
                let item = FieldLifetime { first: Bar(&1) };

                assert!(ptr::eq(item.as_ref(), item.first.as_ref()));
            }

            #[derive(AsRef)]
            #[as_ref([i32])]
            struct ConstParam<const N: usize> {
                first: Bar<[i32; N]>,
            }

            #[test]
            fn const_param() {
                let item = ConstParam { first: Bar([]) };

                assert!(ptr::eq(item.as_ref(), item.first.as_ref()));
            }

            #[derive(AsRef)]
            struct FieldConstParam<const N: usize> {
                #[as_ref([i32])]
                first: Bar<[i32; N]>,
            }

            #[test]
            fn field_const_param() {
                let item = FieldConstParam { first: Bar([]) };

                assert!(ptr::eq(item.as_ref(), item.first.as_ref()));
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

        type RenamedString = String;

        #[derive(AsRef)]
        struct Types(
            #[as_ref(str, RenamedString)] String,
            #[as_ref([u8])] Vec<u8>,
        );

        // Asserts that the macro expansion doesn't generate `AsRef` impl for the field type, by
        // producing trait implementations conflict error during compilation, if it does.
        impl AsRef<Vec<u8>> for Types {
            fn as_ref(&self) -> &Vec<u8> {
                &self.1
            }
        }

        #[test]
        fn types() {
            let item = Types("test".to_owned(), vec![0]);

            let rf: &str = item.as_ref();
            assert!(ptr::eq(rf, item.0.as_ref()));

            let rf: &String = item.as_ref();
            assert!(ptr::eq(rf, &item.0));

            let rf: &[u8] = item.as_ref();
            assert!(ptr::eq(rf, item.1.as_ref()));
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

            #[derive(AsRef)]
            struct Types<T, U>(#[as_ref(str)] T, #[as_ref([u8])] U);

            #[test]
            fn types() {
                let item = Types("test".to_owned(), vec![0u8]);

                let rf: &str = item.as_ref();
                assert!(ptr::eq(rf, item.0.as_ref()));

                let rf: &[u8] = item.as_ref();
                assert!(ptr::eq(rf, item.1.as_ref()));
            }

            #[derive(AsRef)]
            struct TypesWithInner<T, U>(
                #[as_ref(Vec<T>, [T])] Vec<T>,
                #[as_ref(str)] U,
            );

            #[test]
            fn types_with_inner() {
                let item = TypesWithInner(vec![1i32], "a".to_owned());

                let rf: &Vec<i32> = item.as_ref();
                assert!(ptr::eq(rf, &item.0));

                let rf: &[i32] = item.as_ref();
                assert!(ptr::eq(rf, item.0.as_ref()));

                let rf: &str = item.as_ref();
                assert!(ptr::eq(rf, item.1.as_ref()));
            }

            #[derive(AsRef)]
            struct FieldNonGeneric<T>(#[as_ref([T])] Vec<i32>, T);

            #[test]
            fn field_non_generic() {
                let item = FieldNonGeneric(vec![], 2i32);

                assert!(ptr::eq(item.as_ref(), item.0.as_ref()));
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

        type RenamedString = String;

        #[derive(AsRef)]
        struct Types {
            #[as_ref(str, RenamedString)]
            first: String,
            #[as_ref([u8])]
            second: Vec<u8>,
        }

        // Asserts that the macro expansion doesn't generate `AsRef` impl for the field type, by
        // producing trait implementations conflict error during compilation, if it does.
        impl AsRef<Vec<u8>> for Types {
            fn as_ref(&self) -> &Vec<u8> {
                &self.second
            }
        }

        #[test]
        fn types() {
            let item = Types {
                first: "test".to_owned(),
                second: vec![0u8],
            };

            let rf: &str = item.as_ref();
            assert!(ptr::eq(rf, item.first.as_ref()));

            let rf: &String = item.as_ref();
            assert!(ptr::eq(rf, &item.first));

            let rf: &[u8] = item.as_ref();
            assert!(ptr::eq(rf, item.second.as_ref()));
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

            #[derive(AsRef)]
            struct Types<T, U> {
                #[as_ref(str)]
                first: T,
                #[as_ref([u8])]
                second: U,
            }

            #[test]
            fn types() {
                let item = Types {
                    first: "test".to_owned(),
                    second: vec![0u8],
                };

                let rf: &str = item.as_ref();
                assert!(ptr::eq(rf, item.first.as_ref()));

                let rf: &[u8] = item.as_ref();
                assert!(ptr::eq(rf, item.second.as_ref()));
            }

            #[derive(AsRef)]
            struct TypesWithInner<T, U> {
                #[as_ref(Vec<T>, [T])]
                first: Vec<T>,
                #[as_ref(str)]
                second: U,
            }

            #[test]
            fn types_with_inner() {
                let item = TypesWithInner {
                    first: vec![1i32],
                    second: "a".to_owned(),
                };

                let rf: &Vec<i32> = item.as_ref();
                assert!(ptr::eq(rf, &item.first));

                let rf: &[i32] = item.as_ref();
                assert!(ptr::eq(rf, item.first.as_ref()));

                let rf: &str = item.as_ref();
                assert!(ptr::eq(rf, item.second.as_ref()));
            }

            #[derive(AsRef)]
            struct FieldNonGeneric<T> {
                #[as_ref([T])]
                first: Vec<i32>,
                second: T,
            }

            #[test]
            fn field_non_generic() {
                let item = FieldNonGeneric {
                    first: vec![],
                    second: 2i32,
                };

                assert!(ptr::eq(item.as_ref(), item.first.as_ref()));
            }
        }
    }
}

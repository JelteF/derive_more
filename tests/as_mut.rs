#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(nightly, feature(never_type))]
#![allow(clippy::unnecessary_mut_passed)] // testing correct signatures rather than actual code
#![allow(dead_code)] // some code is tested for type checking only

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{borrow::ToOwned, collections::VecDeque, string::String, vec, vec::Vec};

#[cfg(feature = "std")]
use std::collections::VecDeque;

use core::{marker::PhantomData, ptr};

use derive_more::AsMut;

struct Helper(i32, f64, bool);

impl AsMut<i32> for Helper {
    fn as_mut(&mut self) -> &mut i32 {
        &mut self.0
    }
}

impl AsMut<f64> for Helper {
    fn as_mut(&mut self) -> &mut f64 {
        &mut self.1
    }
}

impl AsMut<bool> for Helper {
    fn as_mut(&mut self) -> &mut bool {
        &mut self.2
    }
}

struct LifetimeHelper<'a>(i32, PhantomData<&'a ()>);

impl LifetimeHelper<'static> {
    fn new(val: i32) -> Self {
        Self(val, PhantomData)
    }
}

impl AsMut<i32> for LifetimeHelper<'static> {
    fn as_mut(&mut self) -> &mut i32 {
        &mut self.0
    }
}

struct ConstParamHelper<const N: usize>([i32; N]);

impl AsMut<[i32]> for ConstParamHelper<0> {
    fn as_mut(&mut self) -> &mut [i32] {
        self.0.as_mut()
    }
}

mod single_field {
    use super::*;

    mod tuple {
        use super::*;

        #[test]
        fn nothing() {
            #[derive(AsMut)]
            struct Nothing(String);

            let mut item = Nothing("test".to_owned());

            assert!(ptr::eq(item.as_mut(), &mut item.0));
        }

        #[test]
        fn forward() {
            #[derive(AsMut)]
            #[as_mut(forward)]
            struct Forward(String);

            let mut item = Forward("test".to_owned());

            let rf: &mut str = item.as_mut();
            assert!(ptr::eq(rf, item.0.as_mut()));
        }

        #[test]
        fn field() {
            #[derive(AsMut)]
            struct Field(#[as_mut] String);

            let mut item = Field("test".to_owned());

            assert!(ptr::eq(item.as_mut(), &mut item.0));
        }

        #[test]
        fn field_forward() {
            #[derive(AsMut)]
            struct FieldForward(#[as_mut(forward)] String);

            let mut item = FieldForward("test".to_owned());

            let rf: &mut str = item.as_mut();
            assert!(ptr::eq(rf, item.0.as_mut()));
        }

        #[test]
        fn types() {
            #[derive(AsMut)]
            #[as_mut(i32, f64)]
            struct Types(Helper);

            // Asserts that the macro expansion doesn't generate a blanket `AsMut` impl forwarding
            // to the field type, by producing a trait implementations conflict error during
            // compilation, if it does.
            impl AsMut<bool> for Types {
                fn as_mut(&mut self) -> &mut bool {
                    self.0.as_mut()
                }
            }

            // Asserts that the macro expansion doesn't generate an `AsMut` impl for the field type,
            // by producing a trait implementations conflict error during compilation, if it does.
            impl AsMut<Helper> for Types {
                fn as_mut(&mut self) -> &mut Helper {
                    &mut self.0
                }
            }

            let mut item = Types(Helper(1, 2.0, false));

            let rf: &mut i32 = item.as_mut();
            assert!(ptr::eq(rf, item.0.as_mut()));

            let rf: &mut f64 = item.as_mut();
            assert!(ptr::eq(rf, item.0.as_mut()));
        }

        #[test]
        fn types_with_inner() {
            #[derive(AsMut)]
            #[as_mut(i32, Helper)]
            struct TypesWithInner(Helper);

            // Asserts that the macro expansion doesn't generate a blanket `AsMut` impl forwarding
            // to the field type, by producing a trait implementations conflict error during
            // compilation, if it does.
            impl AsMut<bool> for TypesWithInner {
                fn as_mut(&mut self) -> &mut bool {
                    self.0.as_mut()
                }
            }

            let mut item = TypesWithInner(Helper(1, 2.0, false));

            let rf: &mut i32 = item.as_mut();
            assert!(ptr::eq(rf, item.0.as_mut()));

            let rf: &mut Helper = item.as_mut();
            assert!(ptr::eq(rf, &mut item.0));
        }

        #[test]
        fn types_with_renamed_inner() {
            type RenamedFoo = Helper;

            #[derive(AsMut)]
            #[as_mut(i32, RenamedFoo)]
            struct TypesWithRenamedInner(Helper);

            // Asserts that the macro expansion doesn't generate a blanket `AsMut` impl forwarding
            // to the field type, by producing a trait implementations conflict error during
            // compilation, if it does.
            impl AsMut<bool> for TypesWithRenamedInner {
                fn as_mut(&mut self) -> &mut bool {
                    self.0.as_mut()
                }
            }

            let mut item = TypesWithRenamedInner(Helper(1, 2.0, false));

            let rf: &mut i32 = item.as_mut();
            assert!(ptr::eq(rf, item.0.as_mut()));

            let rf: &mut Helper = item.as_mut();
            assert!(ptr::eq(rf, &mut item.0));
        }

        #[test]
        fn field_types() {
            #[derive(AsMut)]
            struct FieldTypes(#[as_mut(i32, f64)] Helper);

            // Asserts that the macro expansion doesn't generate a blanket `AsMut` impl forwarding
            // to the field type, by producing a trait implementations conflict error during
            // compilation, if it does.
            impl AsMut<bool> for FieldTypes {
                fn as_mut(&mut self) -> &mut bool {
                    self.0.as_mut()
                }
            }

            // Asserts that the macro expansion doesn't generate an `AsMut` impl for the field type,
            // by producing a trait implementations conflict error during compilation, if it does.
            impl AsMut<Helper> for FieldTypes {
                fn as_mut(&mut self) -> &mut Helper {
                    &mut self.0
                }
            }

            let mut item = FieldTypes(Helper(1, 2.0, false));

            let rf: &mut i32 = item.as_mut();
            assert!(ptr::eq(rf, item.0.as_mut()));

            let rf: &mut f64 = item.as_mut();
            assert!(ptr::eq(rf, item.0.as_mut()));
        }

        #[test]
        fn field_types_with_inner() {
            #[derive(AsMut)]
            struct FieldTypesWithInner(#[as_mut(i32, Helper)] Helper);

            // Asserts that the macro expansion doesn't generate a blanket `AsMut` impl forwarding
            // to the field type, by producing a trait implementations conflict error during
            // compilation, if it does.
            impl AsMut<bool> for FieldTypesWithInner {
                fn as_mut(&mut self) -> &mut bool {
                    self.0.as_mut()
                }
            }

            let mut item = FieldTypesWithInner(Helper(1, 2.0, false));

            let rf: &mut i32 = item.as_mut();
            assert!(ptr::eq(rf, item.0.as_mut()));

            let rf: &mut Helper = item.as_mut();
            assert!(ptr::eq(rf, &mut item.0));
        }

        #[test]
        fn field_types_with_renamed_inner() {
            type RenamedFoo = Helper;

            #[derive(AsMut)]
            struct FieldTypesWithRenamedInner(#[as_mut(i32, RenamedFoo)] Helper);

            // Asserts that the macro expansion doesn't generate a blanket `AsMut` impl forwarding to
            // the field type, by producing a trait implementations conflict error during compilation,
            // if it does.
            impl AsMut<bool> for FieldTypesWithRenamedInner {
                fn as_mut(&mut self) -> &mut bool {
                    self.0.as_mut()
                }
            }

            let mut item = FieldTypesWithRenamedInner(Helper(1, 2.0, false));

            let rf: &mut i32 = item.as_mut();
            assert!(ptr::eq(rf, item.0.as_mut()));

            let rf: &mut Helper = item.as_mut();
            assert!(ptr::eq(rf, &mut item.0));
        }

        mod generic {
            use super::*;

            #[test]
            fn nothing() {
                #[derive(AsMut)]
                struct Nothing<T>(T);

                let mut item = Nothing("test".to_owned());

                assert!(ptr::eq(item.as_mut(), &mut item.0));
            }

            #[test]
            fn forward() {
                #[derive(AsMut)]
                #[as_mut(forward)]
                struct Forward<T>(T);

                let mut item = Forward("test".to_owned());

                let rf: &mut str = item.as_mut();
                assert!(ptr::eq(rf, item.0.as_mut()));
            }

            #[test]
            fn field() {
                #[derive(AsMut)]
                struct Field<T>(#[as_mut] T);

                let mut item = Field("test".to_owned());

                assert!(ptr::eq(item.as_mut(), &mut item.0));
            }

            #[test]
            fn field_forward() {
                #[derive(AsMut)]
                struct FieldForward<T>(#[as_mut(forward)] T);

                let mut item = FieldForward("test".to_owned());

                let rf: &mut str = item.as_mut();
                assert!(ptr::eq(rf, item.0.as_mut()));
            }

            #[test]
            fn types() {
                #[derive(AsMut)]
                #[as_mut(i32, f64)]
                struct Types<T>(T);

                // Asserts that the macro expansion doesn't generate a blanket `AsMut` impl
                // forwarding to the field type, by producing a trait implementations conflict error
                // during compilation, if it does.
                impl<T: AsMut<bool>> AsMut<bool> for Types<T> {
                    fn as_mut(&mut self) -> &mut bool {
                        self.0.as_mut()
                    }
                }

                let mut item = Types(Helper(1, 2.0, false));

                let rf: &mut i32 = item.as_mut();
                assert!(ptr::eq(rf, item.0.as_mut()));

                let rf: &mut f64 = item.as_mut();
                assert!(ptr::eq(rf, item.0.as_mut()));
            }

            #[test]
            fn types_inner() {
                #[derive(AsMut)]
                #[as_mut(Vec<T>)]
                struct TypesInner<T>(Vec<T>);

                let mut item = TypesInner(vec![1i32]);

                assert!(ptr::eq(item.as_mut(), &mut item.0));
            }

            #[test]
            fn field_types() {
                #[derive(AsMut)]
                struct FieldTypes<T>(#[as_mut(i32, f64)] T);

                // Asserts that the macro expansion doesn't generate a blanket `AsMut` impl forwarding
                // to the field type, by producing a trait implementations conflict error during
                // compilation, if it does.
                impl<T: AsMut<bool>> AsMut<bool> for FieldTypes<T> {
                    fn as_mut(&mut self) -> &mut bool {
                        self.0.as_mut()
                    }
                }

                let mut item = FieldTypes(Helper(1, 2.0, false));

                let rf: &mut i32 = item.as_mut();
                assert!(ptr::eq(rf, item.0.as_mut()));

                let rf: &mut f64 = item.as_mut();
                assert!(ptr::eq(rf, item.0.as_mut()));
            }

            #[test]
            fn field_types_inner() {
                #[derive(AsMut)]
                struct FieldTypesInner<T>(#[as_mut(Vec<T>)] Vec<T>);

                let mut item = FieldTypesInner(vec![1i32]);

                assert!(ptr::eq(item.as_mut(), &mut item.0));
            }

            #[test]
            fn lifetime() {
                #[derive(AsMut)]
                #[as_mut(i32)]
                struct Lifetime<'a>(LifetimeHelper<'a>);

                let mut item = Lifetime(LifetimeHelper::new(0));

                assert!(ptr::eq(item.as_mut(), item.0.as_mut()));
            }

            #[test]
            fn field_lifetime() {
                #[derive(AsMut)]
                struct FieldLifetime<'a>(#[as_mut(i32)] LifetimeHelper<'a>);

                let mut item = FieldLifetime(LifetimeHelper::new(0));

                assert!(ptr::eq(item.as_mut(), item.0.as_mut()));
            }

            #[test]
            fn const_param() {
                #[derive(AsMut)]
                #[as_mut([i32])]
                struct ConstParam<const N: usize>(ConstParamHelper<N>);

                let mut item = ConstParam(ConstParamHelper([]));

                assert!(ptr::eq(item.as_mut(), item.0.as_mut()));
            }

            #[test]
            fn field_const_param() {
                #[derive(AsMut)]
                struct FieldConstParam<const N: usize>(
                    #[as_mut([i32])] ConstParamHelper<N>,
                );

                let mut item = FieldConstParam(ConstParamHelper([]));

                assert!(ptr::eq(item.as_mut(), item.0.as_mut()));
            }
        }

        #[cfg(nightly)]
        mod never {
            use super::*;

            #[derive(AsMut)]
            struct Nothing(!);
        }

        mod deprecated {
            use super::*;

            #[derive(AsMut)]
            #[deprecated(note = "struct")]
            struct Deprecated(#[deprecated(note = "field")] i32);
        }
    }

    mod named {
        use super::*;

        #[test]
        fn nothing() {
            #[derive(AsMut)]
            struct Nothing {
                first: String,
            }

            let mut item = Nothing {
                first: "test".to_owned(),
            };

            assert!(ptr::eq(item.as_mut(), &mut item.first));
        }

        #[test]
        fn forward() {
            #[derive(AsMut)]
            #[as_mut(forward)]
            struct Forward {
                first: String,
            }

            let mut item = Forward {
                first: "test".to_owned(),
            };

            let rf: &mut str = item.as_mut();
            assert!(ptr::eq(rf, item.first.as_mut()));
        }

        #[test]
        fn field() {
            #[derive(AsMut)]
            struct Field {
                #[as_mut]
                first: String,
            }

            let mut item = Field {
                first: "test".to_owned(),
            };

            assert!(ptr::eq(item.as_mut(), &mut item.first));
        }

        #[test]
        fn field_forward() {
            #[derive(AsMut)]
            struct FieldForward {
                #[as_mut(forward)]
                first: String,
            }

            let mut item = FieldForward {
                first: "test".to_owned(),
            };

            let rf: &mut str = item.as_mut();
            assert!(ptr::eq(rf, item.first.as_mut()));
        }

        #[test]
        fn types() {
            #[derive(AsMut)]
            #[as_mut(i32, f64)]
            struct Types {
                first: Helper,
            }

            // Asserts that the macro expansion doesn't generate a blanket `AsMut` impl forwarding
            // to the field type, by producing a trait implementations conflict error during
            // compilation, if it does.
            impl AsMut<bool> for Types {
                fn as_mut(&mut self) -> &mut bool {
                    self.first.as_mut()
                }
            }

            // Asserts that the macro expansion doesn't generate an `AsMut` impl for the field type,
            // by producing a trait implementations conflict error during compilation, if it does.
            impl AsMut<Helper> for Types {
                fn as_mut(&mut self) -> &mut Helper {
                    &mut self.first
                }
            }

            let mut item = Types {
                first: Helper(1, 2.0, false),
            };

            let rf: &mut i32 = item.as_mut();
            assert!(ptr::eq(rf, item.first.as_mut()));

            let rf: &mut f64 = item.as_mut();
            assert!(ptr::eq(rf, item.first.as_mut()));
        }

        #[test]
        fn types_with_inner() {
            #[derive(AsMut)]
            #[as_mut(i32, Helper)]
            struct TypesWithInner {
                first: Helper,
            }

            // Asserts that the macro expansion doesn't generate a blanket `AsMut` impl forwarding
            // to the field type, by producing a trait implementations conflict error during
            // compilation, if it does.
            impl AsMut<bool> for TypesWithInner {
                fn as_mut(&mut self) -> &mut bool {
                    self.first.as_mut()
                }
            }

            let mut item = TypesWithInner {
                first: Helper(1, 2.0, false),
            };

            let rf: &mut i32 = item.as_mut();
            assert!(ptr::eq(rf, item.first.as_mut()));

            let rf: &mut Helper = item.as_mut();
            assert!(ptr::eq(rf, &mut item.first));
        }

        #[test]
        fn types_with_renamed_inner() {
            type RenamedFoo = Helper;

            #[derive(AsMut)]
            #[as_mut(i32, RenamedFoo)]
            struct TypesWithRenamedInner {
                first: Helper,
            }

            // Asserts that the macro expansion doesn't generate a blanket `AsMut` impl forwarding
            // to the field type, by producing a trait implementations conflict error during
            // compilation, if it does.
            impl AsMut<bool> for TypesWithRenamedInner {
                fn as_mut(&mut self) -> &mut bool {
                    self.first.as_mut()
                }
            }

            let mut item = TypesWithRenamedInner {
                first: Helper(1, 2.0, false),
            };

            let rf: &mut i32 = item.as_mut();
            assert!(ptr::eq(rf, item.first.as_mut()));

            let rf: &mut Helper = item.as_mut();
            assert!(ptr::eq(rf, &mut item.first));
        }

        #[test]
        fn field_types() {
            #[derive(AsMut)]
            struct FieldTypes {
                #[as_mut(i32, f64)]
                first: Helper,
            }

            // Asserts that the macro expansion doesn't generate a blanket `AsMut` impl forwarding
            // to the field type, by producing a trait implementations conflict error during
            // compilation, if it does.
            impl AsMut<bool> for FieldTypes {
                fn as_mut(&mut self) -> &mut bool {
                    self.first.as_mut()
                }
            }

            // Asserts that the macro expansion doesn't generate an `AsMut` impl for the field type,
            // by producing a trait implementations conflict error during compilation, if it does.
            impl AsMut<Helper> for FieldTypes {
                fn as_mut(&mut self) -> &mut Helper {
                    &mut self.first
                }
            }

            let mut item = FieldTypes {
                first: Helper(1, 2.0, false),
            };

            let rf: &mut i32 = item.as_mut();
            assert!(ptr::eq(rf, item.first.as_mut()));

            let rf: &mut f64 = item.as_mut();
            assert!(ptr::eq(rf, item.first.as_mut()));
        }

        #[test]
        fn field_types_with_inner() {
            #[derive(AsMut)]
            struct FieldTypesWithInner {
                #[as_mut(i32, Helper)]
                first: Helper,
            }

            // Asserts that the macro expansion doesn't generate a blanket `AsMut` impl forwarding
            // to the field type, by producing a trait implementations conflict error during
            // compilation, if it does.
            impl AsMut<bool> for FieldTypesWithInner {
                fn as_mut(&mut self) -> &mut bool {
                    self.first.as_mut()
                }
            }

            let mut item = FieldTypesWithInner {
                first: Helper(1, 2.0, false),
            };

            let rf: &mut i32 = item.as_mut();
            assert!(ptr::eq(rf, item.first.as_mut()));

            let rf: &mut Helper = item.as_mut();
            assert!(ptr::eq(rf, &mut item.first));
        }

        #[test]
        fn field_types_with_renamed_inner() {
            type RenamedFoo = Helper;

            #[derive(AsMut)]
            struct FieldTypesWithRenamedInner {
                #[as_mut(i32, RenamedFoo)]
                first: Helper,
            }

            // Asserts that the macro expansion doesn't generate a blanket `AsMut` impl forwarding
            // to the field type, by producing a trait implementations conflict error during
            // compilation, if it does.
            impl AsMut<bool> for FieldTypesWithRenamedInner {
                fn as_mut(&mut self) -> &mut bool {
                    self.first.as_mut()
                }
            }

            let mut item = FieldTypesWithRenamedInner {
                first: Helper(1, 2.0, false),
            };

            let rf: &mut i32 = item.as_mut();
            assert!(ptr::eq(rf, item.first.as_mut()));

            let rf: &mut Helper = item.as_mut();
            assert!(ptr::eq(rf, &mut item.first));
        }

        mod generic {
            use super::*;

            #[test]
            fn nothing() {
                #[derive(AsMut)]
                struct Nothing<T> {
                    first: T,
                }

                let mut item = Nothing {
                    first: "test".to_owned(),
                };

                assert!(ptr::eq(item.as_mut(), &mut item.first));
            }

            #[test]
            fn struct_forward() {
                #[derive(AsMut)]
                #[as_mut(forward)]
                struct Forward<T> {
                    first: T,
                }

                let mut item = Forward {
                    first: "test".to_owned(),
                };

                let rf: &mut str = item.as_mut();
                assert!(ptr::eq(rf, item.first.as_mut()));
            }

            #[test]
            fn field() {
                #[derive(AsMut)]
                struct Field<T> {
                    #[as_mut]
                    first: T,
                }

                let mut item = Field {
                    first: "test".to_owned(),
                };

                assert!(ptr::eq(item.as_mut(), &mut item.first));
            }

            #[test]
            fn field_forward() {
                #[derive(AsMut)]
                struct FieldForward<T> {
                    #[as_mut(forward)]
                    first: T,
                }

                let mut item = FieldForward {
                    first: "test".to_owned(),
                };

                let rf: &mut str = item.as_mut();
                assert!(ptr::eq(rf, item.first.as_mut()));
            }

            #[test]
            fn types() {
                #[derive(AsMut)]
                #[as_mut(i32, f64)]
                struct Types<T> {
                    first: T,
                }

                // Asserts that the macro expansion doesn't generate a blanket `AsMut` impl
                // forwarding to the field type, by producing a trait implementations conflict error
                // during compilation, if it does.
                impl<T: AsMut<bool>> AsMut<bool> for Types<T> {
                    fn as_mut(&mut self) -> &mut bool {
                        self.first.as_mut()
                    }
                }

                let mut item = Types {
                    first: Helper(1, 2.0, false),
                };

                let rf: &mut i32 = item.as_mut();
                assert!(ptr::eq(rf, item.first.as_mut()));

                let rf: &mut f64 = item.as_mut();
                assert!(ptr::eq(rf, item.first.as_mut()));
            }

            #[test]
            fn types_inner() {
                #[derive(AsMut)]
                #[as_mut(Vec<T>)]
                struct TypesInner<T> {
                    first: Vec<T>,
                }

                let mut item = TypesInner { first: vec![1i32] };

                assert!(ptr::eq(item.as_mut(), &mut item.first));
            }

            #[test]
            fn field_types() {
                #[derive(AsMut)]
                struct FieldTypes<T> {
                    #[as_mut(i32, f64)]
                    first: T,
                }

                // Asserts that the macro expansion doesn't generate a blanket `AsMut` impl
                // forwarding to the field type, by producing a trait implementations conflict error
                // during compilation, if it does.
                impl<T: AsMut<bool>> AsMut<bool> for FieldTypes<T> {
                    fn as_mut(&mut self) -> &mut bool {
                        self.first.as_mut()
                    }
                }

                let mut item = FieldTypes {
                    first: Helper(1, 2.0, false),
                };

                let rf: &mut i32 = item.as_mut();
                assert!(ptr::eq(rf, item.first.as_mut()));

                let rf: &mut f64 = item.as_mut();
                assert!(ptr::eq(rf, item.first.as_mut()));
            }

            #[test]
            fn field_types_inner() {
                #[derive(AsMut)]
                struct FieldTypesInner<T> {
                    #[as_mut(Vec<T>)]
                    first: Vec<T>,
                }

                let mut item = FieldTypesInner { first: vec![1i32] };

                assert!(ptr::eq(item.as_mut(), &mut item.first));
            }

            #[test]
            fn lifetime() {
                #[derive(AsMut)]
                #[as_mut(i32)]
                struct Lifetime<'a> {
                    first: LifetimeHelper<'a>,
                }

                let mut item = Lifetime {
                    first: LifetimeHelper::new(0),
                };

                assert!(ptr::eq(item.as_mut(), item.first.as_mut()));
            }

            #[test]
            fn field_lifetime() {
                #[derive(AsMut)]
                struct FieldLifetime<'a> {
                    #[as_mut(i32)]
                    first: LifetimeHelper<'a>,
                }

                let mut item = FieldLifetime {
                    first: LifetimeHelper::new(0),
                };

                assert!(ptr::eq(item.as_mut(), item.first.as_mut()));
            }

            #[test]
            fn const_param() {
                #[derive(AsMut)]
                #[as_mut([i32])]
                struct ConstParam<const N: usize> {
                    first: ConstParamHelper<N>,
                }

                let mut item = ConstParam {
                    first: ConstParamHelper([]),
                };

                assert!(ptr::eq(item.as_mut(), item.first.as_mut()));
            }

            #[test]
            fn field_const_param() {
                #[derive(AsMut)]
                struct FieldConstParam<const N: usize> {
                    #[as_mut([i32])]
                    first: ConstParamHelper<N>,
                }

                let mut item = FieldConstParam {
                    first: ConstParamHelper([]),
                };

                assert!(ptr::eq(item.as_mut(), item.first.as_mut()));
            }
        }

        #[cfg(nightly)]
        mod never {
            use super::*;

            #[derive(AsMut)]
            struct Nothing {
                first: !,
            }
        }

        mod deprecated {
            use super::*;

            #[derive(AsMut)]
            #[deprecated(note = "struct")]
            struct Deprecated {
                #[deprecated(note = "field")]
                field: i32,
            }
        }
    }
}

mod multi_field {
    use super::*;

    mod tuple {
        use super::*;

        #[test]
        fn nothing() {
            #[derive(AsMut)]
            struct Nothing(String, i32);

            let mut item = Nothing("test".to_owned(), 0);

            assert!(ptr::eq(item.as_mut(), &mut item.0));
            assert!(ptr::eq(item.as_mut(), &mut item.1));
        }

        #[test]
        fn skip() {
            #[derive(AsMut)]
            struct Skip(String, i32, #[as_mut(skip)] f64);

            // Asserts that the macro expansion doesn't generate `AsMut` impl for the skipped field,
            // by producing trait implementations conflict error during compilation, if it does.
            impl AsMut<f64> for Skip {
                fn as_mut(&mut self) -> &mut f64 {
                    &mut self.2
                }
            }

            let mut item = Skip("test".to_owned(), 0, 0.0);

            assert!(ptr::eq(item.as_mut(), &mut item.0));
            assert!(ptr::eq(item.as_mut(), &mut item.1));
        }

        #[test]
        fn field() {
            #[derive(AsMut)]
            struct Field(#[as_mut] String, #[as_mut] i32, f64);

            // Asserts that the macro expansion doesn't generate `AsMut` impl for the third field,
            // by producing trait implementations conflict error during compilation, if it does.
            impl AsMut<f64> for Field {
                fn as_mut(&mut self) -> &mut f64 {
                    &mut self.2
                }
            }

            let mut item = Field("test".to_owned(), 0, 0.0);

            assert!(ptr::eq(item.as_mut(), &mut item.0));
            assert!(ptr::eq(item.as_mut(), &mut item.1));
        }

        #[test]
        fn field_forward() {
            #[derive(AsMut)]
            struct FieldForward(#[as_mut(forward)] String, i32);

            let mut item = FieldForward("test".to_owned(), 0);

            let rf: &mut str = item.as_mut();
            assert!(ptr::eq(rf, item.0.as_mut()));
        }

        #[test]
        fn types() {
            type RenamedString = String;

            #[derive(AsMut)]
            struct Types(
                #[as_mut(str, RenamedString)] String,
                #[as_mut([u8])] Vec<u8>,
            );

            // Asserts that the macro expansion doesn't generate `AsMut` impl for the field type, by
            // producing trait implementations conflict error during compilation, if it does.
            impl AsMut<Vec<u8>> for Types {
                fn as_mut(&mut self) -> &mut Vec<u8> {
                    &mut self.1
                }
            }

            let mut item = Types("test".to_owned(), vec![0]);

            let rf: &mut str = item.as_mut();
            assert!(ptr::eq(rf, item.0.as_mut()));

            let rf: &mut String = item.as_mut();
            assert!(ptr::eq(rf, &mut item.0));

            let rf: &mut [u8] = item.as_mut();
            assert!(ptr::eq(rf, item.1.as_mut()));
        }

        mod generic {
            use super::*;

            #[test]
            fn nothing() {
                #[derive(AsMut)]
                struct Nothing<T, U>(Vec<T>, VecDeque<U>);

                let mut item = Nothing(vec![1], VecDeque::from([2]));

                assert!(ptr::eq(item.as_mut(), &mut item.0));
                assert!(ptr::eq(item.as_mut(), &mut item.1));
            }

            #[test]
            fn skip() {
                #[derive(AsMut)]
                struct Skip<T, U, V>(Vec<T>, VecDeque<U>, #[as_mut(skip)] V);

                let mut item = Skip(vec![1], VecDeque::from([2]), 0);

                assert!(ptr::eq(item.as_mut(), &mut item.0));
                assert!(ptr::eq(item.as_mut(), &mut item.1));
            }

            #[test]
            fn field() {
                #[derive(AsMut)]
                struct Field<T, U, V>(#[as_mut] Vec<T>, #[as_mut] VecDeque<U>, V);

                let mut item = Field(vec![1], VecDeque::from([2]), 0);

                assert!(ptr::eq(item.as_mut(), &mut item.0));
                assert!(ptr::eq(item.as_mut(), &mut item.1));
            }

            #[test]
            fn field_forward() {
                #[derive(AsMut)]
                struct FieldForward<T, U>(#[as_mut(forward)] T, U);

                let mut item = FieldForward("test".to_owned(), 0);

                let rf: &mut str = item.as_mut();
                assert!(ptr::eq(rf, item.0.as_mut()));
            }

            #[test]
            fn types() {
                #[derive(AsMut)]
                struct Types<T, U>(#[as_mut(str)] T, #[as_mut([u8])] U);

                let mut item = Types("test".to_owned(), vec![0]);

                let rf: &mut str = item.as_mut();
                assert!(ptr::eq(rf, item.0.as_mut()));

                let rf: &mut [u8] = item.as_mut();
                assert!(ptr::eq(rf, item.1.as_mut()));
            }

            #[test]
            fn types_with_inner() {
                #[derive(AsMut)]
                struct TypesWithInner<T, U>(
                    #[as_mut(Vec<T>, [T])] Vec<T>,
                    #[as_mut(str)] U,
                );

                let mut item = TypesWithInner(vec![1i32], "a".to_owned());

                let rf: &mut Vec<i32> = item.as_mut();
                assert!(ptr::eq(rf, &mut item.0));

                let rf: &mut [i32] = item.as_mut();
                assert!(ptr::eq(rf, item.0.as_mut()));

                let rf: &mut str = item.as_mut();
                assert!(ptr::eq(rf, item.1.as_mut()));
            }

            #[test]
            fn field_non_generic() {
                #[derive(AsMut)]
                struct FieldNonGeneric<T>(#[as_mut([T])] Vec<i32>, T);

                let mut item = FieldNonGeneric(vec![], 2i32);

                assert!(ptr::eq(item.as_mut(), item.0.as_mut()));
            }
        }

        #[cfg(nightly)]
        mod never {
            use super::*;

            #[derive(AsMut)]
            struct Nothing(String, !);
        }
    }

    mod named {
        use super::*;

        #[test]
        fn nothing() {
            #[derive(AsMut)]
            struct Nothing {
                first: String,
                second: i32,
            }

            let mut item = Nothing {
                first: "test".to_owned(),
                second: 0,
            };

            assert!(ptr::eq(item.as_mut(), &mut item.first));
            assert!(ptr::eq(item.as_mut(), &mut item.second));
        }

        #[test]
        fn skip() {
            #[derive(AsMut)]
            struct Skip {
                first: String,
                second: i32,
                #[as_mut(skip)]
                third: f64,
            }

            // Asserts that the macro expansion doesn't generate `AsMut` impl for the skipped field,
            // by producing trait implementations conflict error during compilation, if it does.
            impl AsMut<f64> for Skip {
                fn as_mut(&mut self) -> &mut f64 {
                    &mut self.third
                }
            }

            let mut item = Skip {
                first: "test".to_owned(),
                second: 0,
                third: 0.0,
            };

            assert!(ptr::eq(item.as_mut(), &mut item.first));
            assert!(ptr::eq(item.as_mut(), &mut item.second));
        }

        #[test]
        fn field() {
            #[derive(AsMut)]
            struct Field {
                #[as_mut]
                first: String,
                #[as_mut]
                second: i32,
                third: f64,
            }

            // Asserts that the macro expansion doesn't generate `AsMut` impl for the `third` field,
            // by producing trait implementations conflict error during compilation, if it does.
            impl AsMut<f64> for Field {
                fn as_mut(&mut self) -> &mut f64 {
                    &mut self.third
                }
            }

            let mut item = Field {
                first: "test".to_owned(),
                second: 0,
                third: 0.0,
            };

            assert!(ptr::eq(item.as_mut(), &mut item.first));
            assert!(ptr::eq(item.as_mut(), &mut item.second));
        }

        #[test]
        fn field_forward() {
            #[derive(AsMut)]
            struct FieldForward {
                #[as_mut(forward)]
                first: String,
                second: i32,
            }

            let mut item = FieldForward {
                first: "test".to_owned(),
                second: 0,
            };

            let rf: &mut str = item.as_mut();
            assert!(ptr::eq(rf, item.first.as_mut()));
        }

        #[test]
        fn types() {
            type RenamedString = String;

            #[derive(AsMut)]
            struct Types {
                #[as_mut(str, RenamedString)]
                first: String,
                #[as_mut([u8])]
                second: Vec<u8>,
            }

            // Asserts that the macro expansion doesn't generate `AsMut` impl for unmentioned type,
            // by producing trait implementations conflict error during compilation, if it does.
            impl AsMut<Vec<u8>> for Types {
                fn as_mut(&mut self) -> &mut Vec<u8> {
                    &mut self.second
                }
            }

            let mut item = Types {
                first: "test".to_owned(),
                second: vec![0],
            };

            let rf: &mut str = item.as_mut();
            assert!(ptr::eq(rf, item.first.as_mut()));

            let rf: &mut String = item.as_mut();
            assert!(ptr::eq(rf, &mut item.first));

            let rf: &mut [u8] = item.as_mut();
            assert!(ptr::eq(rf, item.second.as_mut()));
        }

        mod generic {
            use super::*;

            #[test]
            fn nothing() {
                #[derive(AsMut)]
                struct Nothing<T, U> {
                    first: Vec<T>,
                    second: VecDeque<U>,
                }

                let mut item = Nothing {
                    first: vec![1],
                    second: VecDeque::from([2]),
                };

                assert!(ptr::eq(item.as_mut(), &mut item.first));
                assert!(ptr::eq(item.as_mut(), &mut item.second));
            }

            #[test]
            fn skip() {
                #[derive(AsMut)]
                struct Skip<T, U, V> {
                    first: Vec<T>,
                    second: VecDeque<U>,
                    #[as_mut(skip)]
                    third: V,
                }

                let mut item = Skip {
                    first: vec![1],
                    second: VecDeque::from([2]),
                    third: 0,
                };

                assert!(ptr::eq(item.as_mut(), &mut item.first));
                assert!(ptr::eq(item.as_mut(), &mut item.second));
            }

            #[test]
            fn field() {
                #[derive(AsMut)]
                struct Field<T, U, V> {
                    #[as_mut]
                    first: Vec<T>,
                    #[as_mut]
                    second: VecDeque<U>,
                    third: V,
                }

                let mut item = Field {
                    first: vec![1],
                    second: VecDeque::from([2]),
                    third: 0,
                };

                assert!(ptr::eq(item.as_mut(), &mut item.first));
                assert!(ptr::eq(item.as_mut(), &mut item.second));
            }

            #[test]
            fn field_forward() {
                #[derive(AsMut)]
                struct FieldForward<T, U> {
                    #[as_mut(forward)]
                    first: T,
                    second: U,
                }

                let mut item = FieldForward {
                    first: "test".to_owned(),
                    second: 0,
                };

                let rf: &mut str = item.as_mut();
                assert!(ptr::eq(rf, item.first.as_mut()));
            }

            #[test]
            fn types() {
                #[derive(AsMut)]
                struct Types<T, U> {
                    #[as_mut(str)]
                    first: T,
                    #[as_mut([u8])]
                    second: U,
                }

                let mut item = Types {
                    first: "test".to_owned(),
                    second: vec![0],
                };

                let rf: &mut str = item.as_mut();
                assert!(ptr::eq(rf, item.first.as_mut()));

                let rf: &mut [u8] = item.as_mut();
                assert!(ptr::eq(rf, item.second.as_mut()));
            }

            #[test]
            fn types_inner() {
                #[derive(AsMut)]
                struct TypesWithInner<T, U> {
                    #[as_mut(Vec<T>, [T])]
                    first: Vec<T>,
                    #[as_mut(str)]
                    second: U,
                }

                let mut item = TypesWithInner {
                    first: vec![1i32],
                    second: "a".to_owned(),
                };

                let rf: &mut Vec<i32> = item.as_mut();
                assert!(ptr::eq(rf, &mut item.first));

                let rf: &mut [i32] = item.as_mut();
                assert!(ptr::eq(rf, item.first.as_mut()));

                let rf: &mut str = item.as_mut();
                assert!(ptr::eq(rf, item.second.as_mut()));
            }

            #[test]
            fn field_non_generic() {
                #[derive(AsMut)]
                struct FieldNonGeneric<T> {
                    #[as_mut([T])]
                    first: Vec<i32>,
                    second: T,
                }

                let mut item = FieldNonGeneric {
                    first: vec![],
                    second: 2i32,
                };

                assert!(ptr::eq(item.as_mut(), item.first.as_mut()));
            }
        }

        #[cfg(nightly)]
        mod never {
            use super::*;

            #[derive(AsMut)]
            struct Nothing {
                first: !,
                second: i32,
            }
        }
    }
}

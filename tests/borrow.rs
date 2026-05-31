#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(nightly, feature(never_type))]
#![allow(dead_code)] // some code is tested for type checking only

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{borrow::ToOwned, string::String, vec, vec::Vec};

#[cfg(feature = "std")]
use std::borrow::ToOwned;

use core::{borrow::Borrow as _, ptr};

use derive_more::Borrow;

mod single_field {
    use super::*;

    #[test]
    fn tuple() {
        #[derive(Borrow)]
        struct Tuple(String);

        let item = Tuple("test".to_owned());
        let borrowed: &String = item.borrow();

        assert!(ptr::eq(borrowed, &item.0));
    }

    #[test]
    fn named() {
        #[derive(Borrow)]
        struct Named {
            value: String,
        }

        let item = Named {
            value: "test".to_owned(),
        };
        let borrowed: &String = item.borrow();

        assert!(ptr::eq(borrowed, &item.value));
    }

    #[test]
    fn generic() {
        #[derive(Borrow)]
        struct Generic<T>(T);

        let item = Generic("test".to_owned());
        let borrowed: &String = item.borrow();

        assert!(ptr::eq(borrowed, &item.0));
    }

    #[test]
    fn field() {
        #[derive(Borrow)]
        struct Field(#[borrow] String);

        let item = Field("test".to_owned());
        let borrowed: &String = item.borrow();

        assert!(ptr::eq(borrowed, &item.0));
    }

    #[test]
    fn named_field() {
        #[derive(Borrow)]
        struct NamedField {
            #[borrow]
            value: String,
        }

        let item = NamedField {
            value: "test".to_owned(),
        };
        let borrowed: &String = item.borrow();

        assert!(ptr::eq(borrowed, &item.value));
    }

    #[test]
    fn lifetime() {
        #[derive(Borrow)]
        struct Lifetime<'a>(&'a i32);

        let value = 1;
        let item = Lifetime(&value);
        let borrowed: &&i32 = item.borrow();

        assert!(ptr::eq::<i32>(*borrowed, item.0));
    }

    #[test]
    fn field_lifetime() {
        #[derive(Borrow)]
        struct FieldLifetime<'a>(#[borrow] &'a i32);

        let value = 1;
        let item = FieldLifetime(&value);
        let borrowed: &&i32 = item.borrow();

        assert!(ptr::eq::<i32>(*borrowed, item.0));
    }

    #[test]
    fn named_lifetime() {
        #[derive(Borrow)]
        struct NamedLifetime<'a> {
            value: &'a i32,
        }

        let value = 1;
        let item = NamedLifetime { value: &value };
        let borrowed: &&i32 = item.borrow();

        assert!(ptr::eq::<i32>(*borrowed, item.value));
    }

    #[test]
    fn named_field_lifetime() {
        #[derive(Borrow)]
        struct NamedFieldLifetime<'a> {
            #[borrow]
            value: &'a i32,
        }

        let value = 1;
        let item = NamedFieldLifetime { value: &value };
        let borrowed: &&i32 = item.borrow();

        assert!(ptr::eq::<i32>(*borrowed, item.value));
    }

    #[test]
    fn const_param() {
        #[derive(Borrow)]
        struct ConstParam<const N: usize>([i32; N]);

        let item = ConstParam([1, 2]);
        let borrowed: &[i32; 2] = item.borrow();

        assert!(ptr::eq(borrowed, &item.0));
    }

    #[test]
    fn named_const_param() {
        #[derive(Borrow)]
        struct NamedConstParam<const N: usize> {
            value: [i32; N],
        }

        let item = NamedConstParam { value: [1, 2] };
        let borrowed: &[i32; 2] = item.borrow();

        assert!(ptr::eq(borrowed, &item.value));
    }

    #[test]
    fn field_const_param() {
        #[derive(Borrow)]
        struct FieldConstParam<const N: usize>(#[borrow] [i32; N]);

        let item = FieldConstParam([1, 2]);
        let borrowed: &[i32; 2] = item.borrow();

        assert!(ptr::eq(borrowed, &item.0));
    }

    #[test]
    fn named_field_const_param() {
        #[derive(Borrow)]
        struct NamedFieldConstParam<const N: usize> {
            #[borrow]
            value: [i32; N],
        }

        let item = NamedFieldConstParam { value: [1, 2] };
        let borrowed: &[i32; 2] = item.borrow();

        assert!(ptr::eq(borrowed, &item.value));
    }

    #[test]
    fn forward() {
        #[derive(Borrow)]
        #[borrow(forward)]
        struct Forward(String);

        let item = Forward("test".to_owned());
        let borrowed: &str = item.borrow();

        assert!(ptr::eq::<str>(borrowed, item.0.borrow()));
    }

    #[test]
    fn named_forward() {
        #[derive(Borrow)]
        #[borrow(forward)]
        struct NamedForward {
            value: String,
        }

        let item = NamedForward {
            value: "test".to_owned(),
        };
        let borrowed: &str = item.borrow();

        assert!(ptr::eq::<str>(borrowed, item.value.borrow()));
    }

    #[test]
    fn field_forward() {
        #[derive(Borrow)]
        struct FieldForward(#[borrow(forward)] String);

        let item = FieldForward("test".to_owned());
        let borrowed: &str = item.borrow();

        assert!(ptr::eq::<str>(borrowed, item.0.borrow()));
    }

    #[test]
    fn named_field_forward() {
        #[derive(Borrow)]
        struct NamedFieldForward {
            #[borrow(forward)]
            value: String,
        }

        let item = NamedFieldForward {
            value: "test".to_owned(),
        };
        let borrowed: &str = item.borrow();

        assert!(ptr::eq::<str>(borrowed, item.value.borrow()));
    }

    #[test]
    fn forward_generic_container() {
        #[derive(Borrow)]
        #[borrow(forward)]
        struct Forward<T>(Vec<T>);

        let item = Forward(vec![1, 2, 3]);
        let borrowed: &[i32] = item.borrow();

        assert!(ptr::eq::<[i32]>(borrowed, item.0.as_slice()));
    }

    #[test]
    fn forward_with_internal_generic_name_collision() {
        #[derive(Borrow)]
        #[borrow(forward)]
        struct Forward<__BorrowT>(Vec<__BorrowT>);

        let item = Forward(vec![1, 2, 3]);
        let borrowed: &[i32] = item.borrow();

        assert!(ptr::eq::<[i32]>(borrowed, item.0.as_slice()));
    }

    #[cfg(nightly)]
    mod never {
        use super::*;

        #[derive(Borrow)]
        struct Nothing(!);
    }

    mod deprecated {
        use super::*;

        #[derive(Borrow)]
        #[deprecated(note = "struct")]
        struct Deprecated(#[deprecated(note = "field")] i32);

        #[derive(Borrow)]
        #[deprecated(note = "struct")]
        struct DeprecatedNamed {
            #[deprecated(note = "field")]
            field: i32,
        }
    }

    mod trait_object {
        use super::*;

        use core::fmt::Debug;

        #[derive(Borrow)]
        struct DynDebug(dyn Debug);

        #[derive(Borrow)]
        struct DynDebugSend(dyn Debug + Send);

        #[derive(Borrow)]
        struct DynDebugLifetime<'a>(dyn Debug + 'a);

        #[test]
        fn direct() {
            fn assert_debug<T: ?Sized + core::borrow::Borrow<dyn Debug>>() {}
            fn assert_debug_send<T: ?Sized + core::borrow::Borrow<dyn Debug + Send>>() {
            }
            fn assert_debug_lifetime<
                'a,
                T: ?Sized + core::borrow::Borrow<dyn Debug + 'a>,
            >() {
            }

            assert_debug::<DynDebug>();
            assert_debug_send::<DynDebugSend>();
            assert_debug_lifetime::<DynDebugLifetime<'static>>();
        }
    }
}

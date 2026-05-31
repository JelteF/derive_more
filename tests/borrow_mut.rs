#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(nightly, feature(never_type))]
#![allow(dead_code)] // some code is tested for type checking only

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{borrow::ToOwned, string::String, vec, vec::Vec};

#[cfg(feature = "std")]
use std::borrow::ToOwned;

use core::{borrow::BorrowMut as _, ptr};

use derive_more::{Borrow, BorrowMut};

mod single_field {
    use super::*;

    #[test]
    fn tuple() {
        #[derive(Borrow, BorrowMut)]
        struct Tuple(String);

        let mut item = Tuple("test".to_owned());
        let field = ptr::addr_of_mut!(item.0);
        let borrowed: &mut String = item.borrow_mut();

        assert!(ptr::eq(borrowed, field));
        borrowed.push('!');
        assert_eq!(item.0, "test!");
    }

    #[test]
    fn named() {
        #[derive(Borrow, BorrowMut)]
        struct Named {
            value: String,
        }

        let mut item = Named {
            value: "test".to_owned(),
        };
        let field = ptr::addr_of_mut!(item.value);
        let borrowed: &mut String = item.borrow_mut();

        assert!(ptr::eq(borrowed, field));
        borrowed.push('!');
        assert_eq!(item.value, "test!");
    }

    #[test]
    fn generic() {
        #[derive(Borrow, BorrowMut)]
        struct Generic<T>(T);

        let mut item = Generic("test".to_owned());
        let field = ptr::addr_of_mut!(item.0);
        let borrowed: &mut String = item.borrow_mut();

        assert!(ptr::eq(borrowed, field));
        borrowed.push('!');
        assert_eq!(item.0, "test!");
    }

    #[test]
    fn field() {
        #[derive(Borrow, BorrowMut)]
        struct Field(#[borrow_mut] String);

        let mut item = Field("test".to_owned());
        let field = ptr::addr_of_mut!(item.0);
        let borrowed: &mut String = item.borrow_mut();

        assert!(ptr::eq(borrowed, field));
        borrowed.push('!');
        assert_eq!(item.0, "test!");
    }

    #[test]
    fn named_field() {
        #[derive(Borrow, BorrowMut)]
        struct NamedField {
            #[borrow_mut]
            value: String,
        }

        let mut item = NamedField {
            value: "test".to_owned(),
        };
        let field = ptr::addr_of_mut!(item.value);
        let borrowed: &mut String = item.borrow_mut();

        assert!(ptr::eq(borrowed, field));
        borrowed.push('!');
        assert_eq!(item.value, "test!");
    }

    #[test]
    fn lifetime() {
        #[derive(Borrow, BorrowMut)]
        struct Lifetime<'a>(&'a mut i32);

        let mut value = 1;
        let mut item = Lifetime(&mut value);
        let field = ptr::addr_of_mut!(item.0);
        let borrowed: &mut &mut i32 = item.borrow_mut();

        assert!(ptr::eq(borrowed, field));
        **borrowed = 2;
        assert_eq!(value, 2);
    }

    #[test]
    fn const_param() {
        #[derive(Borrow, BorrowMut)]
        struct ConstParam<const N: usize>([i32; N]);

        let mut item = ConstParam([1, 2]);
        let field = ptr::addr_of_mut!(item.0);
        let borrowed: &mut [i32; 2] = item.borrow_mut();

        assert!(ptr::eq(borrowed, field));
        borrowed[0] = 3;
        assert_eq!(item.0, [3, 2]);
    }

    #[test]
    fn forward() {
        #[derive(Borrow, BorrowMut)]
        #[borrow(forward)]
        #[borrow_mut(forward)]
        struct Forward(String);

        let mut item = Forward("test".to_owned());
        let field = item.0.as_mut_str() as *mut str;
        let borrowed: &mut str = item.borrow_mut();

        assert!(ptr::eq::<str>(borrowed, field));
        borrowed.make_ascii_uppercase();
        assert_eq!(item.0, "TEST");
    }

    #[test]
    fn named_forward() {
        #[derive(Borrow, BorrowMut)]
        #[borrow(forward)]
        #[borrow_mut(forward)]
        struct NamedForward {
            value: String,
        }

        let mut item = NamedForward {
            value: "test".to_owned(),
        };
        let field = item.value.as_mut_str() as *mut str;
        let borrowed: &mut str = item.borrow_mut();

        assert!(ptr::eq::<str>(borrowed, field));
        borrowed.make_ascii_uppercase();
        assert_eq!(item.value, "TEST");
    }

    #[test]
    fn field_forward() {
        #[derive(Borrow, BorrowMut)]
        struct FieldForward(
            #[borrow(forward)]
            #[borrow_mut(forward)]
            String,
        );

        let mut item = FieldForward("test".to_owned());
        let field = item.0.as_mut_str() as *mut str;
        let borrowed: &mut str = item.borrow_mut();

        assert!(ptr::eq::<str>(borrowed, field));
        borrowed.make_ascii_uppercase();
        assert_eq!(item.0, "TEST");
    }

    #[test]
    fn named_field_forward() {
        #[derive(Borrow, BorrowMut)]
        struct NamedFieldForward {
            #[borrow(forward)]
            #[borrow_mut(forward)]
            value: String,
        }

        let mut item = NamedFieldForward {
            value: "test".to_owned(),
        };
        let field = item.value.as_mut_str() as *mut str;
        let borrowed: &mut str = item.borrow_mut();

        assert!(ptr::eq::<str>(borrowed, field));
        borrowed.make_ascii_uppercase();
        assert_eq!(item.value, "TEST");
    }

    #[test]
    fn forward_generic_container() {
        #[derive(Borrow, BorrowMut)]
        #[borrow(forward)]
        #[borrow_mut(forward)]
        struct Forward<T>(Vec<T>);

        let mut item = Forward(vec![1, 2, 3]);
        let field = item.0.as_mut_slice() as *mut [i32];
        let borrowed: &mut [i32] = item.borrow_mut();

        assert!(ptr::eq::<[i32]>(borrowed, field));
        borrowed[0] = 4;
        assert_eq!(item.0.as_slice(), &[4, 2, 3]);
    }

    #[test]
    fn forward_with_internal_generic_name_collision() {
        #[derive(Borrow, BorrowMut)]
        #[borrow(forward)]
        #[borrow_mut(forward)]
        struct Forward<__BorrowMutT>(Vec<__BorrowMutT>);

        let mut item = Forward(vec![1, 2, 3]);
        let field = item.0.as_mut_slice() as *mut [i32];
        let borrowed: &mut [i32] = item.borrow_mut();

        assert!(ptr::eq::<[i32]>(borrowed, field));
        borrowed[0] = 4;
        assert_eq!(item.0.as_slice(), &[4, 2, 3]);
    }

    #[cfg(nightly)]
    mod never {
        use super::*;

        #[derive(Borrow, BorrowMut)]
        struct Nothing(!);
    }

    mod deprecated {
        use super::*;

        #[derive(Borrow, BorrowMut)]
        #[deprecated(note = "struct")]
        struct Deprecated(#[deprecated(note = "field")] i32);

        #[derive(Borrow, BorrowMut)]
        #[deprecated(note = "struct")]
        struct DeprecatedNamed {
            #[deprecated(note = "field")]
            field: i32,
        }
    }

    mod trait_object {
        use super::*;

        use core::fmt::Debug;

        #[derive(Borrow, BorrowMut)]
        struct DynDebug(dyn Debug);

        #[derive(Borrow, BorrowMut)]
        struct DynDebugSend(dyn Debug + Send);

        #[derive(Borrow, BorrowMut)]
        struct DynDebugLifetime<'a>(dyn Debug + 'a);

        #[test]
        fn direct() {
            fn assert_debug<T: ?Sized + core::borrow::BorrowMut<dyn Debug>>() {}
            fn assert_debug_send<
                T: ?Sized + core::borrow::BorrowMut<dyn Debug + Send>,
            >() {
            }
            fn assert_debug_lifetime<
                'a,
                T: ?Sized + core::borrow::BorrowMut<dyn Debug + 'a>,
            >() {
            }

            assert_debug::<DynDebug>();
            assert_debug_send::<DynDebugSend>();
            assert_debug_lifetime::<DynDebugLifetime<'static>>();
        }
    }
}

mod with_trait {
    use super::*;
    use derive_more::with_trait::{Borrow, BorrowMut};

    #[derive(Borrow, BorrowMut)]
    struct Tuple(String);

    #[test]
    fn reexports_trait_and_derive() {
        let mut item = Tuple("test".to_owned());
        let borrowed: &mut String = BorrowMut::borrow_mut(&mut item);

        borrowed.push('!');
        assert_eq!(item.0, "test!");
    }
}

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;

mod from;
mod add_like;
mod add_assign_like;
mod mul_like;
mod not_like;

macro_rules! create_derive(
    ($mod_:ident, $trait_:ident, $fn_name: ident) => {
        #[proc_macro_derive($trait_)]
        pub fn $fn_name(input: TokenStream) -> TokenStream {
            let s = input.to_string();
            let ast = syn::parse_macro_input(&s).unwrap();
            $mod_::expand(&ast, stringify!($trait_)).parse().unwrap()
        }
    }
);

create_derive!(from, From, from_derive);

create_derive!(add_like, Add, add_derive);
create_derive!(add_like, Sub, sub_derive);
create_derive!(add_like, BitAnd, bit_and_derive);
create_derive!(add_like, BitOr, bit_or_derive);
create_derive!(add_like, BitXor, bit_xor_derive);

create_derive!(mul_like, Mul, mul_derive);
create_derive!(mul_like, Div, div_derive);
create_derive!(mul_like, Rem, rem_derive);
create_derive!(mul_like, Shr, shr_derive);
create_derive!(mul_like, Shl, shl_derive);

create_derive!(not_like, Not, not_derive);
create_derive!(not_like, Neg, neg_derive);

create_derive!(add_assign_like, AddAssign,    add_assign_derive);
create_derive!(add_assign_like, SubAssign,    sub_assign_derive);
create_derive!(add_assign_like, BitAndAssign, bit_and_assign_derive);
create_derive!(add_assign_like, BitOrAssign,  bit_or_assign_derive);
create_derive!(add_assign_like, BitXorAssign, bit_xor_assign_derive);

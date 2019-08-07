#![no_std]

#[macro_use]
extern crate derive_more;

#[cfg_attr(feature = "add_assign_like", derive(AddAssign))]
#[cfg_attr(feature = "mul_assign_like", derive(MulAssign))]
#[cfg_attr(feature = "add_like", derive(Add))]
#[cfg_attr(feature = "mul_like", derive(Mul))]
#[cfg_attr(feature = "not_like", derive(Not))]
#[cfg_attr(feature = "index", derive(Index))]
#[cfg_attr(feature = "display", derive(Display))]
#[cfg_attr(feature = "from_str", derive(FromStr))]
#[cfg_attr(feature = "into", derive(Into))]
#[cfg_attr(feature = "from", derive(From))]
#[cfg_attr(feature = "index_mut", derive(IndexMut))]
#[cfg_attr(feature = "constructor", derive(Constructor))]
struct MyInts(u64);

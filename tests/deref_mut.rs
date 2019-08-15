#![allow(dead_code, unused_imports)]
#[macro_use]
extern crate derive_more;

#[derive(DerefMut)]
struct MyBoxedInt(Box<i32>);
// Deref implementation is needed for DerefMut
impl ::std::ops::Deref for MyBoxedInt {
    type Target = <Box<i32> as ::std::ops::Deref>::Target;
    #[inline]
    fn deref(&self) -> &Self::Target {
        <Box<i32> as ::std::ops::Deref>::deref(&self.0)
    }
}

#[derive(DerefMut)]
struct NumRef<'a> {
    num: &'a mut i32,
}
// Deref implementation is needed for DerefMut
impl<'a> ::std::ops::Deref for NumRef<'a> {
    type Target = <&'a mut i32 as ::std::ops::Deref>::Target;
    #[inline]
    fn deref(&self) -> &Self::Target {
        <&'a mut i32 as ::std::ops::Deref>::deref(&self.num)
    }
}

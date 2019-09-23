#![allow(dead_code)]
#[macro_use]
extern crate derive_more;

#[derive(DerefMutToInner)]
struct MyInt(i32);

// Deref implementation is needed for DerefMutToInner
impl ::std::ops::Deref for MyInt {
    type Target = i32;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(DerefMutToInner)]
struct Point1D {
    x: i32,
}

// Deref implementation is needed for DerefMutToInner
impl ::std::ops::Deref for Point1D {
    type Target = i32;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.x
    }
}

#[derive(DerefMutToInner)]
struct Point1D2 {
    x: i32,
    #[deref_mut_to_inner(ignore)]
    useless: bool,
}


// Deref implementation is needed for DerefMutToInner
impl ::std::ops::Deref for Point1D2 {
    type Target = i32;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.x
    }
}

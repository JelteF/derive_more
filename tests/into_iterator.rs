#![allow(dead_code, unused_imports)]
#[macro_use]
extern crate derive_more;

#[derive(IntoIterator, IntoIteratorRef, IntoIteratorRefMut)]
struct MyVec(Vec<i32>);

#[derive(IntoIterator, IntoIteratorRef, IntoIteratorRefMut)]
struct Numbers {
    numbers: Vec<i32>,
}

#[derive(IntoIterator, IntoIteratorRef, IntoIteratorRefMut)]
struct Numbers2 {
    numbers: Vec<i32>,
    #[into_iterator(ignore)]
    #[into_iterator_ref(ignore)]
    #[into_iterator_ref_mut(ignore)]
    useless: bool,
}

#[derive(IntoIterator, IntoIteratorRef, IntoIteratorRefMut)]
struct Numbers3 {
    #[into_iterator]
    #[into_iterator_ref]
    #[into_iterator_ref_mut]
    numbers: Vec<i32>,
    useless: bool,
    useless2: bool,
}

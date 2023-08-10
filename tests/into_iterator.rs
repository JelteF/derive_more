#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code, unused_imports)]

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use core::fmt::Debug;

use derive_more::IntoIterator;

fn test_into_iter<T: PartialEq + Debug, I: IntoIterator<Item = T>>(
    iter: I,
    vals: &[T],
) {
    assert_eq!(iter.into_iter().collect::<Vec<_>>(), vals);
}

fn test_into_iter_all<T, I>(mut iter: I, mut vals: Vec<T>)
where
    T: PartialEq + Debug,
    I: IntoIterator<Item = T>,
    for<'a> &'a I: IntoIterator<Item = &'a T>,
    for<'a> &'a mut I: IntoIterator<Item = &'a mut T>,
{
    test_into_iter(&mut iter, &vals.iter_mut().collect::<Vec<_>>());
    test_into_iter(&iter, &vals.iter().collect::<Vec<_>>());
    test_into_iter(iter, &vals);
}

#[derive(IntoIterator)]
#[into_iterator(owned, ref, ref_mut)]
struct MyVec(Vec<i32>);

#[test]
fn tuple_single() {
    let numbers = vec![1, 2, 3];

    test_into_iter_all(MyVec(numbers.clone()), numbers);
}

#[derive(IntoIterator)]
#[into_iterator(owned, ref, ref_mut)]
struct Numbers {
    numbers: Vec<i32>,
}

#[test]
fn named_single() {
    let numbers = vec![1, 2, 3];

    test_into_iter_all(
        Numbers {
            numbers: numbers.clone(),
        },
        numbers,
    );
}

#[derive(IntoIterator)]
struct Numbers2 {
    #[into_iterator(owned, ref, ref_mut)]
    numbers: Vec<i32>,
    useless: bool,
    useless2: bool,
}

fn named_many() {
    let numbers = vec![1, 2, 3];

    test_into_iter_all(
        Numbers2 {
            numbers: numbers.clone(),
            useless: true,
            useless2: true,
        },
        numbers,
    );
}

#[derive(IntoIterator)]
struct Numbers3 {
    #[into_iterator(ref, ref_mut)]
    numbers: Vec<i32>,
    useless: bool,
    useless2: bool,
}

// Test that owned is not enabled when ref/ref_mut are enabled without owned
impl ::core::iter::IntoIterator for Numbers3 {
    type Item = <Vec<i32> as ::core::iter::IntoIterator>::Item;
    type IntoIter = <Vec<i32> as ::core::iter::IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        <Vec<i32> as ::core::iter::IntoIterator>::into_iter(self.numbers)
    }
}

#[derive(IntoIterator)]
struct Generic1<T> {
    #[into_iterator(owned, ref, ref_mut)]
    items: Vec<T>,
}

#[test]
fn generic() {
    let numbers = vec![1, 2, 3];

    test_into_iter_all(
        Generic1 {
            items: numbers.clone(),
        },
        numbers,
    );
}

#[derive(IntoIterator)]
struct Generic2<'a, T, U: Send>
where
    T: Send,
{
    #[into_iterator(owned, ref, ref_mut)]
    items: Vec<T>,
    useless: &'a U,
}

#[test]
fn generic_bounds() {
    let numbers = vec![1, 2, 3];
    let useless = false;

    test_into_iter_all(
        Generic2 {
            items: numbers.clone(),
            useless: &useless,
        },
        numbers,
    );
}

#[derive(IntoIterator)]
struct Generic3<'a, 'b, T> {
    #[into_iterator(owned)]
    items: &'a mut Vec<&'b mut T>,
}

#[test]
fn generic_refs() {
    let mut numbers = vec![1, 2, 3];
    let mut numbers2 = numbers.clone();

    let mut number_refs = numbers.iter_mut().collect::<Vec<_>>();
    let mut number_refs2 = numbers2.iter_mut().collect::<Vec<_>>();

    test_into_iter(
        Generic3 {
            items: &mut number_refs,
        },
        &number_refs2.iter_mut().collect::<Vec<_>>(),
    )
}

#[derive(IntoIterator)]
struct Generic4<T> {
    #[into_iterator]
    items: Vec<T>,
    useless: bool,
}

#[test]
fn generic_owned() {
    let numbers = vec![1, 2, 3];

    test_into_iter(
        Generic4 {
            items: numbers.clone(),
            useless: true,
        },
        &numbers,
    );
}

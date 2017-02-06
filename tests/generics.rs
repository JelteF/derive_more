#[macro_use]
extern crate derive_more;

#[derive(From)]
struct Wrapped<T>(T);
// impl<T> ::std::convert::From<T> for Wrapped<T> {
//     fn from(original: T) -> Wrapped<T> {
//         Wrapped(original)
//     }
// }

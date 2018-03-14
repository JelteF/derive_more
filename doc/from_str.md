% What #[derive(FromStr)] generates

Deriving `FromStr` only works for newtypes, i.e structs with only a single
field. The result is that you will be able to call the `parse()` method on a
string to convert it to your newtype. This only works when the type that is
contained in the type implements `FromStr`.

# Example usage

```rust
# #[macro_use] extern crate derive_more;

#[derive(FromStr)]
struct MyInt(i32);

fn main() {
    let my_5 : MyInt = "5".parse().unwrap();
}
```

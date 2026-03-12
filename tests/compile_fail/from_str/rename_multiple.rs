use derive_more::FromStr;

#[derive(FromStr)]
enum E {
    #[from_str(rename = "a")]
    #[from_str(rename = "b")]
    A,
}

fn main() {}

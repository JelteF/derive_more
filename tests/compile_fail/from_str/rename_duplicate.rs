use derive_more::FromStr;

#[derive(FromStr)]
enum E {
    #[from_str(rename = "a")]
    A,
    #[from_str(rename = "a")]
    B,
}

fn main() {}

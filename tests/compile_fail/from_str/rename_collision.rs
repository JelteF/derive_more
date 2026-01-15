use derive_more::FromStr;

#[derive(FromStr)]
#[from_str(rename_all = "lowercase")]
enum E {
    #[from_str(rename = "b")]
    A,
    B,
}

fn main() {}
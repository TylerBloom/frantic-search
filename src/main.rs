use crate::cr::Cr;

pub mod cr;
pub mod parser;
pub mod search;

fn main() {
    // let text = include_str!("../MagicCompRules 20260227.txt");
    let text = include_str!("../test.txt");
    let cr = Cr::parse(text);
    println!("{cr}");

    let cr = cr.search(&["100.1a".into()]);
    println!();
    println!("{cr}");
}

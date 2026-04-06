use std::env::args;

use frantic_core::{cr::Cr, normalize_cr_text};

fn main() {
    let mut args = args();
    args.next();
    let words: Vec<_> = args.collect();
    println!("{}", Cr::latest().search(&words));
}

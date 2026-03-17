use std::env::args;

use rules_core::cr::Cr;

fn main() {
    let mut args = args();
    args.next();
    let words: Vec<_> = args.collect();
    println!("{}", Cr::latest().search(&words));
}

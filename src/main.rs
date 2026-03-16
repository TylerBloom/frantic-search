pub mod parser;

fn main() {
    // let text = include_str!("../MagicCompRules 20260227.txt");
    let text = include_str!("../test.txt");
    let cr = parser::parse_cr(text);
    println!("{cr:#?}");
}

#[derive(Debug)]
pub struct Section<'a> {
    pub text: &'a str,
    pub sections: Vec<SubSection<'a>>,
}

#[derive(Debug, Default)]
pub struct SubSection<'a> {
    pub text: &'a str,
    pub rules: Vec<Rule<'a>>,
}

#[derive(Debug)]
pub struct Rule<'a> {
    pub text: &'a str,
    // NOTE: There is not all rules have subrules, but there no functional difference between an
    // empty Vec and an optional Vec here.
    pub subrules: Vec<SubRule<'a>>,
}

#[derive(Debug)]
pub struct SubRule<'a> {
    pub text: &'a str,
}

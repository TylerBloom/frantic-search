use std::fmt::Display;

static LATEST_CR: &str = include_str!("../../docs/latest_cr.txt");

#[derive(Debug, Clone)]
pub struct Cr<'a>(pub Vec<Section<'a>>);

impl Cr<'_> {
    pub fn latest() -> Cr<'static> {
        Cr::parse(crate::normalize_cr_text(LATEST_CR).leak())
    }
}

impl Display for Cr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0
            .iter()
            .try_for_each(|section| write!(f, "{section}"))
    }
}

#[derive(Debug, Clone)]
pub struct Section<'a> {
    pub text: &'a str,
    pub subsections: Vec<SubSection<'a>>,
}

impl Display for Section<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)?;
        self.subsections
            .iter()
            .try_for_each(|sub| write!(f, "\n{sub}"))
    }
}

#[derive(Debug, Default, Clone)]
pub struct SubSection<'a> {
    pub text: &'a str,
    pub rules: Vec<Rule<'a>>,
}

impl Display for SubSection<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "  {}", self.text)?;
        self.rules.iter().try_for_each(|rule| write!(f, "\n{rule}"))
    }
}

#[derive(Debug, Clone)]
pub struct Rule<'a> {
    pub text: &'a str,
    // NOTE: There is not all rules have subrules, but there no functional difference between an
    // empty Vec and an optional Vec here.
    pub subrules: Vec<SubRule<'a>>,
}

impl Display for Rule<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "    {}", self.text)?;
        self.subrules
            .iter()
            .try_for_each(|sub| write!(f, "\n{sub}"))
    }
}

#[derive(Debug, Clone)]
pub struct SubRule<'a> {
    pub text: &'a str,
}
impl Display for SubRule<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "      {}", self.text)
    }
}

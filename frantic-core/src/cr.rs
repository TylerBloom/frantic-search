use std::fmt::Display;

#[derive(Debug, Default, Clone)]
pub struct Cr(pub Vec<Section>);

impl Display for Cr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.iter().try_for_each(|section| write!(f, "{section}"))
    }
}

#[derive(Debug, Clone)]
pub struct Section {
    pub text: String,
    pub subsections: Vec<SubSection>,
}

impl Display for Section {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)?;
        self.subsections
            .iter()
            .try_for_each(|sub| write!(f, "\n{sub}"))
    }
}

#[derive(Debug, Default, Clone)]
pub struct SubSection {
    pub number: String,
    pub text: String,
    pub rules: Vec<Rule>,
}

impl Display for SubSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "  {}", self.text)?;
        self.rules.iter().try_for_each(|rule| write!(f, "\n{rule}"))
    }
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub number: String,
    pub text: String,
    // NOTE: There is not all rules have subrules, but there no functional difference between an
    // empty Vec and an optional Vec here.
    pub subrules: Vec<SubRule>,
}

impl Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "    {}", self.text)?;
        self.subrules
            .iter()
            .try_for_each(|sub| write!(f, "\n{sub}"))
    }
}

#[derive(Debug, Clone, Default)]
pub struct SubRule {
    pub number: String,
    pub text: String,
}
impl Display for SubRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "      {}", self.text)
    }
}

use crate::cr::*;

pub struct CrSearch();

impl<'a> Cr<'a> {
    pub fn search(&self, words: &'_ [String]) -> Cr<'a> {
        let mut digest = self.clone();
        digest.0.retain_mut(|section| {
            section.retain(words);
            !section.is_empty()
        });
        digest
    }
}

impl Section<'_> {
    fn is_empty(&self) -> bool {
        self.subsections.is_empty()
    }

    fn retain(&mut self, words: &'_ [String]) {
        self.subsections.retain_mut(|sub| {
            sub.retain(words);
            !sub.is_empty()
        });
    }
}

impl SubSection<'_> {
    fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    fn retain(&mut self, words: &'_ [String]) {
        self.rules.retain_mut(|rule| {
            rule.retain(words);
            !rule.is_empty()
        });
    }
}

impl Rule<'_> {
    fn is_empty(&self) -> bool {
        (self.text.len() < 10) && self.subrules.is_empty()
    }

    fn retain(&mut self, words: &'_ [String]) {
        if !contains_words(self.text, words) {
            let text = self.text.split_once(' ').unwrap_or(("", "")).0;
            self.text = text;
        }
        self.subrules.retain(|sub| contains_words(sub.text, words));
    }
}

fn contains_words(input: &str, words: &[String]) -> bool {
    match words {
        [] => true,
        [first, rest @ ..] => match input.split_once(first) {
            None => false,
            Some((front, back)) => contains_words(front, rest) || contains_words(back, rest),
        },
    }
}

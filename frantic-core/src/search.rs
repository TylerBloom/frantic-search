use crate::cr::*;

impl Cr {
    pub fn search(&self, words: &'_ [String]) -> Self {
        let words: Vec<_> = words.iter().map(|s| s.to_lowercase()).collect();
        let mut digest = self.clone();
        digest.0.retain_mut(|section| {
            section.retain(&words);
            !section.is_empty()
        });
        digest
    }
}

impl Section {
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

impl SubSection {
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

impl Rule {
    fn is_empty(&self) -> bool {
        (self.text.len() < 10) && self.subrules.is_empty()
    }

    fn retain(&mut self, words: &'_ [String]) {
        if !contains_words(&self.text.to_lowercase(), words) {
            let text = self.text.split_once(' ').unwrap_or(("", "")).0;
            self.text = text.to_owned();
        }
        self.subrules
            .retain(|sub| contains_words(&sub.text.to_lowercase(), words));
    }
}

fn contains_words(input: &str, words: &[String]) -> bool {
    match words {
        [] => true,
        [first, rest @ ..] => input
            .split_once(first)
            .is_some_and(|(front, back)| contains_words(front, rest) || contains_words(back, rest)),
    }
}

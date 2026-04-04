use crate::cr::*;

impl<'a> Cr<'a> {
    pub fn parse(text: &'a str) -> Cr<'a> {
        let mut text = build_rules_iter(text);
        let mut digest = Vec::new();
        while let Some(line) = text.next() {
            digest.push(Section::construct(line, &mut text));
        }
        Self(digest)
    }
}

/// The CR's text starts and ends with a bunch of text that is not super helpful as, right now, the
/// focus is just on the rules text. This function returns an iterator over the lines of the rules.
/// Each line should start with a rule number (including section numbers).
///
/// Examples are currently omitted as well.
fn build_rules_iter(text: &str) -> impl Iterator<Item = &str> {
    let mut lines = text.lines().map(str::trim).filter(|line| !line.is_empty());
    // Move the cursor into the table of contents
    lines.find(|line| line.trim() == "Contents");
    // The Glossary and Credits are the end of the table of contents. What follows should be
    // sections 1.
    lines.find(|line| line.trim() == "Glossary");
    lines.find(|line| line.trim() == "Credits");
    lines
        .take_while(|line| *line != "Glossary")
        .filter(|line| !line.starts_with("Example:"))
}

impl<'a> Section<'a> {
    fn construct(line: &'a str, text: impl Iterator<Item = &'a str>) -> Self {
        let mut text = text.peekable();
        let mut subsections = Vec::new();
        loop {
            match text.peek().map(SubSection::parse) {
                Some(mut sub) => {
                    text.next();
                    sub.populate(&mut text);
                    subsections.push(sub);
                }
                None => {
                    return Self {
                        text: line,
                        subsections,
                    };
                }
            }
        }
    }
}

impl<'a> SubSection<'a> {
    fn parse(text: &&'a str) -> Self {
        Self {
            text,
            rules: Vec::new(),
        }
    }

    fn populate(&mut self, text: impl Iterator<Item = &'a str>) {
        let mut text = text.peekable();
        while let Some(mut rule) = text.peek().and_then(Rule::parse) {
            text.next();
            rule.populate(&mut text);
            self.rules.push(rule);
        }
    }
}

impl<'a> Rule<'a> {
    fn parse(text: &&'a str) -> Option<Self> {
        let (header, _) = text.split_once(' ')?;
        let mut chunks = header.split('.');
        let chunk_one = chunks
            .next()
            .is_some_and(|chunk| chunk.parse::<usize>().is_ok());
        let chunk_two = chunks
            .next()
            .is_some_and(|chunk| chunk.parse::<usize>().is_ok());
        let chunk_three = chunks.next().is_some_and(|chunk| chunk.trim().is_empty());
        (chunk_one && chunk_two && chunk_three).then(|| Self {
            text,
            subrules: Vec::new(),
        })
    }

    fn populate(&mut self, text: impl Iterator<Item = &'a str>) {
        let mut text = text.peekable();
        while let Some(sub) = text.peek().and_then(SubRule::parse) {
            self.subrules.push(sub);
            text.next();
        }
    }
}

impl<'a> SubRule<'a> {
    fn parse(text: &&'a str) -> Option<Self> {
        let (header, _) = text.split_once(' ')?;
        header
            .chars()
            .last()
            .is_some_and(char::is_alphabetic)
            .then_some(Self { text })
    }
}

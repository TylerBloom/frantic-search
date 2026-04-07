use std::iter::Peekable;

use crate::cr::*;

// Start -> Section
// Section -> Subsection
// Subsection -> Rule
// Rule -> Subrule | Rule | Section | Subsection
// Subrule -> Subrule | Rule | Section | Subsection

impl<'a> Cr<'a> {
    pub fn parse(text: &'a str) -> Cr<'a> {
        let mut text = build_rules_iter(text).peekable();
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
    fn construct<I: Iterator<Item = &'a str>>(line: &'a str, text: &mut Peekable<I>) -> Self {
        let mut subsections = Vec::new();
        while let Some(mut sub) = text.peek().and_then(SubSection::parse) {
            text.next();
            sub.populate(text);
            subsections.push(sub);
        }
        Self {
            text: line,
            subsections,
        }
    }
}

impl<'a> SubSection<'a> {
    fn parse(text: &&'a str) -> Option<Self> {
        let (pre, _) = text.split_once('.')?;
        (pre.len() >= 3).then(|| Self {
            text,
            rules: Vec::new(),
        })
    }

    fn populate<I: Iterator<Item = &'a str>>(&mut self, text: &mut Peekable<I>) {
        while let Some(mut rule) = text.peek().and_then(Rule::parse) {
            text.next();
            rule.populate(text);
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

    fn populate<I: Iterator<Item = &'a str>>(&mut self, text: &mut Peekable<I>) {
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

#[cfg(test)]
mod tests {
    use crate::cr::Cr;

    static CR_EXERPT: &str = r#"Contents
Glossary
Credits

1. Game Concepts

100. General

100.1. These Magic rules apply to any Magic game with two or more players, including two-player games and multiplayer games.

100.1a A two-player game is a game that begins with only two players.

100.1b A multiplayer game is a game that begins with more than two players. See section 8, “Multiplayer Rules.”

100.2. To play, each player needs their own deck of traditional Magic cards, small items to represent any tokens and counters, and some way to clearly track life totals.

100.2a In constructed play (a way of playing in which each player creates their own deck ahead of time), each deck has a minimum deck size of 60 cards. A constructed deck may contain any number of basic land cards and no more than four of any card with a particular English name other than basic land cards. For the purposes of deck construction, cards with interchangeable names have the same English name (see rule 201.3).

100.2b In limited play (a way of playing in which each player gets the same quantity of unopened Magic product such as booster packs and creates their own deck using only this product and basic land cards), each deck has a minimum deck size of 40 cards. A limited deck may contain as many duplicates of a card as are included with the product.

100.2c Commander decks are subject to additional deckbuilding restrictions and requirements. See rule 903, “Commander,” for details.

100.2d Some formats and casual play variants allow players to use a supplementary deck of nontraditional Magic cards (see rule 108.2a). These supplementary decks have their own deck construction rules. See rule 717, “Attraction Cards;” rule 901, “Planechase;” and rule 904, “Archenemy.”

100.3. Some cards require coins or traditional dice. Some casual variants require additional items, such as specially designated cards, nontraditional Magic cards, and specialized dice.

100.4. Each player may also have a sideboard, which is a group of additional cards the player may use to modify their deck between games of a match. Sideboard rules and restrictions for some formats are modified by the Magic: The Gathering Tournament Rules (found at WPN.Wizards.com/en/rules-documents).

2. Next Section"#;

    #[test]
    fn basic_parse() {
        let cr = Cr::parse(CR_EXERPT);
        println!("{cr}\n");
        // Check sections
        assert_eq!(cr.0[0].text, "1. Game Concepts");
        assert_eq!(cr.0[1].text, "2. Next Section");
        assert_eq!(cr.0.len(), 2);
        let section = &cr.0[0];
        panic!();

        // Check subsections
        assert_eq!(section.subsections.len(), 4);
        for (i, sub) in section.subsections.iter().enumerate() {
            println!("{}", sub.text);
            assert!(sub.text.starts_with(&format!("100.{i}")));
        }

        // Check rules
        assert_eq!(section.subsections.len(), 4);
        for (i, sub) in section.subsections.iter().enumerate() {
            println!("{}", sub.text);
            assert!(sub.text.starts_with(&format!("100.{i}")));
        }

        // Check subrules
        assert_eq!(section.subsections.len(), 4);
        for (i, sub) in section.subsections.iter().enumerate() {
            println!("{}", sub.text);
            assert!(sub.text.starts_with(&format!("100.{i}")));
        }
    }
}

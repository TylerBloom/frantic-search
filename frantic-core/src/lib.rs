pub mod cr;
pub mod parser;
pub mod search;

/// Sometimes that published CR contains Windows escape sequences instead of \n characters. This
/// corrects that.
pub fn normalize_cr_text(text: &str) -> String {
    text.replace("\r", "\n")
}

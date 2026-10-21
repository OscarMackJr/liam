//! Console display helpers. Upstream text (hometown's answer, a suppressed
//! suggestion, olive's identifiers and status) is display only and must not
//! control the banker's terminal (design §9). Evidence files are not touched
//! here; they are content-free anyway.

/// Replaces every control character other than `\n` and `\t` with U+FFFD:
/// C0 controls (including ESC and `\r`), DEL and C1 (U+0080..U+009F). All
/// other text is unchanged.
pub fn printable(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_control() && c != '\n' && c != '\t' {
                '\u{FFFD}'
            } else {
                c
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::printable;

    #[test]
    fn control_characters_become_replacement_characters() {
        assert_eq!(printable("a\u{1b}[2Jb\rc"), "a\u{FFFD}[2Jb\u{FFFD}c");
        assert_eq!(
            printable("\u{0}\u{7}\u{8}\u{b}\u{c}\u{1f}\u{7f}\u{80}\u{9b}\u{9f}"),
            "\u{FFFD}".repeat(10)
        );
        assert_eq!(
            printable("\u{1b}]8;;http://x\u{7}link\u{1b}]8;;\u{7}"),
            "\u{FFFD}]8;;http://x\u{FFFD}link\u{FFFD}]8;;\u{FFFD}"
        );
    }

    #[test]
    fn normal_text_is_unchanged() {
        let text = "Line one\n\tindented caf\u{e9} \u{1F600} \u{a0}\u{a0}end";
        assert_eq!(printable(text), text);
        assert_eq!(printable(""), "");
    }
}

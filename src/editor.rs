/**
 * Minimal file editor
 * "wq" -> save and quit
 * "q" -> quit
 */
use std::io::{self, BufRead, Write};

/// Returns the typed content on "wq", `None` on "q" or end of input.
pub fn edit(input: &mut impl BufRead, out: &mut impl Write) -> io::Result<Option<String>> {
    writeln!(
        out,
        "Welcome to the editor! Type 'wq' to save and quit or 'q' to quit without saving."
    )?;

    let mut content = String::new();
    loop {
        write!(out, "> ")?;
        out.flush()?;

        let mut line = String::new();
        if input.read_line(&mut line)? == 0 {
            writeln!(out, "Exit without save")?;
            return Ok(None);
        }

        match line.trim() {
            "wq" => return Ok(Some(content)),
            "q" => {
                writeln!(out, "Exit without save")?;
                return Ok(None);
            }
            _ => content.push_str(&line),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(input: &str) -> Option<String> {
        edit(&mut input.as_bytes(), &mut Vec::new()).unwrap()
    }

    #[test]
    fn wq_saves_typed_lines() {
        assert_eq!(run("a\n\nb\nwq\n").as_deref(), Some("a\n\nb\n"));
    }

    #[test]
    fn q_discards() {
        assert_eq!(run("a\nq\n"), None);
    }

    #[test]
    fn end_of_input_discards_instead_of_looping() {
        assert_eq!(run("a\n"), None);
    }
}

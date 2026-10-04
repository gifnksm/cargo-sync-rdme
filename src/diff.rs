use std::{
    fmt,
    io::{self, Write as _},
};

use similar::{ChangeTag, TextDiff};

use crate::terminal::StreamInfo;

#[derive(Debug)]
struct Line(Option<usize>);

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            None => write!(f, "    "),
            Some(idx) => write!(f, "{:>4}", idx + 1),
        }
    }
}

pub(crate) fn write_pretty_diff(stream: StreamInfo, old: &str, new: &str) -> Result<(), io::Error> {
    let styler = stream.styler();
    let diff = TextDiff::from_lines(old, new);

    let mut output = stream.lock();
    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            writeln!(&mut output, "{0:─^1$}┼{0:─^2$}", "─", 9, 120)?;
        }
        for op in group {
            for change in diff.iter_inline_changes(op) {
                let (sign, style) = match change.tag() {
                    ChangeTag::Delete => ("-", styler.style().red()),
                    ChangeTag::Insert => ("+", styler.style().green()),
                    ChangeTag::Equal => (" ", styler.style().dim()),
                };
                write!(
                    &mut output,
                    "{}{} │{}",
                    styler.styled(Line(change.old_index())).dim(),
                    styler.styled(Line(change.new_index())).dim(),
                    style.apply_to(sign).bold(),
                )?;
                for (emphasized, value) in change.iter_strings_lossy() {
                    if emphasized {
                        write!(
                            &mut output,
                            "{}",
                            style.apply_to(value).underlined().on_black()
                        )?;
                    } else {
                        write!(&mut output, "{}", style.apply_to(value))?;
                    }
                }
                if change.missing_newline() {
                    writeln!(&mut output)?;
                }
            }
        }
    }
    Ok(())
}

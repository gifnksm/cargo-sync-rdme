use std::{fmt, io};

use similar::{ChangeTag, TextDiff};
use supports_color::Stream;

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

#[derive(Debug)]
struct DiffStyler {
    stream: Stream,
}

impl DiffStyler {
    fn new(stream: Stream) -> Self {
        Self { stream }
    }

    fn style(&self) -> console::Style {
        let s = console::Style::new();
        match self.stream {
            Stream::Stdout => s.for_stdout(),
            Stream::Stderr => s.for_stderr(),
        }
    }

    fn styled<D>(&self, val: D) -> console::StyledObject<D> {
        let s = console::style(val);
        match self.stream {
            Stream::Stdout => s.for_stdout(),
            Stream::Stderr => s.for_stderr(),
        }
    }
}

pub(crate) fn write_pretty_diff(stream: Stream, old: &str, new: &str) -> Result<(), io::Error> {
    let styling = DiffStyler::new(stream);
    let diff = TextDiff::from_lines(old, new);

    let mut output: &mut dyn io::Write = match stream {
        Stream::Stdout => &mut io::stdout().lock(),
        Stream::Stderr => &mut io::stderr().lock(),
    };

    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            writeln!(&mut output, "{0:─^1$}┼{0:─^2$}", "─", 9, 120)?;
        }
        for op in group {
            for change in diff.iter_inline_changes(op) {
                let (sign, style) = match change.tag() {
                    ChangeTag::Delete => ("-", styling.style().red()),
                    ChangeTag::Insert => ("+", styling.style().green()),
                    ChangeTag::Equal => (" ", styling.style().dim()),
                };
                write!(
                    &mut output,
                    "{}{} │{}",
                    styling.styled(Line(change.old_index())).dim(),
                    styling.styled(Line(change.new_index())).dim(),
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

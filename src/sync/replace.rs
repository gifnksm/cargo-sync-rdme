use std::{borrow::Cow, iter, range::Range};

use crate::sync::{contents::Contents, marker};

pub(in super::super) fn replace_all(text: &str, contents: &[Contents<'_>]) -> String {
    let pairs = contents
        .iter()
        .map(|contents| (contents, contents.specifier().span));

    interpolate_ranges((0..text.len()).into(), pairs)
        .map(|(contents, range)| match contents {
            Some(contents) => marker::make_marked_contents(contents).into(),
            None => Cow::Borrowed(&text[range]),
        })
        .collect()
}

fn interpolate_ranges<T>(
    range: Range<usize>,
    items: impl IntoIterator<Item = (T, Range<usize>)>,
) -> impl Iterator<Item = (Option<T>, Range<usize>)> {
    let mut items = items.into_iter().peekable();
    let mut offset = range.start;
    iter::from_fn(move || match items.peek() {
        Some(&(_, Range { start, .. })) if offset < start => {
            let range = (offset..start).into();
            offset = start;
            Some((None, range))
        }
        Some(_) => {
            let (item, Range { start, end }) = items.next().unwrap();
            offset = end;
            Some((Some(item), (start..end).into()))
        }
        None if offset < range.end => {
            let range = Range::from(offset..range.end);
            offset = range.end;
            Some((None, range))
        }
        None => None,
    })
}

#[cfg(test)]
mod tests {
    use similar_asserts::assert_eq;

    #[test]
    fn interpolate_ranges() {
        let items = [(1, (0..1).into()), (2, (1..2).into()), (3, (2..3).into())];
        let ranges = super::interpolate_ranges((0..3).into(), items);
        assert_eq!(
            ranges.collect::<Vec<_>>(),
            vec![
                (Some(1), (0..1).into()),
                (Some(2), (1..2).into()),
                (Some(3), (2..3).into()),
            ]
        );

        let items = [
            (1, (3..4).into()),
            (2, (4..5).into()),
            (3, (6..7).into()),
            (4, (8..9).into()),
        ];
        let ranges = super::interpolate_ranges((0..10).into(), items);
        assert_eq!(
            ranges.collect::<Vec<_>>(),
            vec![
                (None, (0..3).into()),
                (Some(1), (3..4).into()),
                (Some(2), (4..5).into()),
                (None, (5..6).into()),
                (Some(3), (6..7).into()),
                (None, (7..8).into()),
                (Some(4), (8..9).into()),
                (None, (9..10).into()),
            ]
        );
    }
}

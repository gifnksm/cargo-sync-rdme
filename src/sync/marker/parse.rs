use std::fmt::{self, Display};

use miette::{Diagnostic, SourceSpan};
use pulldown_cmark::{Event, OffsetIter, Options};
use snafu::{OptionExt as _, Snafu, ensure};

use crate::{
    parse::{self, Spanned},
    sync::marker::MAGIC,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Marker<'a> {
    Replace(SpannedReplaceSpecifier<'a>),
    Start(SpannedReplaceSpecifier<'a>),
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ReplaceSpecifier<'a> {
    pub(super) kind: Spanned<&'a str>,
    pub(super) group: Option<Spanned<&'a str>>,
}

#[derive(Debug, Snafu, Diagnostic)]
pub(crate) enum ParseMarkerError {
    #[snafu(display("unexpected token: `{token}`, expected: {expected}"))]
    UnexpectedToken {
        token: String,
        expected: String,
        #[label]
        span: SourceSpan,
    },
    #[snafu(display("unexpected end of marker, expected: {expected}"))]
    UnexpectedEndOfMarker {
        expected: String,
        #[label]
        span: SourceSpan,
    },
}

#[derive(Debug)]
pub(super) struct MarkerParser<'a> {
    markdown: &'a str,
    parser: OffsetIter<'a>,
}

impl<'a> MarkerParser<'a> {
    pub(super) fn new(markdown: &'a str) -> Self {
        let parser = pulldown_cmark::Parser::new_ext(markdown, Options::all()).into_offset_iter();
        Self { markdown, parser }
    }

    pub(super) fn try_next(&mut self) -> Result<Option<SpannedMarker<'a>>, ParseMarkerError> {
        for (event, range) in self.parser.by_ref() {
            if let Event::Html(_html) = event {
                // Use the original markdown slice for this HTML event instead of the
                // `Event::Html` payload. The payload may be normalized by pulldown-cmark,
                // which would make the parsed marker text diverge from the original source span
                // and break diagnostics.
                //
                // As of 2026-08-21, pulldown-cmark main contains an unreleased change that
                // replaces `\0` with `\u{FFFD}` in `Event::Html`:
                // <https://github.com/pulldown-cmark/pulldown-cmark/blob/07bae2459d90175b661d42b8acf207382e111ae5/pulldown-cmark/src/parse.rs#L2404-L2406>
                let html = &self.markdown[range.clone()];
                let html = Spanned::new(html, range);
                if let Some(marker) = parse_marker(html)? {
                    return Ok(Some(marker));
                }
            }
        }
        Ok(None)
    }
}

type Input<'a> = Spanned<&'a str>;
type SpannedMarker<'a> = Spanned<Marker<'a>>;
type SpannedReplaceSpecifier<'a> = Spanned<ReplaceSpecifier<'a>>;
type SpannedToken<'a> = Spanned<Token<'a>>;

// Marker syntax:
//
// marker ::= replace-marker | start-marker | end-marker
// replace-marker ::= "<!-- cargo-sync-rdme " specifier " -->"
// start-marker ::= "<!-- cargo-sync-rdme " specifier " [[ -->"
// end-marker ::= "<!-- cargo-sync-rdme ]] -->"
// specifier ::= marker-kind [ ":" group-name ]
// marker-kind ::= ident
// group-name ::= ident
// ident ::= [A-Za-z][-_A-Za-z0-9]*

pub(super) fn parse_marker(html: Input<'_>) -> Result<Option<SpannedMarker<'_>>, ParseMarkerError> {
    let html = html.trim();
    let html_span = html.span;

    let Some(comment_body) = trim_comment(html) else {
        return Ok(None);
    };
    let Some(marker_body) = trim_magic(comment_body) else {
        return Ok(None);
    };

    let Some((specifier, rest)) = parse_specifier(marker_body)? else {
        let (_, rest) = expect_token(marker_body, Token::EndMarkerSymbol, "marker kind or `]]`")?;
        expect_end_of_marker(rest)?;
        return Ok(Some(Spanned::new(Marker::End, html_span)));
    };

    if next_token(rest).is_none() {
        return Ok(Some(Spanned::new(Marker::Replace(specifier), html_span)));
    }
    let (_token, rest) = expect_token(rest, Token::StartMarkerSymbol, "end of marker or `[[`")?;
    expect_end_of_marker(rest)?;
    Ok(Some(Spanned::new(Marker::Start(specifier), html_span)))
}

pub(super) fn parse_specifier(
    input: Input<'_>,
) -> Result<Option<(SpannedReplaceSpecifier<'_>, Input<'_>)>, ParseMarkerError> {
    let input = input.trim_start();
    let Ok((kind, rest)) = expect_ident(input, "marker kind") else {
        return Ok(None);
    };

    let Ok((_colon, rest)) = expect_token(rest, Token::Colon, "`:`") else {
        return Ok(Some((
            Spanned::new(
                ReplaceSpecifier { kind, group: None },
                input.prefix_of(rest).span,
            ),
            rest,
        )));
    };

    let (group, rest) = expect_ident(rest, "group name")?;
    Ok(Some((
        Spanned::new(
            ReplaceSpecifier {
                kind,
                group: Some(group),
            },
            input.prefix_of(rest).span,
        ),
        rest,
    )))
}

fn trim_magic(comment_body: Input<'_>) -> Option<Input<'_>> {
    let (head, tail) = comment_body
        .split_once_fn(char::is_whitespace)
        .unwrap_or((comment_body, comment_body.end()));
    if head != MAGIC {
        return None;
    }
    Some(tail.trim())
}

fn trim_comment(html: Input<'_>) -> Option<Input<'_>> {
    let comment_body = html
        .trim()
        .strip_prefix_str("<!--")?
        .trim_start()
        .strip_suffix_str("-->")?
        .trim_end();
    Some(comment_body)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Token<'a> {
    Ident(&'a str),
    Colon,
    UnknownChar(&'a str),
    StartMarkerSymbol,
    EndMarkerSymbol,
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Ident(s) | Token::UnknownChar(s) => s.fmt(f),
            Token::Colon => ":".fmt(f),
            Token::StartMarkerSymbol => "[[".fmt(f),
            Token::EndMarkerSymbol => "]]".fmt(f),
        }
    }
}

fn eat_str<'a>(input: Input<'a>, target: &str) -> Option<(Input<'a>, Input<'a>)> {
    let rest = input.strip_prefix_str(target)?;
    let token = input.prefix_of(rest);
    Some((token, rest))
}

fn eat_ident(input: Input<'_>) -> Option<(Input<'_>, Input<'_>)> {
    let mut it = input.value.chars();
    if !it.as_str().starts_with(parse::is_ident_start) {
        return None;
    }
    it.next();
    while it.as_str().starts_with(parse::is_ident_continue) {
        it.next();
    }
    let rest = input.substr(it.as_str());
    let ident = input.prefix_of(rest);
    Some((ident, rest))
}

fn eat_char(input: Input<'_>) -> Option<(Input<'_>, Input<'_>)> {
    let mut it = input.value.chars();
    let _ = it.next()?;
    let rest = input.substr(it.as_str());
    let token = input.prefix_of(rest);
    Some((token, rest))
}

fn next_token(input: Input<'_>) -> Option<(SpannedToken<'_>, Input<'_>)> {
    let input = input.trim_start();
    if let Some((token, rest)) = eat_str(input, "[[") {
        let token = Spanned::new(Token::StartMarkerSymbol, token.span);
        return Some((token, rest));
    }
    if let Some((token, rest)) = eat_str(input, "]]") {
        let token = Spanned::new(Token::EndMarkerSymbol, token.span);
        return Some((token, rest));
    }
    if let Some((token, rest)) = eat_str(input, ":") {
        let token = Spanned::new(Token::Colon, token.span);
        return Some((token, rest));
    }
    if let Some((token, rest)) = eat_ident(input) {
        let token = Spanned::new(Token::Ident(token.value), token.span);
        return Some((token, rest));
    }
    if let Some((token, rest)) = eat_char(input) {
        let token = Spanned::new(Token::UnknownChar(token.value), token.span);
        return Some((token, rest));
    }
    None
}

fn expect_token<'a>(
    input: Input<'a>,
    expected_token: Token<'_>,
    expected: &str,
) -> Result<(SpannedToken<'a>, Input<'a>), ParseMarkerError> {
    let (token, rest) = next_token(input).with_context(|| UnexpectedEndOfMarkerSnafu {
        expected,
        span: input.end().source_span(),
    })?;
    ensure!(
        token.value == expected_token,
        UnexpectedTokenSnafu {
            token: token.value.to_string(),
            expected,
            span: token.source_span(),
        }
    );
    Ok((token, rest))
}

fn expect_ident<'a>(
    input: Input<'a>,
    expected: &str,
) -> Result<(Input<'a>, Input<'a>), ParseMarkerError> {
    let (token, rest) = next_token(input).with_context(|| UnexpectedEndOfMarkerSnafu {
        expected,
        span: input.end().source_span(),
    })?;
    let Token::Ident(ident) = token.value else {
        return Err(UnexpectedTokenSnafu {
            token: token.value.to_string(),
            expected,
            span: token.source_span(),
        }
        .build());
    };
    Ok((Spanned::new(ident, token.span), rest))
}

fn expect_end_of_marker(input: Input<'_>) -> Result<(), ParseMarkerError> {
    let Some((token, _rest)) = next_token(input) else {
        return Ok(());
    };
    Err(UnexpectedTokenSnafu {
        token: token.value.to_string(),
        expected: "end of marker",
        span: token.source_span(),
    }
    .build())
}

#[cfg(test)]
mod tests {
    use super::*;

    use indoc::indoc;
    use similar_asserts::assert_eq;

    impl<'a> Marker<'a> {
        #[track_caller]
        pub(crate) fn into_replace(self) -> SpannedReplaceSpecifier<'a> {
            let Self::Replace(specifier) = self else {
                panic!("unexpected marker: {self:?}");
            };
            specifier
        }

        #[track_caller]
        pub(crate) fn into_start(self) -> SpannedReplaceSpecifier<'a> {
            let Self::Start(specifier) = self else {
                panic!("unexpected marker: {self:?}");
            };
            specifier
        }

        #[track_caller]
        pub(crate) fn into_end(self) {
            let Self::End = self else {
                panic!("unexpected marker: {self:?}");
            };
        }
    }

    impl ParseMarkerError {
        #[track_caller]
        pub(crate) fn into_unexpected_token(self) -> (String, String, SourceSpan) {
            let Self::UnexpectedToken {
                token,
                expected,
                span,
            } = self
            else {
                panic!("unexpected error: {self:?}");
            };
            (token, expected, span)
        }

        #[track_caller]
        pub(crate) fn into_unexpected_eom(self) -> (String, SourceSpan) {
            let Self::UnexpectedEndOfMarker { expected, span } = self else {
                panic!("unexpected error: {self:?}");
            };
            (expected, span)
        }
    }

    #[test]
    fn parse_marker_parses_markers() {
        let source = Spanned::from_str("<!-- cargo-sync-rdme kind:group -->");
        let marker = parse_marker(source).unwrap().unwrap();
        source.assert_spanned(marker, "<!-- cargo-sync-rdme kind:group -->");
        let specifier = marker.value.into_replace();
        source.assert_spanned(specifier, "kind:group");
        source.assert_spanned_str(specifier.value.kind, "kind");
        source.assert_spanned_str(specifier.value.group.unwrap(), "group");

        let source = Spanned::from_str("<!-- cargo-sync-rdme kind:group [[ -->");
        let marker = parse_marker(source).unwrap().unwrap();
        source.assert_spanned(marker, "<!-- cargo-sync-rdme kind:group [[ -->");
        let specifier = marker.value.into_start();
        source.assert_spanned(specifier, "kind:group");
        source.assert_spanned_str(specifier.value.kind, "kind");
        source.assert_spanned_str(specifier.value.group.unwrap(), "group");

        let source = Spanned::from_str("<!-- cargo-sync-rdme ]] -->");
        let marker = parse_marker(source).unwrap().unwrap();
        source.assert_spanned(marker, "<!-- cargo-sync-rdme ]] -->");
        marker.value.into_end();
    }

    #[test]
    fn parse_marker_rejects_invalid_markers() {
        let source = Spanned::from_str("<!-- cargo-sync-rdme -->");
        let err = parse_marker(source).unwrap_err();
        let ParseMarkerError::UnexpectedEndOfMarker { expected, span } = err else {
            panic!("unexpected error: {err:?}");
        };
        assert_eq!(expected, "marker kind or `]]`");
        source.assert_source_span(span, "");
        assert_eq!(span.offset(), source.value.len() - " -->".len());

        let source = Spanned::from_str("<!-- cargo-sync-rdme [[ -->");
        let (token, expected, span) = parse_marker(source).unwrap_err().into_unexpected_token();
        assert_eq!(token, "[[");
        assert_eq!(expected, "marker kind or `]]`");
        source.assert_source_span(span, "[[");

        let source = Spanned::from_str("<!-- cargo-sync-rdme badge:123 -->");
        let (token, expected, span) = parse_marker(source).unwrap_err().into_unexpected_token();
        assert_eq!(token, "1");
        assert_eq!(expected, "group name");
        source.assert_source_span(span, "1");

        let source = Spanned::from_str("<!-- cargo-sync-rdme ]] xxx -->");
        let (token, expected, span) = parse_marker(source).unwrap_err().into_unexpected_token();
        assert_eq!(token, "xxx");
        assert_eq!(expected, "end of marker");
        source.assert_source_span(span, "xxx");

        let source = Spanned::from_str("<!-- cargo-sync-rdme badge:bar xxx -->");
        let (token, expected, span) = parse_marker(source).unwrap_err().into_unexpected_token();
        assert_eq!(token, "xxx");
        assert_eq!(expected, "end of marker or `[[`");
        source.assert_source_span(span, "xxx");

        let source = Spanned::from_str("<!-- cargo-sync-rdme badge:bar [[ xxx -->");
        let (token, expected, span) = parse_marker(source).unwrap_err().into_unexpected_token();
        assert_eq!(token, "xxx");
        assert_eq!(expected, "end of marker");
        source.assert_source_span(span, "xxx");
    }

    #[test]
    fn parse_marker_ignores_non_markers() {
        let source = Spanned::from_str("<p>paragraph</p>");
        assert!(parse_marker(source).unwrap().is_none());

        let source = Spanned::from_str("<!-- test -->");
        assert!(parse_marker(source).unwrap().is_none());
    }

    #[test]
    fn parse_specifier_parses_kind_and_group() {
        let source = Spanned::from_str("kind:group");
        let (specifier, rest) = parse_specifier(source).unwrap().unwrap();
        source.assert_spanned(specifier, "kind:group");
        source.assert_spanned_str(specifier.value.kind, "kind");
        source.assert_spanned_str(specifier.value.group.unwrap(), "group");
        source.assert_spanned_str(rest, "");

        let source = Spanned::from_str(" kind:group ");
        let (specifier, rest) = parse_specifier(source).unwrap().unwrap();
        source.assert_spanned(specifier, "kind:group");
        source.assert_spanned_str(specifier.value.kind, "kind");
        source.assert_spanned_str(specifier.value.group.unwrap(), "group");
        source.assert_spanned_str(rest, " ");

        let source = Spanned::from_str(" kind : group xxx");
        let (specifier, rest) = parse_specifier(source).unwrap().unwrap();
        source.assert_spanned(specifier, "kind : group");
        source.assert_spanned_str(specifier.value.kind, "kind");
        source.assert_spanned_str(specifier.value.group.unwrap(), "group");
        source.assert_spanned_str(rest, " xxx");

        let source = Spanned::from_str(" kind ");
        let (specifier, rest) = parse_specifier(source).unwrap().unwrap();
        source.assert_spanned(specifier, "kind");
        source.assert_spanned_str(specifier.value.kind, "kind");
        assert!(specifier.value.group.is_none());
        source.assert_spanned_str(rest, " ");

        let source = Spanned::from_str("  ");
        assert!(parse_specifier(source).unwrap().is_none());
    }

    #[test]
    fn parse_specifier_rejects_invalid_specifiers() {
        let source = Spanned::from_str(" kind: ");
        let (expected, span) = parse_specifier(source).unwrap_err().into_unexpected_eom();
        assert_eq!(expected, "group name");
        source.assert_source_span(span, "");
        assert_eq!(span.offset(), source.value.len());

        let source = Spanned::from_str(" :group");
        assert!(parse_specifier(source).unwrap().is_none());
    }

    #[test]
    fn trim_magic_trims_magic() {
        let source = Spanned::from_str("cargo-sync-rdme   test  foo bar");
        let trimmed = trim_magic(source).unwrap();
        source.assert_spanned_str(trimmed, "test  foo bar");

        let source = Spanned::from_str("cargo-sync-rdme");
        let trimmed = trim_magic(source).unwrap();
        source.assert_spanned_str(trimmed, "");

        let source = Spanned::from_str("cargo-sync-rdmexxx");
        assert!(trim_magic(source).is_none());
    }

    #[test]
    fn trim_comment_trims_comment_tags() {
        let source = Spanned::from_str("<!-- test -->");
        let trimmed = trim_comment(source).unwrap();
        source.assert_spanned_str(trimmed, "test");

        let source = Spanned::from_str("<!--test-->");
        let trimmed = trim_comment(source).unwrap();
        source.assert_spanned_str(trimmed, "test");
    }

    #[test]
    fn trim_comment_ignores_non_comment() {
        let source = Spanned::from_str("test");
        assert!(trim_comment(source).is_none());
        let source = Spanned::from_str("<!--test");
        assert!(trim_comment(source).is_none());
        let source = Spanned::from_str("<p>paragraph</p>");
        assert!(trim_comment(source).is_none());
    }

    #[test]
    fn next_token_parses_tokens() {
        let source = Spanned::from_str("kind:group kind-x : group_y[[ ]]");
        let (token, rest) = next_token(source).unwrap();
        assert_eq!(token.value, Token::Ident("kind"));
        source.assert_spanned(token, "kind");
        source.assert_spanned_str(rest, ":group kind-x : group_y[[ ]]");

        let (token, rest) = next_token(rest).unwrap();
        assert_eq!(token.value, Token::Colon);
        source.assert_spanned(token, ":");
        source.assert_spanned_str(rest, "group kind-x : group_y[[ ]]");

        let (token, rest) = next_token(rest).unwrap();
        assert_eq!(token.value, Token::Ident("group"));
        source.assert_spanned(token, "group");
        source.assert_spanned_str(rest, " kind-x : group_y[[ ]]");

        let (token, rest) = next_token(rest).unwrap();
        assert_eq!(token.value, Token::Ident("kind-x"));
        source.assert_spanned(token, "kind-x");
        source.assert_spanned_str(rest, " : group_y[[ ]]");

        let (token, rest) = next_token(rest).unwrap();
        assert_eq!(token.value, Token::Colon);
        source.assert_spanned(token, ":");
        source.assert_spanned_str(rest, " group_y[[ ]]");

        let (token, rest) = next_token(rest).unwrap();
        assert_eq!(token.value, Token::Ident("group_y"));
        source.assert_spanned(token, "group_y");
        source.assert_spanned_str(rest, "[[ ]]");

        let (token, rest) = next_token(rest).unwrap();
        assert_eq!(token.value, Token::StartMarkerSymbol);
        source.assert_spanned(token, "[[");
        source.assert_spanned_str(rest, " ]]");

        let (token, rest) = next_token(rest).unwrap();
        assert_eq!(token.value, Token::EndMarkerSymbol);
        source.assert_spanned(token, "]]");
        source.assert_spanned_str(rest, "");
    }

    #[test]
    fn next_token_returns_none_for_str_without_tokens() {
        let source = Spanned::from_str("  ");
        assert!(next_token(source).is_none());
    }

    #[test]
    fn next_token_returns_unknown_tokens() {
        let source = Spanned::from_str("1x x123@#");
        let (token, rest) = next_token(source).unwrap();
        assert_eq!(token.value, Token::UnknownChar("1"));
        source.assert_spanned(token, "1");
        source.assert_spanned_str(rest, "x x123@#");

        let (token, rest) = next_token(rest).unwrap();
        assert_eq!(token.value, Token::Ident("x"));
        source.assert_spanned(token, "x");
        source.assert_spanned_str(rest, " x123@#");

        let (token, rest) = next_token(rest).unwrap();
        assert_eq!(token.value, Token::Ident("x123"));
        source.assert_spanned(token, "x123");
        source.assert_spanned_str(rest, "@#");

        let (token, rest) = next_token(rest).unwrap();
        assert_eq!(token.value, Token::UnknownChar("@"));
        source.assert_spanned(token, "@");
        source.assert_spanned_str(rest, "#");
    }

    #[test]
    fn parser_returns_substr_of_source() {
        let source = indoc! {"
            some text
            <!-- cargo-sync-rdme kind:group [[ -->
            more text
            <!-- cargo-sync-rdme ]] -->
            end text
            <!-- cargo-sync-rdme kind:group -->
        "};
        let source = Spanned::from_str(source);
        let mut parser = MarkerParser::new(source.value);

        let marker = parser.try_next().unwrap().unwrap();
        source.assert_spanned(marker, "<!-- cargo-sync-rdme kind:group [[ -->");
        let specifier = marker.value.into_start();
        source.assert_spanned(specifier, "kind:group");
        source.assert_spanned_str(specifier.value.kind, "kind");
        source.assert_spanned_str(specifier.value.group.unwrap(), "group");

        let marker = parser.try_next().unwrap().unwrap();
        source.assert_spanned(marker, "<!-- cargo-sync-rdme ]] -->");
        marker.value.into_end();

        let marker = parser.try_next().unwrap().unwrap();
        source.assert_spanned(marker, "<!-- cargo-sync-rdme kind:group -->");
        let specifier = marker.value.into_replace();
        source.assert_spanned(specifier, "kind:group");
        source.assert_spanned_str(specifier.value.kind, "kind");
        source.assert_spanned_str(specifier.value.group.unwrap(), "group");

        assert!(parser.try_next().unwrap().is_none());
    }

    #[test]
    fn nul_character_conversion_does_not_affect_error_span() {
        let source = indoc! {"
            some text
            <!-- cargo-sync-rdme kind:group [[ \0 -->
            more text
            <!-- cargo-sync-rdme ]] \0 -->
            end text
            <!-- cargo-sync-rdme kind:group \0 -->
        "};
        let source = Spanned::from_str(source);
        let mut parser = MarkerParser::new(source.value);

        let (token, expected, span) = parser.try_next().unwrap_err().into_unexpected_token();
        assert_eq!(token, "\0");
        assert_eq!(expected, "end of marker");
        source.assert_source_span(span, "\0");

        let (token, expected, span) = parser.try_next().unwrap_err().into_unexpected_token();
        assert_eq!(token, "\0");
        assert_eq!(expected, "end of marker");
        source.assert_source_span(span, "\0");

        let (token, expected, span) = parser.try_next().unwrap_err().into_unexpected_token();
        assert_eq!(token, "\0");
        assert_eq!(expected, "end of marker or `[[`");
        source.assert_source_span(span, "\0");

        assert!(parser.try_next().unwrap().is_none());
    }
}

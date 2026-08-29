use pulldown_cmark::Event;

use crate::sync::marker;

pub(super) fn convert<'a, I>(events: I) -> impl Iterator<Item = Event<'a>>
where
    I: IntoIterator<Item = Event<'a>>,
{
    events.into_iter().map(|event| {
        let mut event = event;
        marker::escape_marker(&mut event);
        event
    })
}

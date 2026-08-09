use std::{borrow::Cow, collections::HashMap};

use pulldown_cmark::{BrokenLink, BrokenLinkCallback, CowStr, Event, Options, Parser, Tag};
use rustdoc_types::{Id, Item};

use crate::sync::contents::rustdoc::document::IntraLinkResolver;

#[derive(Debug)]
pub(super) struct LinkMappingConfig<'map, 'url> {
    mappings: &'map HashMap<String, String>,
    local_html_root_url: &'url str,
}

impl<'map, 'url> LinkMappingConfig<'map, 'url> {
    pub(super) fn new(
        mappings: &'map HashMap<String, String>,
        local_html_root_url: &'url str,
    ) -> Self {
        Self {
            mappings,
            local_html_root_url,
        }
    }

    pub(super) fn build_mapper<'doc>(
        &self,
        resolver: &IntraLinkResolver<'_>,
        item: &'doc Item,
    ) -> Option<LinkMapper<'doc, 'map>> {
        let docs = item.docs.as_deref()?;
        let map = item
            .links
            .iter()
            .map(|(name, id)| (name.as_str(), resolve_link(resolver, self, name, *id)))
            .collect();
        Some(LinkMapper { docs, map })
    }
}

#[tracing::instrument(skip_all, fields(name = name, id = ?id))]
fn resolve_link<'map>(
    resolver: &IntraLinkResolver<'_>,
    config: &LinkMappingConfig<'map, '_>,
    name: &str,
    id: Id,
) -> Option<Cow<'map, str>> {
    if let Some(url) = config.mappings.get(name) {
        return Some(url.into());
    }
    let Some(url) = resolver
        .resolve_link(id)
        .and_then(|target| target.build_url(config.local_html_root_url))
    else {
        tracing::warn!("failed to resolve link");
        return None;
    };
    Some(url.into())
}

#[derive(Debug)]
pub(super) struct LinkMapper<'doc, 'map> {
    docs: &'doc str,
    map: HashMap<&'doc str, Option<Cow<'map, str>>>,
}

impl<'url, 'input> BrokenLinkCallback<'input> for &LinkMapper<'_, 'url>
where
    'url: 'input,
{
    fn handle_broken_link(
        &mut self,
        link: BrokenLink<'input>,
    ) -> Option<(CowStr<'input>, CowStr<'input>)> {
        let target = self.map.get(&*link.reference)?.as_ref()?;
        Some((target.clone().into(), "".into()))
    }
}

impl LinkMapper<'_, '_> {
    pub(super) fn build_parser(&self) -> impl Iterator<Item = Event<'_>> {
        Parser::new_with_broken_link_callback(self.docs, Options::all(), Some(self)).map(
            |mut event| {
                if let Event::Start(Tag::Link { dest_url: url, .. }) = &mut event
                    && let Some(Some(full_url)) = self.map.get(url.as_ref())
                {
                    *url = full_url.clone().into();
                }
                event
            },
        )
    }
}

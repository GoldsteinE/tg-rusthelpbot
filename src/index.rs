#![allow(dead_code)]

use std::collections::HashMap;

use crate::rustdoc_types::{self, FetchedSearchIndex, ItemType, Path, SearchIndex};

pub type RawItem = rustdoc_types::Item;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Item {
    ty: ItemType,
    url: String,
    desc: String,
}

fn format_url(
    base_url: &str,
    path: &str,
    ty: ItemType,
    name: &str,
    frag_ty: ItemType,
    frag_name: &str,
) -> String {
    let ty = ty.to_url_slug();
    let frag_ty = frag_ty.to_url_slug();
    let capacity =
        base_url.len() + path.len() + ty.len() + name.len() + frag_ty.len() + frag_name.len() + 10;
    let mut result = String::with_capacity(capacity);
    result.push_str(base_url);
    result.push('/');
    for part in path.split("::") {
        result.push_str(part);
        result.push('/');
    }
    result.push_str(ty);
    result.push('.');
    result.push_str(name);
    result.push_str(".html#");
    result.push_str(frag_ty);
    result.push('.');
    result.push_str(frag_name);
    result
}

impl Item {
    // Yes, this _is_ in fact horrible.
    // I'll leave fold marker for you: {{{
    pub fn try_from_rustdoc(
        parent: Option<&Path>,
        raw: RawItem,
        base_url: &str,
    ) -> Option<(String, Self)> {
        match raw.ty {
            ItemType::ExternCrate | ItemType::Primitive | ItemType::Keyword => {
                /* We don't want to process these */
                None
            }
            ItemType::Mod => {
                let RawItem {
                    ty,
                    mut path,
                    name,
                    desc,
                    ..
                } = raw;
                let mut url = path.replace("::", "/");
                url.push('/');
                url.push_str(&name);
                url.push_str("/index.html");
                path.push_str("::");
                path.push_str(&name);
                Some((path, Item { ty, url, desc }))
            }
            _ => match parent {
                Some(parent) => match (raw.ty, parent.ty) {
                    (_, ItemType::Primitive) => {
                        let path = [&parent.name, "::", &raw.name].concat();
                        let url = format_url(
                            base_url,
                            &path,
                            ItemType::Primitive,
                            &parent.name,
                            raw.ty,
                            &raw.name,
                        );
                        Some((
                            path,
                            Item {
                                ty: raw.ty,
                                url,
                                desc: raw.desc,
                            },
                        ))
                    }
                    (ItemType::Structfield, ItemType::Variant) => {
                        let (path, enum_name) = match raw
                            .path
                            .rfind("::")
                            .map(|i| (&raw.path[..i], &raw.path[i + 2..]))
                        {
                            Some(x) => x,
                            None => {
                                log::warn!("failed to split structfield variant");
                                return None;
                            }
                        };
                        let full_path = [&raw.path, "::", &parent.name, "::", &raw.name].concat();
                        let url = format_url(
                            base_url,
                            &path,
                            ItemType::Variant,
                            &raw.name,
                            ItemType::Variant,
                            &format!("{}.field.{}", enum_name, raw.name),
                        );
                        Some((
                            full_path,
                            Item {
                                ty: raw.ty,
                                url,
                                desc: raw.desc,
                            },
                        ))
                    }
                    _ => {
                        let full_path = [&raw.path, "::", &parent.name, "::", &raw.name].concat();
                        let url = format_url(
                            base_url,
                            &raw.path,
                            parent.ty,
                            &parent.name,
                            raw.ty,
                            &raw.name,
                        );
                        Some((
                            full_path,
                            Item {
                                ty: raw.ty,
                                url,
                                desc: raw.desc,
                            },
                        ))
                    }
                },
                None => {
                    let path = [&raw.path, "::", &raw.name].concat();
                    let url = [
                        &raw.path.replace("::", "/"),
                        "/",
                        raw.ty.to_url_slug(),
                        ".",
                        &raw.name,
                        ".html",
                    ]
                    .concat();
                    Some((
                        path,
                        Item {
                            ty: raw.ty,
                            url,
                            desc: raw.desc,
                        },
                    ))
                }
            },
        }
    }
    // }}}
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct Index {
    items: HashMap<String, Item>,
}

impl Index {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn populate_from_rustdoc(&mut self, index: FetchedSearchIndex) {
        let FetchedSearchIndex {
            base_url,
            index: SearchIndex {
                mut items, paths, ..
            },
        } = index;

        // Fix empty paths
        let mut last_path = String::new();
        for item in &mut items {
            if item.path.is_empty() {
                item.path = last_path.clone();
            }
            last_path = item.path.clone();
        }

        self.items.extend(
            items
                .into_iter()
                .filter_map(|it| {
                    if let Some(parent_idx) = it.parent {
                        let parent = if let Some(parent) = paths.get(parent_idx) {
                            parent
                        } else {
                            log::warn!("parent idx {} is not in list", parent_idx);
                            return None;
                        };
                        Some((Some(parent), it))
                    } else {
                        Some((None, it))
                    }
                })
                .filter_map(|(parent, it)| Item::try_from_rustdoc(parent, it, &base_url)),
        );
    }

    pub fn from_rustdoc(index: FetchedSearchIndex) -> Self {
        let mut result = Self::new();
        result.populate_from_rustdoc(index);
        result
    }
}

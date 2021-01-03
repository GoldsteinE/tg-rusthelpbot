#![allow(dead_code)]

use std::collections::HashMap;

use crate::rustdoc_types::{self, ItemType, Path, SearchIndex};

pub type RawItem = rustdoc_types::Item;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Item {
    ty: ItemType,
    url: String,
    desc: String,
}

impl Item {
    pub fn try_from_rustdoc(paths: &[Path], raw: RawItem) -> Option<(String, Self)> {
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
            _ => match raw.parent {
                Some(parent_idx) => {
                    let parent = if let Some(parent) = paths.get(parent_idx) {
                        parent
                    } else {
                        log::warn!("failed to get parent_idx {} from {:?}", parent_idx, paths);
                        return None;
                    };
                    match (raw.ty, parent.ty) {
                        (_, ItemType::Primitive) => {
                            let path = [&parent.name, "::", &raw.name].concat();
                            let url = [
                                &raw.path.replace("::", "/"),
                                "/primitive.",
                                &parent.name,
                                ".html#",
                                &raw.ty.to_url_slug(),
                                ".",
                                &raw.name,
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
                            let full_path =
                                [&raw.path, "::", &parent.name, "::", &raw.name].concat();
                            let url = [
                                &path.replace("::", "/"),
                                "/variant.",
                                &raw.name,
                                ".html#variant.",
                                enum_name,
                                ".field.",
                                &raw.name,
                            ]
                            .concat();
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
                            let full_path =
                                [&raw.path, "::", &parent.name, "::", &raw.name].concat();
                            let url = [
                                &raw.path.replace("::", "/"),
                                "/",
                                parent.ty.to_url_slug(),
                                ".",
                                &parent.name,
                                ".html#",
                                raw.ty.to_url_slug(),
                                ".",
                                &raw.name,
                            ]
                            .concat();
                            Some((
                                full_path,
                                Item {
                                    ty: raw.ty,
                                    url,
                                    desc: raw.desc,
                                },
                            ))
                        }
                    }
                }
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
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct Index {
    items: HashMap<String, Item>,
}

impl Index {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn populate_from_rustdoc(&mut self, index: SearchIndex) {
        let SearchIndex { mut items, paths, .. } = index;

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
                .filter_map(|it| Item::try_from_rustdoc(&paths, it)),
        );
    }

    pub fn from_rustdoc(index: SearchIndex) -> Self {
        let mut result = Self::new();
        result.populate_from_rustdoc(index);
        result
    }
}

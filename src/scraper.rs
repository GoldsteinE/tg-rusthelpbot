#![allow(dead_code)]

use std::{collections::HashMap, fmt};

use color_eyre::eyre::{self, WrapErr as _};
use once_cell::sync::OnceCell;
use rusty_v8 as v8;

use crate::rustdoc_types::{FetchedSearchIndex, SearchIndex};

static V8_INIT: OnceCell<()> = OnceCell::new();
// It's probably better to create these via v8, but I dunno how to
static JS_PRELUDE: &str = r#"
function addSearchOptions() { /* ignore */ }
function initSearch() { /* ignore */ }
"#;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[derive(Debug)]
pub struct Scraper {
    http: reqwest::Client,
    js_engine: v8::OwnedIsolate,
}

impl Scraper {
    pub fn new() -> reqwest::Result<Self> {
        let mut hm = reqwest::header::HeaderMap::new();
        hm.insert(
            "X-Contact-Me",
            "https://github.com/GoldsteinE"
                .parse()
                .expect("failed to parse header value"),
        );
        let http = reqwest::Client::builder()
            .user_agent(APP_USER_AGENT)
            .default_headers(hm)
            .build()?;

        V8_INIT.get_or_init(|| {
            let platform = v8::new_default_platform().unwrap();
            v8::V8::initialize_platform(platform);
            v8::V8::initialize();
        });

        let js_engine = v8::Isolate::new(Default::default());
        Ok(Self { http, js_engine })
    }

    pub async fn find_index_url(
        &self,
        crate_name: impl fmt::Display,
    ) -> eyre::Result<reqwest::Url> {
        use select::{document::Document, predicate::Name};

        let url = format!("https://docs.rs/{}", crate_name);
        let resp = self.http.get(&url).send().await?;
        let base_url = resp.url().clone();
        let html = resp.text().await?;
        let doc = Document::from(html.as_str());
        doc.find(Name("script"))
            .filter_map(|node| node.attr("src").filter(|src| src.contains("search-index")))
            .next()
            .ok_or_else(|| eyre::eyre!("failed to find search index URL for crate {}", crate_name))
            .and_then(|uri| {
                base_url
                    .join(uri)
                    .wrap_err("failed to parse search index URL")
            })
    }

    pub async fn fetch_index_by_url(
        &mut self,
        url: impl reqwest::IntoUrl,
    ) -> eyre::Result<HashMap<String, FetchedSearchIndex>> {
        let url = url.into_url()?;
        let mut base_url = url.clone();
        base_url
            .path_segments_mut()
            .map_err(|()| eyre::eyre!("invalid URL passed to fetch_index_by_url(): {}", url))?
            .pop();
        let base_url = base_url.to_string();
        let js_string = [JS_PRELUDE, &self.http.get(url).send().await?.text().await?].concat();
        let scope = &mut v8::HandleScope::new(&mut self.js_engine);
        let context = v8::Context::new(scope);
        let scope = &mut v8::ContextScope::new(scope, context);
        let code = v8::String::new(scope, &js_string)
            .ok_or_else(|| eyre::eyre!("failed to convert code to JS string"))?;
        let script = v8::Script::compile(scope, code, None)
            .ok_or_else(|| eyre::eyre!("failed to compile JS script"))?;
        script.run(scope);
        let key = v8::String::new(scope, "searchIndex")
            .ok_or_else(|| eyre::eyre!("failed to convert `searchIndex` to JS string"))?;
        let global = context.global(scope);
        let search_index_var = global
            .get(scope, key.into())
            .ok_or_else(|| eyre::eyre!("searchIndex var is not set after executing JS script"))?;
        let search_index = v8::json::stringify(scope, search_index_var)
            .ok_or_else(|| eyre::eyre!("failed to stringify searchIndex"))?;
        let search_index_json = search_index.to_rust_string_lossy(scope);
        let indices: HashMap<String, SearchIndex> = serde_json::from_str(&search_index_json)
            .wrap_err("failed to parse search index JSON")?;
        Ok(indices
            .into_iter()
            .map(|(k, index)| {
                (
                    k,
                    FetchedSearchIndex {
                        base_url: base_url.clone(),
                        index,
                    },
                )
            })
            .collect())
    }
}

use async_std::stream::StreamExt;
use log::debug;
use lychee_lib::{Collector, Input, InputSource::*, Request, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::string::String;
use wildmatch::WildMatch;

/// Collect links from file
pub async fn collect_links(
    path: &str,
    ignored_uri_wildcards: &Vec<String>,
) -> Result<HashMap<String, Request>> {
    debug!(
        "Collecting links in file: {:#?}, ignoring these links: {:#?}",
        &path, &ignored_uri_wildcards
    );

    let input = vec![Input {
        source: FsPath(PathBuf::from(path)),
        file_type_hint: None,
        excluded_paths: None,
    }];
    debug!("Lychee inputs:\n{:#?}", &input);

    let links = Collector::new(None) // base
        .skip_missing_inputs(false) // don't skip missing inputs? (default=false)
        .use_html5ever(false) // use html5ever for parsing? (default=false)
        .include_verbatim(true)
        .collect_links(input)
        .await // base url or directory
        .collect::<Result<Vec<_>>>()
        .await?;
    debug!("Found links:\n{:#?}", &links);

    // Dedup them
    let mut links_map: HashMap<String, Request> = HashMap::new();
    for link in links {
        let uri = link.uri.as_str();
        let matches_any_ignored_uri_wildcard =
            ignored_uri_wildcards.iter().any(|ignored_wildcard| {
                if let Some(stripped_uri) = uri.strip_suffix('/') {
                    WildMatch::new(ignored_wildcard).matches(stripped_uri)
                } else {
                    WildMatch::new(ignored_wildcard).matches(uri)
                }
            });
        if !matches_any_ignored_uri_wildcard {
            links_map.insert(uri.to_string(), link.clone());
        }
    }
    debug!("De-duplicated links:\n{:#?}", &links_map);

    Ok(links_map)
}
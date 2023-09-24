use core::time;
use std::path::{Path, PathBuf};
use std::{cmp, collections::HashMap, collections::HashSet, iter::FromIterator, thread};

use config::LinkCheckerLevel;
use libs::rayon::prelude::*;

use crate::Site;
use errors::{bail, Result};
use libs::rayon;
use libs::url::Url;

/// Check whether all internal links pointing to explicit anchor fragments are valid.
///
/// This is very similar to `check_external_links`, although internal links checking
/// is always performed (while external ones only conditionally in `zola check`).  If broken links
/// are encountered, the `internal_level` setting in config.toml will determine whether they are
/// treated as warnings or errors.
pub fn check_internal_links_with_anchors(site: &Site) -> Vec<String> {
    println!("Checking all internal links with anchors.");
    let library = site.library.write().expect("Get lock for check_internal_links_with_anchors");

    // Chain all internal links, from both sections and pages.
    let page_links = library
        .pages
        .values()
        .flat_map(|p| p.internal_links.iter().map(move |l| (p.file.path.clone(), l)));
    let section_links = library
        .sections
        .values()
        .flat_map(|p| p.internal_links.iter().map(move |l| (p.file.path.clone(), l)));
    let all_links = page_links.chain(section_links);

    // Only keep links with anchor fragments, and count them too.
    // Bare files have already been checked elsewhere, thus they are not interesting here.
    let mut anchors_total = 0usize;
    let links_with_anchors = all_links
        .filter_map(|(page_path, link)| match link {
            (md_path, Some(anchor)) => Some((page_path, md_path, anchor)),
            _ => None,
        })
        .inspect(|_| anchors_total = anchors_total.saturating_add(1));

    // Check for targets existence (including anchors), then keep only the faulty
    // entries for error reporting purposes.
    let missing_targets = links_with_anchors.filter(|(page, md_path, anchor)| {
        // There are a few `expect` here since the presence of the .md file will
        // already have been checked in the markdown rendering
        let mut full_path = site.base_path.clone();
        full_path.push("content");
        for part in md_path.split('/') {
            full_path.push(part);
        }
        // NOTE: This will also match _index.foobar.md where foobar is not a language
        // as well as any other string containing "_index." which is now referenced as
        // unsupported page path in the docs.
        if md_path.contains("_index.") {
            let section = library.sections.get(&full_path).unwrap_or_else(|| {
                panic!(
                    "Couldn't find section {} in check_internal_links_with_anchors from page {:?}",
                    md_path,
                    page.strip_prefix(&site.base_path).unwrap()
                )
            });
            !section.has_anchor(anchor)
        } else {
            let page = library.pages.get(&full_path).unwrap_or_else(|| {
                panic!(
                    "Couldn't find page {} in check_internal_links_with_anchors from page {:?}",
                    md_path,
                    page.strip_prefix(&site.base_path).unwrap()
                )
            });

            !(page.has_anchor(anchor) || page.has_anchor_id(anchor))
        }
    });

    // Format faulty entries into error messages, and collect them.
    let messages = missing_targets
        .map(|(page_path, md_path, anchor)| {
            format!(
                "The anchor in the link `@/{}#{}` in {} does not exist.",
                md_path,
                anchor,
                page_path.to_string_lossy(),
            )
        })
        .collect::<Vec<_>>();

    // Finally emit a summary, and return overall anchors-checking result.
    if messages.is_empty() {
        println!("> Successfully checked {} internal link(s) with anchors.", anchors_total);
    } else {
        println!(
            "> Checked {} internal link(s) with anchors: {} target(s) missing.",
            anchors_total,
            messages.len(),
        );
    }
    messages
}

fn should_skip_by_prefix(link: &str, skip_prefixes: &[String]) -> bool {
    skip_prefixes.iter().any(|prefix| link.starts_with(prefix))
}

fn get_link_domain(link: &str) -> Result<String> {
    return match Url::parse(link) {
        Ok(url) => match url.host_str().map(String::from) {
            Some(domain_str) => Ok(domain_str),
            None => bail!("could not parse domain `{}` from link", link),
        },
        Err(err) => bail!("could not parse domain `{}` from link: `{}`", link, err),
    };
}

/// Checks all external links and returns all the errors that were encountered.
/// Empty vec == all good
pub fn check_external_links(site: &Site) -> Vec<String> {
    let library = site.library.write().expect("Get lock for check_external_links");

    struct LinkDef {
        file_path: PathBuf,
        external_link: String,
        domain: String,
    }

    impl LinkDef {
        pub fn new(file_path: &Path, external_link: &str, domain: String) -> Self {
            Self {
                file_path: file_path.to_path_buf(),
                external_link: external_link.to_string(),
                domain,
            }
        }
    }

    let mut messages: Vec<String> = vec![];
    let mut external_links = Vec::new();
    for p in library.pages.values() {
        external_links.push((&p.file.path, &p.external_links));
    }
    for s in library.sections.values() {
        external_links.push((&s.file.path, &s.external_links));
    }

    let mut checked_links: Vec<LinkDef> = vec![];
    let mut skipped_link_count: u32 = 0;
    let mut invalid_url_links: u32 = 0;
    // First we look at all the external links, skip those the user wants to skip and record
    // the ones that have invalid URLs
    for (file_path, links) in external_links {
        for link in links {
            if should_skip_by_prefix(link, &site.config.link_checker.skip_prefixes) {
                skipped_link_count += 1;
            } else {
                match get_link_domain(link) {
                    Ok(domain) => {
                        checked_links.push(LinkDef::new(file_path, link, domain));
                    }
                    Err(err) => {
                        // We could use the messages.len() to keep track of them for below
                        // but it's more explicit this way
                        invalid_url_links += 1;
                        messages.push(err.to_string());
                    }
                }
            }
        }
    }

    println!(
        "Checking {} external link(s). Skipping {} external link(s).{}",
        // Get unique links count from Vec by creating a temporary HashSet.
        HashSet::<&str>::from_iter(
            checked_links.iter().map(|link_def| link_def.external_link.as_str())
        )
        .len(),
        skipped_link_count,
        if invalid_url_links == 0 {
            "".to_string()
        } else {
            format!(" {} link(s) had unparseable URLs.", invalid_url_links)
        }
    );

    if checked_links.is_empty() {
        return Vec::new();
    }

    // error out if we're in error mode and any external URLs couldn't be parsed
    if site.config.link_checker.external_level == LinkCheckerLevel::Error && !messages.is_empty() {
        return messages;
    }

    let mut links_by_domain: HashMap<&str, Vec<&LinkDef>> = HashMap::new();
    for link in checked_links.iter() {
        if links_by_domain.contains_key(link.domain.as_str()) {
            links_by_domain.get_mut(link.domain.as_str()).unwrap().push(link);
        } else {
            links_by_domain.insert(link.domain.as_str(), vec![link]);
        }
    }

    let cpu_count = match thread::available_parallelism() {
        Ok(count) => count.get(),
        Err(_) => 1,
    };
    // create thread pool with lots of threads so we can fetch
    // (almost) all pages simultaneously, limiting all links for a single
    // domain to one thread to avoid rate-limiting
    let num_threads = cmp::min(links_by_domain.len(), cmp::max(8, cpu_count));
    match rayon::ThreadPoolBuilder::new().num_threads(num_threads).build() {
        Ok(pool) => {
            let errors = pool.install(|| {
                links_by_domain
                    .par_iter()
                    .map(|(_, links)| {
                        let mut num_links_left = links.len();
                        let mut checked_links: HashMap<&str, Option<link_checker::Result>> =
                            HashMap::new();
                        links
                            .iter()
                            .filter_map(move |link_def| {
                                num_links_left -= 1;

                                // Avoid double-checking the same url (e.g. for translated pages).
                                let external_link = link_def.external_link.as_str();
                                if let Some(optional_res) = checked_links.get(external_link) {
                                    if let Some(res) = optional_res {
                                        return Some((
                                            &link_def.file_path,
                                            external_link,
                                            res.clone(),
                                        ));
                                    }
                                    return None;
                                }

                                let res = link_checker::check_url(
                                    external_link,
                                    &site.config.link_checker,
                                );

                                if num_links_left > 0 {
                                    // Prevent rate-limiting, wait before next crawl unless we're done with this domain
                                    thread::sleep(time::Duration::from_millis(500));
                                }

                                if link_checker::is_valid(&res) {
                                    checked_links.insert(external_link, None);
                                    None
                                } else {
                                    checked_links.insert(external_link, Some(res.clone()));
                                    Some((&link_def.file_path, external_link, res))
                                }
                            })
                            .collect::<Vec<_>>()
                    })
                    .flatten()
                    .collect::<Vec<_>>()
            });

            println!(
                "> Checked {} external link(s): {} error(s) found.",
                checked_links.len(),
                errors.len()
            );

            for (page_path, link, check_res) in errors {
                messages.push(format!(
                    "Broken link in {} to {}: {}",
                    page_path.to_string_lossy(),
                    link,
                    link_checker::message(&check_res)
                ));
            }
        }
        Err(pool_err) => messages.push(pool_err.to_string()),
    }

    messages
}

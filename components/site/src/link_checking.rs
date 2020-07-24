use rayon::prelude::*;

use crate::Site;
use errors::{Error, ErrorKind, Result};

/// Very similar to check_external_links but can't be merged as far as I can see since we always
/// want to check the internal links but only the external in zola check :/
pub fn check_internal_links_with_anchors(site: &Site) -> Result<()> {
    let library = site.library.write().expect("Get lock for check_internal_links_with_anchors");
    let page_links = library
        .pages()
        .values()
        .map(|p| {
            let path = &p.file.path;
            p.internal_links_with_anchors.iter().map(move |l| (path.clone(), l))
        })
        .flatten();
    let section_links = library
        .sections()
        .values()
        .map(|p| {
            let path = &p.file.path;
            p.internal_links_with_anchors.iter().map(move |l| (path.clone(), l))
        })
        .flatten();
    let all_links = page_links.chain(section_links).collect::<Vec<_>>();

    if site.config.is_in_check_mode() {
        println!("Checking {} internal link(s) with an anchor.", all_links.len());
    }

    if all_links.is_empty() {
        return Ok(());
    }

    let mut full_path = site.base_path.clone();
    full_path.push("content");

    let errors: Vec<_> = all_links
        .iter()
        .filter_map(|(page_path, (md_path, anchor))| {
            // There are a few `expect` here since the presence of the .md file will
            // already have been checked in the markdown rendering
            let mut p = full_path.clone();
            for part in md_path.split('/') {
                p.push(part);
            }
            if md_path.contains("_index.md") {
                let section = library
                    .get_section(&p)
                    .expect("Couldn't find section in check_internal_links_with_anchors");
                if section.has_anchor(&anchor) {
                    None
                } else {
                    Some((page_path, md_path, anchor))
                }
            } else {
                let page = library
                    .get_page(&p)
                    .expect("Couldn't find section in check_internal_links_with_anchors");
                if page.has_anchor(&anchor) {
                    None
                } else {
                    Some((page_path, md_path, anchor))
                }
            }
        })
        .collect();

    if site.config.is_in_check_mode() {
        println!(
            "> Checked {} internal link(s) with an anchor: {} error(s) found.",
            all_links.len(),
            errors.len()
        );
    }

    if errors.is_empty() {
        return Ok(());
    }

    let msg = errors
        .into_iter()
        .map(|(page_path, md_path, anchor)| {
            format!(
                "The anchor in the link `@/{}#{}` in {} does not exist.",
                md_path,
                anchor,
                page_path.to_string_lossy(),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    Err(Error { kind: ErrorKind::Msg(msg), source: None })
}

pub fn check_external_links(site: &Site) -> Result<()> {
    let library = site.library.write().expect("Get lock for check_external_links");
    let page_links = library
        .pages()
        .values()
        .map(|p| {
            let path = &p.file.path;
            p.external_links.iter().map(move |l| (path.clone(), l))
        })
        .flatten();
    let section_links = library
        .sections()
        .values()
        .map(|p| {
            let path = &p.file.path;
            p.external_links.iter().map(move |l| (path.clone(), l))
        })
        .flatten();
    let all_links = page_links.chain(section_links).collect::<Vec<_>>();
    println!("Checking {} external link(s).", all_links.len());

    if all_links.is_empty() {
        return Ok(());
    }

    // create thread pool with lots of threads so we can fetch
    // (almost) all pages simultaneously
    let threads = std::cmp::min(all_links.len(), 32);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .map_err(|e| Error { kind: ErrorKind::Msg(e.to_string()), source: None })?;

    let errors: Vec<_> = pool.install(|| {
        all_links
            .par_iter()
            .filter_map(|(page_path, link)| {
                if site
                    .config
                    .link_checker
                    .skip_prefixes
                    .iter()
                    .any(|prefix| link.starts_with(prefix))
                {
                    return None;
                }
                let res = link_checker::check_url(&link, &site.config.link_checker);
                if link_checker::is_valid(&res) {
                    None
                } else {
                    Some((page_path, link, res))
                }
            })
            .collect()
    });

    println!("> Checked {} external link(s): {} error(s) found.", all_links.len(), errors.len());

    if errors.is_empty() {
        return Ok(());
    }

    let msg = errors
        .into_iter()
        .map(|(page_path, link, check_res)| {
            format!(
                "Dead link in {} to {}: {}",
                page_path.to_string_lossy(),
                link,
                link_checker::message(&check_res)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    Err(Error { kind: ErrorKind::Msg(msg), source: None })
}

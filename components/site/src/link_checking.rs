use rayon::prelude::*;

use crate::Site;
use errors::{Error, ErrorKind, Result};

/// Check whether all internal links pointing to explicit anchor fragments are valid.
///
/// This is very similar to `check_external_links`, although internal links checking
/// is always performed (while external ones only conditionally in `zola check`).
pub fn check_internal_links_with_anchors(site: &Site) -> Result<()> {
    println!("Checking all internal links with anchors.");
    let library = site.library.write().expect("Get lock for check_internal_links_with_anchors");

    // Chain all internal links, from both sections and pages.
    let page_links = library
        .pages()
        .values()
        .map(|p| {
            let path = &p.file.path;
            p.internal_links.iter().map(move |l| (path.clone(), l))
        })
        .flatten();
    let section_links = library
        .sections()
        .values()
        .map(|p| {
            let path = &p.file.path;
            p.internal_links.iter().map(move |l| (path.clone(), l))
        })
        .flatten();
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
    let missing_targets = links_with_anchors.filter(|(_, md_path, anchor)| {
        // There are a few `expect` here since the presence of the .md file will
        // already have been checked in the markdown rendering
        let mut full_path = site.base_path.clone();
        full_path.push("content");
        for part in md_path.split('/') {
            full_path.push(part);
        }
        if md_path.contains("_index.md") {
            let section = library
                .get_section(&full_path)
                .expect("Couldn't find section in check_internal_links_with_anchors");
            !section.has_anchor(&anchor)
        } else {
            let page = library
                .get_page(&full_path)
                .expect("Couldn't find section in check_internal_links_with_anchors");
            !page.has_anchor(&anchor)
        }
    });

    // Format faulty entries into error messages, and collect them.
    let errors = missing_targets
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
    match errors.len() {
        0 => {
            println!("> Succesfully checked {} internal link(s) with anchors.", anchors_total);
            Ok(())
        }
        errors_total => {
            println!(
                "> Checked {} internal link(s) with anchors: {} target(s) missing.",
                anchors_total, errors_total,
            );
            Err(Error { kind: ErrorKind::Msg(errors.join("\n")), source: None })
        }
    }
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

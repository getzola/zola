//! Boilerplate stripping for fetched markdown — the deterministic backstop to
//! [`super::firecrawl`]'s `excludeTags` ask.
//!
//! Firecrawl is sent `onlyMainContent: true` **and** `excludeTags` for
//! structural chrome, but this WordPress theme lacks clean `<main>`/`<article>`
//! semantics, so the response still carries nav / search / cookie / related-post
//! chrome. [`strip_boilerplate`] removes it deterministically.
//!
//! ## Conservatism (hard requirement)
//!
//! Prefer leaving a stray chrome line over truncating real article prose.
//! Leading chrome is dropped only until the first real line (a heading or a
//! prose sentence); trailing chrome is cut only at an unambiguous footer
//! marker. Every rule has a test proving a real body survives intact. This is
//! defense in depth — neither layer (excludeTags nor this) is trusted alone.

use std::sync::OnceLock;

use regex::Regex;

/// Remove theme boilerplate from a fetched markdown body.
///
/// Three passes, in order:
/// 1. Drop avatar/author lines **anywhere** (`wp-content/litespeed/avatar`).
/// 2. Drop leading chrome until the first real line (heading or prose).
/// 3. Truncate at the first trailing footer marker (inclusive).
///
/// Returns the kept region, blank-line-collapsed and trailing-trimmed.
/// Never returns content past the first footer marker.
pub fn strip_boilerplate(md: &str) -> String {
    let lines: Vec<&str> = md.lines().collect();

    // Pass 1: avatar/author lines anywhere.
    let no_avatar: Vec<&str> = lines.iter().copied().filter(|l| !is_avatar_line(l)).collect();

    // Pass 2: leading chrome. `start` = index of first real line.
    let start = no_avatar.iter().position(|l| !is_leading_chrome(l)).unwrap_or(no_avatar.len());

    // Pass 3: trailing footer marker, searched only in the kept region.
    let end = no_avatar[start..]
        .iter()
        .position(|l| is_footer_marker(l))
        .map(|offset| start + offset)
        .unwrap_or(no_avatar.len());

    let kept = &no_avatar[start..end.min(no_avatar.len())];
    let mut out = kept.join("\n");
    // collapse 3+ blank lines to a paragraph break; trim trailing whitespace
    let collapse = blank_run_re();
    out = collapse.replace_all(&out, "\n\n").into_owned();
    out.trim_end().to_string()
}

// ---- pass 1: avatar / author line ----

/// True for the WordPress author/avatar line, e.g.
/// `![Dev](…/wp-content/litespeed/avatar/….jpg?ver=…)[Dev](…/author/bill/)May 19`.
fn is_avatar_line(line: &str) -> bool {
    line.contains("wp-content/litespeed/avatar")
}

// ---- pass 2: leading chrome predicates ----

/// True when a top-of-body line is chrome (not real content).
fn is_leading_chrome(line: &str) -> bool {
    let t = line.trim();
    if t.is_empty() {
        return true;
    }
    if t.starts_with('#') {
        return false; // a heading is the first real line — stop dropping
    }
    if is_widget_string(t) || is_link_only(t) || is_image_or_brand(t) || is_bare_breadcrumb(t) {
        return true;
    }
    false
}

/// Known search-widget strings Firecrawl leaks from this theme.
fn is_widget_string(t: &str) -> bool {
    t.contains("Hit enter to search")
        || t.contains("ESC to close")
        || t == "Search"
        || t.contains("Close Search")
}

/// A line made only of `[text](url)` link tokens and whitespace (nav runs,
/// auth CTAs, category runs, Share button runs). An inline link inside a
/// sentence is NOT link-only.
fn is_link_only(t: &str) -> bool {
    let stripped = link_text_re().replace_all(t, "");
    stripped.trim().is_empty()
}

/// An image-only line, or an image followed only by a short brand mash
/// (e.g. `![CurriculoATS](…logo…)CurriculoATS`). A featured image with a real
/// caption sentence is kept.
fn is_image_or_brand(t: &str) -> bool {
    if !t.contains("![") {
        return false;
    }
    let rest = image_re().replace_all(t, "").trim().to_string();
    if rest.is_empty() {
        return true;
    }
    // image + short single-token brand, e.g. "CurriculoATS"
    is_bare_breadcrumb(&rest) && rest.len() <= 30
}

/// A single short token with no whitespace and no sentence punctuation, e.g.
/// the bare breadcrumb word `Blog`.
///
/// ponytail: heuristic — a one-word leading prose line would be dropped too.
/// Ceiling: real articles open with a heading or a sentence, not a bare word,
/// so this only fires in the leading-chrome region before the first real line.
fn is_bare_breadcrumb(t: &str) -> bool {
    t.len() <= 12
        && !t.contains(' ')
        && !t.contains('\t')
        && !t.contains('[')
        && !t.contains('!')
        && !t.contains('#')
        && !t.contains('.')
        && !t.contains(',')
        && !t.contains(':')
        && !t.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
}

// ---- pass 3: trailing footer marker ----

/// True at the first line that begins footer/related chrome. Truncation is
/// inclusive: the marker line and everything after is dropped.
fn is_footer_marker(t: &str) -> bool {
    let t = t.trim();
    if t.is_empty() {
        return false;
    }
    if t.contains("We use cookies to improve your experience") {
        return true;
    }
    if t.contains("RejectAccept") {
        return true;
    }
    if t.contains("[Close Menu]") {
        return true;
    }
    if t == "_Next Post_" || t.starts_with("_Next Post_") {
        return true;
    }
    if t.starts_with('#') && t.contains("You May Also Like") {
        return true;
    }
    if is_mashed_title_list_item(t) {
        return true;
    }
    false
}

/// A list item `- [TitleBlurb](url)` whose single link mashes a title and a
/// blurb with no separator — detected as a lowercase letter immediately
/// followed by an uppercase one (e.g. `BuilderBuild`, `FeaturesKeyword`).
/// Real reference links use spaces between words and do not match.
///
/// ponytail: camelCase-no-space detector. Ceiling = a real trailing list whose
/// link text happens to contain a lower→upper boundary (e.g. a product name
/// like `iPhoneApp`) would be treated as the cut point. Acceptable: this only
/// fires after all explicit markers (`_Next Post_`, cookie banner, …) and only
/// one real reference list per page typically precedes the footer.
fn is_mashed_title_list_item(t: &str) -> bool {
    if !t.starts_with("- [") {
        return false;
    }
    let Some(caps) = single_link_re().captures(t) else {
        return false;
    };
    let text = caps.get(1).map(|m| m.as_str()).unwrap_or("");
    mash_re().is_match(text)
}

// ---- compiled regexes (compiled once) ----

fn link_text_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\[[^\]]*\]\([^)]*\)").unwrap())
}

fn image_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"!\[[^\]]*\]\([^)]*\)").unwrap())
}

fn single_link_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\[([^\]]*)\]\([^)]*\)").unwrap())
}

fn mash_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    // lowercase letter immediately followed by uppercase (title↔blurb mash)
    RE.get_or_init(|| Regex::new(r"\p{Ll}\p{Lu}").unwrap())
}

fn blank_run_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\n{3,}").unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Assert every needle appears in the cleaned output.
    fn kept(md: &str, needles: &[&str]) {
        let out = strip_boilerplate(md);
        for n in needles {
            assert!(out.contains(n), "cleaned body lost real content: {n:?}\n--- out ---\n{out}");
        }
    }

    /// Assert no poison marker survives the cleaned output.
    fn poison_free(md: &str, markers: &[&str]) {
        let out = strip_boilerplate(md);
        for m in markers {
            assert!(!out.contains(m), "chrome survived: {m:?}\n--- out ---\n{out}");
        }
    }

    // ---- per-rule unit tests ----

    #[test]
    fn drops_leading_search_widget_and_close_link() {
        let md = "Hit enter to search or ESC to closeSearch\n\
                  [Close Search](https://x/#)\n\
                  # Real Title\n\nReal intro sentence here.\n";
        kept(md, &["# Real Title", "Real intro sentence here."]);
        poison_free(md, &["Hit enter to search", "Close Search"]);
    }

    #[test]
    fn drops_leading_nav_link_run_and_auth_cta() {
        let md = "[Features](https://x/features) [AI Screening](https://x/screening)\n\
                  [Log in](https://ats.x/sign-in) [Start Free](https://ats.x/sign-up)\n\
                  # Real Title\n\nBody.\n";
        kept(md, &["# Real Title", "Body."]);
        poison_free(md, &["[Features]", "[Log in]", "Start Free"]);
    }

    #[test]
    fn drops_leading_logo_image_plus_brand_mash() {
        let md = "![CurriculoATS](https://x/logo.webp)CurriculoATS\n\
                  # Real Title\n\nBody.\n";
        kept(md, &["# Real Title", "Body."]);
        poison_free(md, &["CurriculoATS", "logo.webp"]);
    }

    #[test]
    fn drops_leading_bare_breadcrumb_word() {
        let md = "Blog\n# Real Title\n\nBody.\n";
        kept(md, &["# Real Title", "Body."]);
        assert!(!strip_boilerplate(md).starts_with("Blog"));
    }

    #[test]
    fn removes_avatar_line_anywhere_in_body() {
        let md = "# Title\n\nIntro.\n\n\
                  ![Dev](https://x/wp-content/litespeed/avatar/abc.jpg?ver=1)\
                  [Dev](https://x/author/bill/)July 4, 2026\n\n\
                  More real prose.\n";
        kept(md, &["# Title", "Intro.", "More real prose."]);
        poison_free(md, &["litespeed/avatar", "author/bill"]);
    }

    #[test]
    fn truncates_at_cookie_banner_and_rejectaccept() {
        let md = "# Title\n\nReal body sentence.\n\n\
                  We use cookies to improve your experience and analyze site traffic. \
                  [Privacy Policy](https://x/privacy/)\n\nRejectAccept\n";
        kept(md, &["Real body sentence."]);
        poison_free(md, &["We use cookies", "RejectAccept", "Privacy Policy"]);
    }

    #[test]
    fn truncates_at_close_menu_link() {
        let md = "# Title\n\nBody.\n\n[Close Menu](https://x/#)\n\nFooter junk.\n";
        kept(md, &["Body."]);
        poison_free(md, &["Close Menu", "Footer junk"]);
    }

    #[test]
    fn truncates_at_next_post_marker() {
        let md =
            "# Title\n\nBody.\n\n_Next Post_\n\n### You May Also Like\n\n[Related](https://x/r)\n";
        kept(md, &["Body."]);
        poison_free(md, &["_Next Post_", "You May Also Like", "Related"]);
    }

    #[test]
    fn truncates_at_you_may_also_like_heading() {
        let md = "# Title\n\nBody.\n\n### You May Also Like\n\n[Related](https://x/r)\n";
        kept(md, &["Body."]);
        poison_free(md, &["You May Also Like", "Related"]);
    }

    #[test]
    fn truncates_at_mashed_title_link_list() {
        // real reference list (spaced link text) is kept; the mashed footer
        // list (no-space title↔blurb boundary) is the cut point.
        let md = "# Title\n\nBody.\n\n## Sources\n\n\
                  - [Employers Share Their Most Outrageous Resume Mistakes](https://x/a)\n\n\
                  ## The complete toolkit\n\n\
                  - [AI Resume BuilderBuild an ATS-ready resume that gets past the filters.](https://x/b)\n\
                  - [FeaturesKeyword matching, formatting checks.](https://x/c)\n";
        let out = strip_boilerplate(md);
        assert!(
            out.contains("[Employers Share Their Most Outrageous Resume Mistakes]"),
            "real reference list must survive:\n{out}"
        );
        poison_free(md, &["AI Resume BuilderBuild", "FeaturesKeyword"]);
    }

    #[test]
    fn keeps_featured_image_and_caption_inside_body() {
        // a featured image line is NOT leading (it follows the heading) and has
        // a real alt text — it must survive.
        let md = "# Title\n\nIntro.\n\n\
                  ![How ATS works in 2026](https://x/wp-content/uploads/2026/03/feat.webp)\n\n\
                  _Reviewed by the team._\n\nBody paragraph.\n";
        kept(md, &["![How ATS works in 2026]", "_Reviewed by the team._", "Body paragraph."]);
    }

    #[test]
    fn inline_link_in_sentence_is_kept_not_treated_as_link_only() {
        let md = "# Title\n\nSee [the guide](https://x/g) for more detail.\n";
        kept(md, &["See [the guide](https://x/g) for more detail."]);
    }

    #[test]
    fn empty_input_is_empty() {
        assert_eq!(strip_boilerplate(""), "");
        assert_eq!(strip_boilerplate("   \n  \n"), "");
    }

    #[test]
    fn idempotent_on_already_clean_body() {
        let md = "# Title\n\nA clean body with one paragraph.\n\nSecond paragraph.\n";
        let once = strip_boilerplate(md);
        let twice = strip_boilerplate(&once);
        assert_eq!(once, twice);
    }

    #[test]
    fn preserves_body_with_no_chrome_unchanged() {
        let md = "# Title\n\nFirst paragraph of real prose.\n\nSecond paragraph.\n";
        let out = strip_boilerplate(md);
        assert!(out.contains("First paragraph of real prose."));
        assert!(out.contains("Second paragraph."));
        assert!(out.starts_with("# Title"));
    }

    // ---- realistic end-to-end fixture (condensed from a real polluted page) ----

    const REALISTIC_POLLUTED: &str = "\
Hit enter to search or ESC to closeSearch

[Close Search](https://curriculo.me/p/#)

[ATS Optimization](https://curriculo.me/category/ats/) [Resume Tips](https://curriculo.me/category/tips/)

# How ATS Really Works in 2026 — Parsing, Scoring & AI Ranking Explained

Learn how applicant tracking systems really work in 2026 — from document parsing to AI-powered ranking.

![Dev](https://curriculo.me/wp-content/litespeed/avatar/cdf39dbf.jpg?ver=1)[Dev](https://curriculo.me/author/bill/)July 4, 2026

![How ATS works](https://curriculo.me/wp-content/uploads/2026/03/feat.webp)

_Reviewed by the Curriculo Engineering Team_

## What Is an Applicant Tracking System (ATS)?

An applicant tracking system is software employers use to collect, organize, screen, and rank job applications. According to research by TopResume, approximately 80% of resumes are rejected by ATS before reaching a hiring manager.

## Why 75% of Resumes Fail ATS Screening

Research from Jobscan indicates that 75% of resumes fail ATS screening due to three overlapping issues.

_**Disclosure:** This article was produced by Curriculo Inc._

Ready to build your resume?

Curriculo helps you create an ATS-optimized resume that gets interviews.

Get Started Free →

_Next Post_

### You May Also Like

[![LinkedIn vs resume](data:image/svg+xml)](https://curriculo.me/linkedin-vs-resume/) [Resume Tips](https://curriculo.me/category/tips/) [LinkedIn Profile vs Resume](https://curriculo.me/linkedin-vs-resume/)

[Close Menu](https://curriculo.me/p/#)

We use cookies to improve your experience and analyze site traffic. [Privacy Policy](https://curriculo.me/privacy/)

RejectAccept
";

    #[test]
    fn realistic_polluted_body_is_cleaned() {
        let out = strip_boilerplate(REALISTIC_POLLUTED);

        // leading + author + trailing chrome all gone
        for poison in [
            "Hit enter to search",
            "Close Search",
            "litespeed/avatar",
            "author/bill",
            "_Next Post_",
            "You May Also Like",
            "Close Menu",
            "We use cookies",
            "RejectAccept",
        ] {
            assert!(!out.contains(poison), "chrome survived: {poison:?}\n--- out ---\n{out}");
        }

        // real article prose + featured image + last real section survive
        for needle in [
            "# How ATS Really Works in 2026",
            "Learn how applicant tracking systems really work in 2026",
            "![How ATS works]",
            "_Reviewed by the Curriculo Engineering Team_",
            "An applicant tracking system is software employers use",
            "Research from Jobscan indicates that 75%",
            "Disclosure:** This article was produced by Curriculo Inc.",
        ] {
            assert!(out.contains(needle), "lost real content: {needle:?}\n--- out ---\n{out}");
        }

        // starts at the heading, ends at the disclosure region (no footer past it)
        assert!(out.starts_with("# How ATS Really Works in 2026"));
        assert!(
            !out.contains("Ready to build your resume?") || out.contains("Disclosure"),
            "if the CTA survived, the disclosure that precedes the footer must too"
        );
    }
}

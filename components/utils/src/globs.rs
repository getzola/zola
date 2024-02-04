use libs::globset::{Glob, GlobSet, GlobSetBuilder};

use errors::{bail, Result};

pub fn build_ignore_glob_set(ignore: &Vec<String>, name: &str) -> Result<GlobSet> {
    // Convert the file glob strings into a compiled glob set matcher. We want to do this once,
    // at program initialization, rather than for every page, for example. We arrange for the
    // globset matcher to always exist (even though it has to be inside an Option at the
    // moment because of the TOML serializer); if the glob set is empty the `is_match` function
    // of the globber always returns false.
    let mut glob_set_builder = GlobSetBuilder::new();
    for pat in ignore {
        let glob = match Glob::new(pat) {
            Ok(g) => g,
            Err(e) => bail!("Invalid ignored_{} glob pattern: {}, error = {}", name, pat, e),
        };
        glob_set_builder.add(glob);
    }
    Ok(glob_set_builder.build()?)
}

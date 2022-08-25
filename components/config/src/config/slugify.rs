use serde::{Deserialize, Serialize};

use utils::slugs::SlugifyStrategy;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Slugify {
    pub paths: SlugifyStrategy,
    pub paths_keep_dates: bool,
    pub taxonomies: SlugifyStrategy,
    pub anchors: SlugifyStrategy,
}

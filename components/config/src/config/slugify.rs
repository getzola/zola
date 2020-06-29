use serde_derive::{Deserialize, Serialize};

use utils::slugs::SlugifyStrategy;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Slugify {
    pub paths: SlugifyStrategy,
    pub taxonomies: SlugifyStrategy,
    pub anchors: SlugifyStrategy,
}

impl Default for Slugify {
    fn default() -> Self {
        Slugify {
            paths: SlugifyStrategy::On,
            taxonomies: SlugifyStrategy::On,
            anchors: SlugifyStrategy::On,
        }
    }
}

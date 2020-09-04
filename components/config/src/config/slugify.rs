use serde_derive::{Deserialize, Serialize};

use utils::slugs::SlugifyStrategy;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Slugify {
    pub paths: SlugifyStrategy,
    pub taxonomies: SlugifyStrategy,
    pub anchors: SlugifyStrategy,
}

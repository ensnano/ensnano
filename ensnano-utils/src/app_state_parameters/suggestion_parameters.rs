use serde::{Deserialize, Serialize};

/// Parameters of strand suggestions.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct SuggestionParameters {
    pub include_scaffold: bool,
    pub include_intra_strand: bool,
    pub include_xover_ends: bool,
    pub ignore_groups: bool,
}

impl Default for SuggestionParameters {
    fn default() -> Self {
        Self {
            include_intra_strand: true,
            include_scaffold: true,
            include_xover_ends: false,
            ignore_groups: false,
        }
    }
}

impl SuggestionParameters {
    #[must_use]
    pub fn with_include_scaffold(&self, include_scaffold: bool) -> Self {
        let mut ret = *self;
        ret.include_scaffold = include_scaffold;
        ret
    }

    #[must_use]
    pub fn with_intra_strand(&self, intra_strand: bool) -> Self {
        let mut ret = *self;
        ret.include_intra_strand = intra_strand;
        ret
    }

    #[must_use]
    pub fn with_ignore_groups(&self, ignore_groups: bool) -> Self {
        let mut ret = *self;
        ret.ignore_groups = ignore_groups;
        ret
    }

    #[must_use]
    pub fn with_xover_ends(&self, include_xover_ends: bool) -> Self {
        let mut ret = *self;
        ret.include_xover_ends = include_xover_ends;
        ret
    }
}

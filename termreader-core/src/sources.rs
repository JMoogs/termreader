use termreader_sources::sources::{Source, SourceID, Sources};

#[derive(Clone)]
pub(super) struct SourceContext {
    sources: Sources,
}

impl SourceContext {
    pub(super) fn build() -> Self {
        Self {
            sources: Sources::build(),
        }
    }

    pub(super) fn get_source_by_id(&self, id: SourceID) -> Option<&Source> {
        self.sources.get_source_by_id(id)
    }

    pub(super) fn get_source_info(&self) -> Vec<(SourceID, String)> {
        self.sources.get_source_info()
    }
}

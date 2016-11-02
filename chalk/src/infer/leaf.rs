use lalrpop_intern::InternedString;
use std::sync::Arc;

use super::universe::UniverseIndex;
use super::var::InferenceVariable;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InferenceLeaf {
    data: Arc<InferenceLeafData>,
}

impl InferenceLeaf {
    pub fn new(data: InferenceLeafData) -> Self {
        InferenceLeaf { data: Arc::new(data) }
    }
}

deref_to!(InferenceLeaf.data => InferenceLeafData);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InferenceLeafData {
    pub kind: InferenceLeafKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InferenceLeafKind {
    Variable(InferenceVariable), // X
    Application(InferenceApplication), // C(...)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InferenceApplication {
    pub constant: InferenceConstant,
    pub args: Vec<InferenceLeaf>,
}

// C(...)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct InferenceConstant {
    pub operator: InternedString,
    pub universe_index: UniverseIndex,
}

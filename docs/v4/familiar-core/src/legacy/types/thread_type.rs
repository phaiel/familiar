use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum ThreadType {
    Person,
    Place,
    Concept,
    GenericObject,
}


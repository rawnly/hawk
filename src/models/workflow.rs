use serde::Deserialize;
use crate::models::files::*;

#[derive(Debug, Clone, Deserialize)]
pub struct Workflow {
    pub name: String,
}

impl File<Workflow> for Workflow {}


use crate::models::files::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Workflow {
    pub name: String,
}

impl File<Workflow> for Workflow {}

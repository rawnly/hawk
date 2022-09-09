use crate::models::files::File;
use crate::models::workflow::Workflow;
use std::fs;
use std::path::Path;

pub fn copy_file(source: &Path, target_dir: &str, scope: &str) -> std::io::Result<()> {
    let filename = target_filename(source, target_dir, scope);
    to_void_result(fs::copy(source, filename))
}

pub fn remove_file(source: &Path, target: &str, scope: &str) -> std::io::Result<()> {
    let filename = target_filename(source, target, scope);

    if Path::new(&filename).exists() {
        fs::remove_file(filename)?
    }

    Ok(())
}

pub fn target_filename(source: &Path, target: &str, scope: &str) -> String {
    let name = source.file_name().unwrap().to_str().unwrap();

    format!("{}/{}--{}", target, scope, name)
}

pub fn to_void_result<T>(r: std::io::Result<T>) -> std::io::Result<()> {
    match r {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

#[deprecated(since = "0.1.4", note = "Use is_workflow_file instead")]
pub fn is_yaml(path: &str) -> bool {
    path.ends_with("yml") || path.ends_with("yaml")
}

// checks if the given filepath is a valid workflow file
// path must be the entire filepath
pub fn is_workflow_file(filepath: &Path) -> bool {
    let is_yaml = match filepath.extension() {
        None => false,
        Some(ext) => ext.eq("yaml") || ext.eq("yml"),
    };

    if let Err(err) = Workflow::load(filepath) {
        panic!("[{}] An error has occurred: {}", filepath.display(), err);
    }

    is_yaml
}

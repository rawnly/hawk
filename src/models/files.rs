use std::{ fs, io, path::Path };
use serde::{de, Serialize};

pub type Result<T> = std::result::Result<T, FileError>;

#[derive(Debug, Clone)]
struct PackageJsonError;

#[derive(Debug, Clone)]
pub enum FileKind {
    JSON,
    YAML
}

impl FileKind {
    pub fn from_path(p: &Path) -> std::result::Result<FileKind, FileError> {
        match p.extension().unwrap().to_str().unwrap() {
            "json" => Ok(FileKind::JSON),
            "yml" | "yaml" => Ok(FileKind::YAML),
            _ => Err(FileError::UnsupportedExtension)
        }
    }
}


pub trait File<T> 
where T: de::DeserializeOwned 
{
    /// Reads file from filesystem. It must be json or yaml.
    fn load(path: &Path) -> Result<T> {
        let r = fs::File::open(path)?;
        let kind = FileKind::from_path(path)?;

        dbg!(path.display());

        match kind {
            FileKind::JSON => match serde_json::from_reader(r) {
                    Ok(d) => Ok(d),
                    Err(e) => Err(FileError::from(e))
                }
            FileKind::YAML => match serde_yaml::from_reader(r) {
                    Ok(d) => Ok(d),
                    Err(e) => Err(FileError::from(e))
                }
        }
    }

    fn write(&self, path: &Path) -> Result<()> 
    where Self: Serialize 
    {
        let r = fs::File::create(path)?;
        let kind = FileKind::from_path(path)?;

        match kind {
            FileKind::JSON => serde_json::to_writer_pretty(r, self)?,
            FileKind::YAML => serde_yaml::to_writer(r, self)?,
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum FileError {
    NotFound,
    UnsupportedExtension,
    InvalidYAMLSyntax(serde_yaml::Error),
    InvalidJSONSyntax(serde_json::Error),
    IO(io::Error)
}

impl From<serde_json::Error> for FileError {
    fn from(e: serde_json::Error) -> Self {
        FileError::InvalidJSONSyntax(e)
    }
}

impl From<serde_yaml::Error> for FileError {
    fn from(e: serde_yaml::Error) -> Self {
        FileError::InvalidYAMLSyntax(e)
    }
}

impl From<io::Error> for FileError {
    fn from(e: io::Error) -> Self {
       match e.kind() {
           io::ErrorKind::NotFound => FileError::NotFound,
           _ => FileError::IO(e)
       } 
    }
}

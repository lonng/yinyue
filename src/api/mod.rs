use std::fs::File;
use std::path::Path;

use failure::Fail;
use reqwest;

mod adapter;
pub mod crypto;

pub use adapter::{mp3_info, mv_info, parse_adapter, Adapter};

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "url parse failed: {}", _0)]
    UrlParseError(url::ParseError),
    #[fail(display = "io error: {}", _0)]
    Io(std::io::Error),
    #[fail(display = "invalid url: {}", _0)]
    InvalidUrl(String),
    #[fail(display = "adapter not found: {}", _0)]
    AdapterNotFound(String),
    #[fail(display = "encrypt failed")]
    Encrypt,
    #[fail(display = "request failed: {:?}", _0)]
    Reqwest(reqwest::Error),
    #[fail(display = "download error: {}", _0)]
    Download(String),
    #[fail(display = "invalid media type: {}", _0)]
    InvalidType(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<url::ParseError> for Error {
    fn from(e: url::ParseError) -> Self {
        Error::UrlParseError(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Artist {
    id: i32,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Album {
    id: i32,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Song {
    id: i32,
    mv: i32,
    name: String,
    ar: Vec<Artist>,
    al: Album,
}

impl Song {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn mv(&self) -> i32 {
        self.mv
    }

    pub fn artist(&self) -> String {
        self.ar
            .iter()
            .map(|ref x| x.name.clone())
            .collect::<Vec<String>>()
            .join(" & ")
    }

    pub fn file_name(&self, format: &str) -> String {
        format
            .replace("$name", self.name.as_str())
            .replace("$artist", self.artist().as_str())
            .replace("$album", self.al.name.as_str())
    }
}

impl ToString for Song {
    fn to_string(&self) -> String {
        format!(
            "ID: {}, Name: {}, Artist: {}, Album: {}",
            self.id,
            self.name.as_str(),
            self.artist().as_str(),
            self.al.name.as_str()
        )
    }
}

pub fn download(fileurl: String, filepath: &str) -> Result<()> {
    let path = Path::new(filepath);
    if path.exists() {
        return Err(Error::Download(format!("file exists: {}", filepath)));
    }

    let mut file = File::create(filepath)?;
    let mut remote = reqwest::get(&fileurl)?;

    std::io::copy(&mut remote, &mut file)?;
    Ok(())
}

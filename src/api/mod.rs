use std::fs::File;
use std::path::Path;

use failure::Fail;
use id3::{Tag, Version};
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
    #[fail(display = "empty response")]
    EmptyResponse,
    #[fail(display = "mv not found")]
    MvNotFound,
    #[fail(display = "serde json failed: {:?}", _0)]
    SerdeJson(serde_json::Error),
    #[fail(display = "metadata error: {:?}", _0)]
    MetadataError(id3::Error),
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

impl From<id3::Error> for Error {
    fn from(e: id3::Error) -> Self {
        Error::MetadataError(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::SerdeJson(e)
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

    pub fn file_name(&self, format: &str) -> String {
        format
            .replace("$name", self.name.as_str())
            .replace("$artist", self.joined_artist_names(" & ").as_str())
            .replace("$album", self.al.name.as_str())
    }

    pub fn joined_artist_names(&self, separator: &str) -> String {
        self.ar
            .iter()
            .map(|ref x| x.name.clone())
            .collect::<Vec<String>>()
            .join(separator)
    }
}

impl ToString for Song {
    fn to_string(&self) -> String {
        format!(
            "ID: {}, Name: {}, Artist: {}, Album: {}",
            self.id,
            self.name.as_str(),
            self.joined_artist_names(" & ").as_str(),
            self.al.name.as_str()
        )
    }
}

pub fn add_metadata(song: &Song, filepath: &str) -> Result<()> {
    let mut tag = Tag::new();
    tag.set_artist(song.joined_artist_names("/"));
    tag.set_title(song.name.clone());
    tag.set_album(song.al.name.clone());

    tag.write_to_path(filepath, Version::Id3v24)?;
    Ok(())
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

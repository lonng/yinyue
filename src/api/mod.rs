extern crate url;

pub struct Song {}

pub trait Adapter {
    fn song_list(&self) -> Result<Vec<Song>, APIError>;
}

pub enum APIError {
    UrlParseError,
    AdapterNotFound
}

pub fn parse_adapter(rawurl: &str) -> Result<Box<Adapter>, APIError> {
    let result = url::Url::parse(rawurl);
    if result.is_err() {
        return Err(APIError::UrlParseError)
    }

    let url_data = result.unwrap();
    let path = url_data.path();
    let slash_index = path.rfind("/").unwrap();
    let adapter_name = &path[(slash_index+1)..];

    match adapter_name {
        "song" => Ok(Box::new(SongAdapter{id: "1000".to_string()})),
        _ => Err(APIError::AdapterNotFound)
    }
}

struct SongAdapter {
    id: String
}

impl Adapter for SongAdapter {
    fn song_list(&self) -> Result<Vec<Song>, APIError> {
        unimplemented!()
    }
}

struct PlaylistAdapter {
    id: String
}

struct AlbumAdapter {
    id: String
}

struct CommonAdapter {
    id: String,
    url: String,
}
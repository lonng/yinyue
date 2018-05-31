extern crate url;

pub struct Song {}

pub trait Adapter {
    fn song_list(&self) -> Result<Vec<Song>, Error>;
}

pub enum Error {}

pub fn parse_adapter(rawurl: String) -> Result<Box<Adapter>, Error> {
    //let result = url::Url::parse(rawurl.as_str())?;
    Ok(Box::new(SongAdapter{id: "1000".to_string()}))
}

struct SongAdapter {
    id: String
}

impl Adapter for SongAdapter {
    fn song_list(&self) -> Result<Vec<Song>, Error> {
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
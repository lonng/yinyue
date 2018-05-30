pub struct Song {}

pub trait Adapter {
    fn song_list() -> Result<Vec<Song>, Error>;
}

pub struct Error {}

pub fn parse_adapter(rawurl: String) -> Result<Adapter, Error> {}

struct SongAdapter {
    id: String
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
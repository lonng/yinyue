use base64;
use rand::{Rng, thread_rng};
use regex::Regex;
use reqwest;
use reqwest::{Client, Url};
use reqwest::header;
use serde_json;
use std::collections::HashMap;

pub mod crypto;

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
    pub fn artist(&self) -> String {
        self.ar.iter().map(|ref x|x.name.clone()).collect::<Vec<String>>().join(" & ")
    }

    pub fn file_name(&self, format: &str) -> String {
        format.replace("$name", self.name.as_str())
            .replace("$artist", self.artist().as_str())
            .replace("$album", self.al.name.as_str())
    }
}

impl ToString for Song {
    fn to_string(&self) -> String {
        format!("ID: {}, Name: {}, Artist: {}, Album: {}",
                self.id,
                self.name.as_str(),
                self.artist().as_str(),
                self.al.name.as_str())
    }
}

pub trait Adapter {
    fn song_list(&self) -> Result<Vec<Song>, APIError>;
}

#[derive(Debug)]
pub enum APIError {
    UrlParseError,
    InvalidUrl,
    AdapterNotFound,
    HttpError(String),
}


static MODULUS: &'static str = "00e0b509f6259df8642dbc35662901477df22677ec152b5ff68ace615bb7b725152b3ab17a876aea8a5aa76d2e417629ec4ee341f56135fccf695280104e0312ecbda92557c93870114af6c9d05c4f7f0c3685b7a46bee255932575cce10b424d813cfe4875d3e82047b97ddef52741d546b8e289dc6935b3ece0462db0a22b8e7";
const NONCE: &'static str = "0CoJUm6Qyw8W8jud";
const PUB_KEY: &'static str = "010001";
const SECRET_KEY: &'static str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

fn create_secret_key(size: usize) -> String {
    let mut rng = thread_rng();
    let mut key = Vec::<u8>::new();
    for _ in 0..size {
        let index = rng.gen_range(0, SECRET_KEY.len());
        key.push(SECRET_KEY.as_bytes()[index]);
    }
    String::from_utf8(key).unwrap()
}

pub fn parse_adapter(rawurl: &str) -> Result<Box<Adapter>, APIError> {
    let repl = rawurl.replace("/#/", "/");
    let rawurl = repl.as_str();
    let result = Url::parse(rawurl);
    if result.is_err() {
        return Err(APIError::UrlParseError);
    }

    let url_data = result.unwrap();
    let path = url_data.path();

    let slash_index = path.rfind("/").unwrap();
    let adapter_name = &path[(slash_index + 1)..];
    let hash_query: HashMap<_, _> = url_data.query_pairs().into_owned().collect();

    let id = hash_query.get("id");
    if id.is_none() {
        return Err(APIError::InvalidUrl);
    }

    match adapter_name {
        "song" => Ok(Box::new(SongAdapter { id: id.unwrap().to_string() })),
        "playlist" => Ok(Box::new(PlaylistAdapter { id: id.unwrap().to_string() })),
        "album" => Ok(Box::new(AlbumAdapter { id: id.unwrap().to_string() })),
        "artist" => Ok(Box::new(CommonAdapter { id: id.unwrap().to_string(), url: "http://music.163.com/artist".to_string() })),
        "toplist" => Ok(Box::new(CommonAdapter { id: id.unwrap().to_string(), url: "http://music.163.com/discover/toplist".to_string() })),
        "djradio" => Ok(Box::new(CommonAdapter { id: id.unwrap().to_string(), url: "http://music.163.com/djradio".to_string() })),
        _ => Err(APIError::AdapterNotFound)
    }
}

fn header() -> header::Headers {
    let mut headers = header::Headers::new();
    headers.set(header::Host::new("music.163.com", None));
    headers.set(header::Origin::new("http", "music.163.com", None));
    headers.set(header::Referer::new("http://music.163.com/"));
    headers.set(header::UserAgent::new("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_10_5) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/45.0.2454.99 Safari/537.36"));
    headers
}

fn post(url: &str, payload: String) -> Result<String, reqwest::Error> {
    let client = Client::builder()
        .gzip(true)
        .default_headers(header()).build()?;

    let params1 = crypto::aes_encrypt(payload, NONCE);
    let key = create_secret_key(16);
    let params2 = crypto::aes_encrypt(base64::encode(&params1), key.as_str());
    let params = base64::encode(&params2);
    let enc_sec_key = crypto::encrypt(key.as_str(), PUB_KEY, MODULUS);

    let form = [
        ("params", params),
        ("encSecKey", enc_sec_key)
    ];

    let mut result = client.post(url).form(&form).send()?;
    result.text()
}

struct SongAdapter {
    id: String
}

impl Adapter for SongAdapter {
    fn song_list(&self) -> Result<Vec<Song>, APIError> {
        #[derive(Debug, Serialize, Deserialize)]
        struct Response {
            code: i32,
            songs: Vec<Song>,
            #[serde(skip_serializing_if = "Option::is_none")]
            msg: Option<String>,
        }

        let reqtext = json!({
            "c": format!("[{{id:{}}}]", self.id),
            "ids": vec![&self.id],
        });

        println!("{}", reqtext.to_string());

        let body = post("http://music.163.com/weapi/v3/song/detail?csrf_token=", reqtext.to_string()).unwrap();
        let resp: Response = serde_json::from_str(body.as_str()).unwrap();

        Ok(resp.songs)
    }
}

struct PlaylistAdapter {
    id: String
}

impl Adapter for PlaylistAdapter {
    fn song_list(&self) -> Result<Vec<Song>, APIError> {
        #[derive(Debug, Serialize, Deserialize)]
        struct PlayList {
            tracks: Vec<Song>
        }

        #[derive(Debug, Serialize, Deserialize)]
        struct Response {
            code: i32,
            playlist: PlayList,
            #[serde(skip_serializing_if = "Option::is_none")]
            msg: Option<String>,
        }

        let reqtext = json!({
            "csrf_token":"",
            "id": self.id,
            "n": 1000,
        });

        let body = post("http://music.163.com/weapi/v3/playlist/detail?csrf_token=", reqtext.to_string()).unwrap();
        let resp: Response = serde_json::from_str(body.as_str()).unwrap();

        Ok(resp.playlist.tracks)
    }
}

struct AlbumAdapter {
    id: String
}

impl Adapter for AlbumAdapter {
    fn song_list(&self) -> Result<Vec<Song>, APIError> {
        #[derive(Debug, Serialize, Deserialize)]
        struct Response {
            code: i32,
            songs: Vec<Song>,
            #[serde(skip_serializing_if = "Option::is_none")]
            msg: Option<String>,
        }

        let reqtext = json!({
            "csrf_token":"",
        });

        let url = format!("http://music.163.com/weapi/v1/album/{}?csrf_token=", self.id);
        let body = post(url.as_str(), reqtext.to_string()).unwrap();
        let resp: Response = serde_json::from_str(body.as_str()).unwrap();

        Ok(resp.songs)
    }
}

struct CommonAdapter {
    id: String,
    url: String,
}

impl Adapter for CommonAdapter {
    fn song_list(&self) -> Result<Vec<Song>, APIError> {
        let client = Client::builder()
            .gzip(true)
            .default_headers(header()).build().unwrap();

        let url = format!("{}?id={}", self.url, self.id);
        let mut response = client.get(url.as_str()).send().unwrap();
        let text = response.text().unwrap();
        let body = text.as_str();

        let matcher = Regex::new("href=\"/song\\?id=(?P<id>\\d*)\"").unwrap();
        let iter = matcher.captures_iter(body);
        let mut ids = Vec::<String>::new();
        for mat in iter {
            ids.push(mat["id"].to_string());
        }

        if ids.len() < 1 {
            return Ok(vec![]);
        }

        #[derive(Debug, Serialize, Deserialize)]
        struct Response {
            code: i32,
            songs: Vec<Song>,
            #[serde(skip_serializing_if = "Option::is_none")]
            msg: Option<String>,
        }

        let reqtext = json!({
            "ids": ids,
            "c": ids.into_iter().map(|x|format!("{{id:{}}}", x)).collect::<Vec<String>>(),
        });

        let body = post("http://music.163.com/weapi/v3/song/detail?csrf_token=", reqtext.to_string()).unwrap();
        let resp: Response = serde_json::from_str(body.as_str()).unwrap();

        Ok(resp.songs)
    }
}
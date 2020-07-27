use std::collections::HashMap;

use base64;
use rand::{thread_rng, Rng};
use regex::Regex;
use reqwest;
use reqwest::header;
use reqwest::{Client, Url};
use serde_json;

use super::Song;
use super::{Error, Result};
use crate::api::crypto;

pub trait Adapter {
    fn song_list(&self) -> Result<Vec<Song>>;
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
    // It's OK the unwrap here because of all characters are alphabetic and numeric
    String::from_utf8(key).unwrap()
}

/// Parses URL scheme and retrieve class and id for URL
///
/// http://music.163.com/playlist?id=892177597
///                      ^ class     ^ id
///
/// # Examples
///
/// ```
/// use yinyue::api::parse_adapter;
/// let x = parse_adapter("http://music.163.com/playlist?id=892177597");
/// assert_eq!(x, Ok(Box::new(PlaylistAdapter { id: "892177597".into() })));
///
/// let y = parse_adapter("http://music.163.com/#/album?id=38595209");
/// assert_eq!(y, Ok(Box::new(AlbumAdapter { id: "892177597".into() })));
/// ```
///
/// ```ignore
/// http://music.163.com/playlist?id=892177597
/// http://music.163.com/#/album?id=38595209
/// http://music.163.com/#/song?id=557584888
/// http://music.163.com/#/artist?id=10559
/// ```
pub fn parse_adapter(url: &str) -> Result<Box<dyn Adapter>> {
    let repl = url.replace("/#/", "/");
    let url_data = Url::parse(&repl)?;
    let path = url_data.path();

    // Retrieve class and id from URL
    let slash_index = match path.rfind("/") {
        Some(index) => index,
        None => return Err(Error::InvalidUrl(url.to_owned())),
    };
    let adapter_name = &path[(slash_index + 1)..];
    let queries = url_data
        .query_pairs()
        .into_owned()
        .collect::<HashMap<_, _>>();
    let id = match queries.get("id") {
        Some(id) => id.to_owned(),
        None => return Err(Error::InvalidUrl(url.to_owned())),
    };

    // Downgrade to `CommonAdapter` if there is not special adapter for this URL
    // The `CommonAdapter` use regexp to match song lists
    match adapter_name {
        "song" => Ok(Box::new(SongAdapter { id })),
        "playlist" => Ok(Box::new(PlaylistAdapter { id })),
        "album" => Ok(Box::new(AlbumAdapter { id })),
        "artist" => Ok(Box::new(CommonAdapter {
            id,
            url: "http://music.163.com/artist",
        })),
        "toplist" => Ok(Box::new(CommonAdapter {
            id,
            url: "http://music.163.com/discover/toplist",
        })),
        "djradio" => Ok(Box::new(CommonAdapter {
            id,
            url: "http://music.163.com/djradio",
        })),
        _ => Err(Error::AdapterNotFound(adapter_name.to_owned())),
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

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Reqwest(e)
    }
}

fn post(url: &str, payload: String) -> Result<String> {
    let client = Client::builder()
        .gzip(true)
        .default_headers(header())
        .build()?;

    let params1 = crypto::aes_encrypt(payload, NONCE);
    let key = create_secret_key(16);
    let params2 = crypto::aes_encrypt(base64::encode(&params1), key.as_str());
    let params = base64::encode(&params2);
    let enc_sec_key = match crypto::encrypt(key.as_str(), PUB_KEY, MODULUS) {
        Some(key) => key,
        None => return Err(Error::Encrypt),
    };

    let form = [("params", params), ("encSecKey", enc_sec_key)];

    let mut result = client.post(url).form(&form).send()?;
    let body = result.text()?;

    Ok(body)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MP3Info {
    url: String,
    size: i32,
}

pub fn mp3_info(id: i32, r: &str) -> Result<String> {
    #[derive(Debug, Serialize, Deserialize)]
    struct Response {
        code: i32,
        data: Vec<MP3Info>,
        #[serde(skip_serializing_if = "Option::is_none")]
        msg: Option<String>,
    }

    let req = json!({
        "br": r,
        "ids": format!("[{}]", id)
    });

    let body = post(
        "http://music.163.com/weapi/song/enhance/player/url?csrf_token=",
        req.to_string(),
    )?;

    let resp = serde_json::from_str::<'_, Response>(body.as_str())?;
    if resp.data.len() < 1 || resp.data[0].url.len() == 0 {
        return Err(Error::EmptyResponse);
    }
    Ok(resp.data[0].clone().url)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MVInfo {
    url: String,
    size: i32,
}

pub fn mv_info(id: i32, r: &str) -> Result<String> {
    if id == 0 {
        return Err(Error::MvNotFound);
    }
    #[derive(Debug, Serialize, Deserialize)]
    struct Response {
        code: i32,
        data: MVInfo,
        #[serde(skip_serializing_if = "Option::is_none")]
        msg: Option<String>,
    }

    let req = json!({
        "csrf_token": "",
        "r": r,
        "id": id
    });

    let body = post(
        "http://music.163.com/weapi/song/enhance/play/mv/url?csrf_token=",
        req.to_string(),
    )?;
    let resp = serde_json::from_str::<'_, Response>(body.as_str())?;
    if resp.data.url.len() == 0 {
        return Err(Error::EmptyResponse);
    }
    Ok(resp.data.url)
}

struct SongAdapter {
    id: String,
}

impl Adapter for SongAdapter {
    fn song_list(&self) -> Result<Vec<Song>> {
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

        let body = post(
            "http://music.163.com/weapi/v3/song/detail?csrf_token=",
            reqtext.to_string(),
        )
        .unwrap();
        let resp: Response = serde_json::from_str(body.as_str()).unwrap();

        Ok(resp.songs)
    }
}

struct PlaylistAdapter {
    id: String,
}

impl Adapter for PlaylistAdapter {
    fn song_list(&self) -> Result<Vec<Song>> {
        #[derive(Debug, Serialize, Deserialize)]
        struct PlayList {
            tracks: Vec<Song>,
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

        let body = post(
            "http://music.163.com/weapi/v3/playlist/detail?csrf_token=",
            reqtext.to_string(),
        )
        .unwrap();
        let resp: Response = serde_json::from_str(body.as_str()).unwrap();

        Ok(resp.playlist.tracks)
    }
}

struct AlbumAdapter {
    id: String,
}

impl Adapter for AlbumAdapter {
    fn song_list(&self) -> Result<Vec<Song>> {
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

        let url = format!(
            "http://music.163.com/weapi/v1/album/{}?csrf_token=",
            self.id
        );
        let body = post(url.as_str(), reqtext.to_string()).unwrap();
        let resp: Response = serde_json::from_str(body.as_str()).unwrap();

        Ok(resp.songs)
    }
}

struct CommonAdapter {
    id: String,
    url: &'static str,
}

impl Adapter for CommonAdapter {
    fn song_list(&self) -> Result<Vec<Song>> {
        let client = Client::builder()
            .gzip(true)
            .default_headers(header())
            .build()
            .unwrap();

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

        let body = post(
            "http://music.163.com/weapi/v3/song/detail?csrf_token=",
            reqtext.to_string(),
        )
        .unwrap();
        let resp: Response = serde_json::from_str(body.as_str()).unwrap();

        Ok(resp.songs)
    }
}

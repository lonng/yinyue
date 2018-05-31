extern crate yinyue;

use yinyue::api;

fn main() {
    let adapter = api::parse_adapter("http://music.163.com/playlist?id=892177597").unwrap();
    let song_list = adapter.song_list().unwrap();
    println!("{:#?}", song_list)
}

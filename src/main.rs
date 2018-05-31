extern crate yinyue;

use yinyue::api;

fn main() {
    //let adapter = api::parse_adapter("http://music.163.com/playlist?id=892177597").unwrap();
//    let adapter = api::parse_adapter("http://music.163.com/#/album?id=38595209").unwrap();
//    let adapter = api::parse_adapter("http://music.163.com/#/song?id=557584888").unwrap();
    let adapter = api::parse_adapter("http://music.163.com/#/artist?id=10559").unwrap();
    let song_list = adapter.song_list().unwrap();
    println!("{:#?}", song_list)
}

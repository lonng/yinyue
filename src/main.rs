extern crate yinyue;
extern crate getopts;

use yinyue::api;
use getopts::Options;
use std::env;
use std::process;
use std::path::Path;
use std::fs;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] url", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    //let adapter = api::parse_adapter("http://music.163.com/playlist?id=892177597").unwrap();
    //let adapter = api::parse_adapter("http://music.163.com/#/album?id=38595209").unwrap();
    //let adapter = api::parse_adapter("http://music.163.com/#/song?id=557584888").unwrap();
    //let adapter = api::parse_adapter("http://music.163.com/#/artist?id=10559").unwrap();

    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("t", "type", "mp3 or mv", "");
    opts.optopt("f", "format", "filename format($name, $artist, $album)", "");
    opts.optopt("q", "quality", "quality(480/720/1080 for mv, 12800/19200/32000 for music)", "");
    opts.optopt("d", "dir", "save to target directory", "");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(_) => {
            print_usage(program.as_str(), opts);
            process::exit(-1);
        }
    };

    if matches.free.is_empty() {
        print_usage(program.as_str(), opts);
        return;
    }

    let url = matches.free[0].clone();
    let dir = matches.opt_str("dir")
        .or(Some("music".to_string()))
        .unwrap();

    let fmt = matches.opt_str("format")
        .or(Some("$artist - $name".to_string()))
        .unwrap();

    let typ = matches.opt_str("type")
        .or(Some("mp3".to_string()))
        .unwrap();

    let qua = match typ.as_str() {
        "mp3" => {
            matches.opt_str("quality")
                .or(Some("19200".to_string()))
                .unwrap()
        }

        "mv" => {
            matches.opt_str("quality")
                .or(Some("480".to_string()))
                .unwrap()
        }

        _ => unreachable!()
    };

    println!("Output directory: {}", dir);
    println!("File name format: {}", fmt);
    println!("Media type: {}", typ);
    println!("Media quality: {}", qua);

    if !Path::new(dir.as_str()).exists() {
        fs::create_dir_all(&dir).unwrap();
    }

    println!("Starting fetch song list from: {}", url);
    let adapter = api::parse_adapter(url.as_str()).unwrap();
    let song_list = adapter.song_list().unwrap();

    println!("Fetching song list completed, total amount: {}", song_list.len());

    for song in song_list {
        println!("Parse song download info: {}", song.to_string());
    }

    println!("Download complete, target directory: {}", dir);
}

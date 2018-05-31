extern crate yinyue;
extern crate getopts;
extern crate reqwest;

use yinyue::api;
use getopts::Options;
use std::env;
use std::process;
use std::path::Path;
use std::fs;
use std::fs::File;

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
    let total = song_list.len();
    println!("Fetching song list completed, total amount: {}", total);

    let target_dir = Path::new(&dir);
    for (i, song) in song_list.iter().enumerate() {
        println!("Parse song download info: {}", song.to_string());
        let download_url = match typ.as_str() {
            "mp3" => api::mp3_info(song.id(), qua.as_str()),
            "mv" => {
                let mv = song.mv();
                if mv > 0 {
                    api::mv_info(song.mv(), qua.as_str())
                }else {
                    None
                }
            },
            _ => unreachable!()
        };

        if download_url.is_none() {
            continue;
        }

        let fileurl = download_url.unwrap();
        let extension = match fileurl.rfind(".") {
            None => {
                match typ.as_str() {
                    "mp3" => "mp3".to_string(),
                    "mv" => "mp4".to_string(),
                    _ => unreachable!()
                }
            },
            Some(index) => fileurl[index..].to_string()
        };
        let filename = format!("{}{}", song.file_name(&fmt),extension);
        println!("Downloading: [{}/{}]{}", i+1, total, filename);
        download(fileurl, target_dir.join(filename).to_str().unwrap());
    }

    println!("Download complete, target directory: {}", dir);
}

fn download(fileurl: String, filepath: &str) {
    let path = Path::new(filepath);
    if path.exists() {
        println!("File exists: {}", filepath);
        return;
    }

    let mut file = File::create(filepath);
    if file.is_err() {
        println!("Create file failed: {:?}", file.err());
        return;
    }

    let mut remote = reqwest::get(&fileurl);
    if remote.is_err() {
        println!("Send request failed: {}", fileurl);
        return;
    }

    let result = std::io::copy(&mut remote.unwrap(), &mut file.unwrap());
    if result.is_err() {
        println!("Save file failed: {:?}", result.err());
        return;
    }
}

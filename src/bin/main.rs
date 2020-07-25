#[macro_use]
extern crate log;

use getopts::Matches;
use getopts::Options;
use std::env;
use std::fs;
use std::path::Path;
use std::process;

use simple_logger;

use yinyue::api::{self, Result};

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] url", program);
    println!("{}", opts.usage(&brief));
}

fn main() {
    // Initialize the logger
    simple_logger::init_with_level(log::Level::Info).unwrap();

    // Parse the command line arguments
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("t", "type", "mp3 or mv", "");
    opts.optopt("f", "format", "filename format($name, $artist, $album)", "");
    opts.optopt(
        "q",
        "quality",
        "quality(480/720/1080 for mv, 12800/19200/32000 for music)",
        "",
    );
    opts.optopt("d", "dir", "save to target directory", "");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(_) => {
            print_usage(program.as_str(), opts);
            process::exit(-1);
        }
    };

    if matches.free.is_empty() {
        print_usage(program.as_str(), opts);
        return;
    }

    if let Err(err) = run(&matches) {
        info!("Error occurred while download: {:?}", err);
    }
}

fn run(matches: &Matches) -> Result<()> {
    // Parse command line arguments
    let dir = matches.opt_str("dir").unwrap_or("music".into());
    let fmt = matches
        .opt_str("format")
        .unwrap_or("$artist - $name".into());

    // Only `mp3` and `mv` are acceptable types
    let typ = matches.opt_str("type").unwrap_or("mp3".into());
    if typ != "mp3" && typ != "mv" {
        return Err(api::Error::InvalidType(typ));
    }

    let qua = match typ.as_ref() {
        "mp3" => matches.opt_str("quality").unwrap_or("19200".into()),
        "mv" => matches.opt_str("quality").unwrap_or("480".into()),
        _ => unreachable!(),
    };

    info!("Output directory: {}", dir);
    info!("File name format: {}", fmt);
    info!("Media type: {}", typ);
    info!("Media quality: {}", qua);

    // Create target directory if it is not exists
    if !Path::new(dir.as_str()).exists() {
        fs::create_dir_all(&dir)?;
    }

    info!("Starting fetch song list from: {}", matches.free[0]);
    let adapter = api::parse_adapter(&matches.free[0])?;
    let song_list = adapter.song_list()?;
    let total = song_list.len();

    info!("Fetching song list completed, total amount: {}", total);
    let target_dir = Path::new(&dir);
    for (i, song) in song_list.iter().enumerate() {
        info!("Parse song download info: {}", song.to_string());
        let download_url = match typ.as_str() {
            "mp3" => api::mp3_info(song.id(), qua.as_str()),
            "mv" => api::mv_info(song.mv(), qua.as_str()),
            _ => unreachable!(),
        };

        let download_url = match download_url {
            Ok(url) => url,
            Err(err) => {
                error!("Parse download URL failed: {}", err);
                continue;
            }
        };
        let extension = match download_url.rfind(".") {
            None => typ.as_str(),
            Some(index) => {
                // truncate query parameters in URL if any
                if let Some(i) = download_url.rfind("?") {
                    &download_url[index..i]
                } else {
                    &download_url[index..]
                }
            },
        };
        let filename = format!("{}{}", song.file_name(&fmt), extension);
        info!("Downloading: [{}/{}]{}]", i + 1, total, filename);
        match api::download(download_url, target_dir.join(filename).to_str().unwrap()) {
            Ok(_) => {}
            Err(err) => error!("Download file failed: {:?}", err),
        }
    }
    info!("Download complete, target directory: {}", dir);
    Ok(())
}

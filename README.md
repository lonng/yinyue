# yinyue

[![Codacy Badge](https://api.codacy.com/project/badge/Grade/144f265f634b4015bbe5bc7f03233b03)](https://app.codacy.com/app/lonng/yinyue?utm_source=github.com&utm_medium=referral&utm_content=lonng/yinyue&utm_campaign=Badge_Grade_Dashboard)

网易云音乐批量下载，支持下载mp3和mv，支持歌单，专辑，歌手热门，电台，单曲，排行榜

```text
http://music.163.com/playlist?id=892177597
http://music.163.com/#/album?id=38595209"
http://music.163.com/#/song?id=557584888
http://music.163.com/#/artist?id=10559"
http://music.163.com/#/discover/toplist?id=3779629
http://music.163.com/#/djradio?id=527162580
```

## Build

```bash
cargo build --release
```

## Usage
```text
Usage: target/release/yinyue [options] url

Options:
    -t, --type          mp3 or mv
    -f, --format        filename format($name, $artist, $album)
    -q, --quality       quality(480/720/1080 for mv, 12800/19200/32000 for
                        music)
    -d, --dir           save to target directory
```

### 参数解释
```
    -t, --type
      需要下载的类型，可选mp3或者mv
      
    -f, --format
      保存文件的格式，$name: 歌名, $artist: 歌手名, $album: 专辑名)，默认为$artist - $name
      
    -q, --quality
      下载多媒体质量，音频可选: 12800/19200/32000(单位比特率), 视频可选480/720/1080(P)
      
    -d, --dir
      文件保存路径，如果路径不存在，会自动创建，默认为music
```

## Example

```shell
yinyue -t mv http://music.163.com/playlist?id=892177597
```

Support for:
```text
http://music.163.com/song?id=$reource_id
http://music.163.com/playlist?id=$reource_id
http://music.163.com/album?id=$reource_id
http://music.163.com/artist?id=$reource_id
http://music.163.com/toplist?id=$reource_id
http://music.163.com/djradio?id=$reource_id
```

#![allow(unused)]
use serde_json::Value;
use std::{cmp::Ordering, path::Path, process::Command};

#[derive(Eq, PartialEq, Debug, PartialOrd, Ord)]
enum VideoQuality {
    HD2160,
    HD1440,
    HD1080,
    HD720,
    Large,
    Medium,
    Small,
    Tiny,
}

#[derive(Eq, PartialEq, Debug, PartialOrd, Ord)]
enum AudioQuality {
    Medium,
    Low,
}

#[derive(Eq, PartialEq, Debug, PartialOrd, Ord)]
enum VideoCodec {
    //.MP4
    Av01,
    Avc1,

    //.WEBM
    Vp9,
}

#[derive(Eq, PartialEq, Debug, PartialOrd, Ord)]
enum AudioCodec {
    //.weba
    Opus,
    //.3gp
    Mp4a,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Video {
    pub codec: VideoCodec,
    pub quality: VideoQuality,
    pub url: String,
}

impl Video {
    pub fn path(&self) -> &'static str {
        match self.codec {
            VideoCodec::Av01 | VideoCodec::Avc1 => "video.mp4",
            VideoCodec::Vp9 => "video.webm",
        }
    }
}

//Low Opus - Low Quality
//Low Opus - High Quality
//Low Mp4a - Low Quality
//Keep in mind not all opus files of the same quality are the same.
//This hopefully won't be a problem when trying to download the best
//quality.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Audio {
    pub quality: AudioQuality,
    pub codec: AudioCodec,
    pub url: String,
}

impl Audio {
    pub fn path(&self) -> &'static str {
        match self.codec {
            AudioCodec::Opus => "audio.weba",
            AudioCodec::Mp4a => "audio.3gp",
        }
    }
}

fn download(url: &str, file: impl AsRef<Path>) {
    let res = minreq::get(url)
        .with_max_status_line_length(10_000_000)
        .send()
        .unwrap();

    std::fs::write(file, res.as_bytes()).unwrap();
}

fn main() {
    // let (audio, video) = get_urls("n4Ft4WDA3oU");
    let (audio, video) = get_urls("N5kd-JIVCgg");

    std::thread::scope(|s| {
        s.spawn(|| download(&audio.url, audio.path()));
        s.spawn(|| download(&video.url, video.path()));
    });

    //Combine audio and video.
    Command::new("ffmpeg")
        .args([
            "-i",
            video.path(),
            "-i",
            audio.path(),
            "-c",
            "copy",
            //TODO: Get video title
            "output.mkv",
        ])
        .output()
        .unwrap();
}

fn get_urls(id: &str) -> (Audio, Video) {
    let url = "https://youtubei.googleapis.com/youtubei/v1/player?key=AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8";
    let json = format!(
        "{{
        \"videoId\": \"{id}\",
        \"context\": {{
            \"client\": {{
                \"clientName\": \"ANDROID\",
                \"clientVersion\": \"16.02\"
            }}
        }}
    }}"
    );

    let response = minreq::post(url).with_body(json).send().unwrap();
    let str = response.as_str().unwrap();
    let out: Value = serde_json::from_str(&str).unwrap();
    let data = &out["streamingData"];
    let formats = &data["formats"].as_array().unwrap();
    let adaptive_formats = &data["adaptiveFormats"].as_array().unwrap();

    let mut videos = Vec::new();
    let mut audios = Vec::new();

    for tag in formats.iter().chain(adaptive_formats.iter()) {
        let video_quality = &tag["quality"].as_str().unwrap();
        let audio_quality = &tag["audioQuality"].as_str().unwrap_or("empty");
        let mime_type = &tag["mimeType"].as_str().unwrap();
        let url = &tag["url"].as_str().unwrap();

        let video_quality = match *video_quality {
            "tiny" => VideoQuality::Tiny,
            "small" => VideoQuality::Small,
            "medium" => VideoQuality::Medium,
            "large" => VideoQuality::Large,
            "hd720" => VideoQuality::HD720,
            "hd1080" => VideoQuality::HD1080,
            "hd1440" => VideoQuality::HD1440,
            "hd2160" => VideoQuality::HD2160,
            _ => panic!("{}", video_quality),
        };

        let audio_quality = match *audio_quality {
            "AUDIO_QUALITY_LOW" => Some(AudioQuality::Low),
            "AUDIO_QUALITY_MEDIUM" => Some(AudioQuality::Medium),
            "empty" => None,
            _ => panic!("{}", audio_quality),
        };

        let (av, codec) = mime_type.split_once("; ").unwrap();

        let (first, _) = if codec.contains(",") {
            let (first, audio) = codec.split_once(",").unwrap();
            (first, Some(audio))
        } else {
            (codec, None)
        };

        let video_codec = |input: &str| -> VideoCodec {
            if input.contains("avc1") {
                VideoCodec::Avc1
            } else if input.contains("av01") {
                VideoCodec::Av01
            } else if input.contains("vp9") {
                VideoCodec::Vp9
            } else {
                unreachable!("{}", input)
            }
        };

        let audio_codec = |input: &str| -> AudioCodec {
            if input.contains("mp4a") {
                AudioCodec::Mp4a
            } else if input.contains("opus") {
                AudioCodec::Opus
            } else {
                unreachable!("{}", input)
            }
        };

        if av.starts_with("video/") && audio_quality.is_none() {
            let video = Video {
                codec: video_codec(first),
                quality: video_quality,
                url: url.to_string(),
            };
            videos.push(video);
        } else if av.starts_with("audio/") {
            let audio = Audio {
                codec: audio_codec(first),
                quality: audio_quality.unwrap(),
                url: url.to_string(),
            };
            audios.push(audio);
        }
    }

    audios.sort();
    audios.sort_by(|a, b| {
        if a.quality == b.quality {
            Ordering::Equal
        } else {
            a.quality.cmp(&b.quality)
        }
    });

    videos.sort();
    videos.sort_by(|a, b| {
        if a.quality == b.quality {
            Ordering::Equal
        } else {
            a.quality.cmp(&b.quality)
        }
    });

    (audios.remove(0), videos.remove(0))
}

#![allow(unused)]
use serde_json::Value;
use std::{fmt::Display, fs};

#[derive(Eq, PartialEq, Debug)]
enum Format {
    Video,
    Audio,
}

#[derive(Eq, PartialEq, Debug)]
enum Quality {
    Tiny,
    Small,
    Medium,
    Large,
    HD720,
    HD1080,
    HD1440,
    HD2160,
}

#[derive(Eq, PartialEq, Debug)]
enum Codec {
    Vp9,
    Mp4v,
    Mp4a,
    Av01,
    Avc1,
    Opus,
}

#[derive(Eq, PartialEq, Debug)]
enum AudioQuality {
    Low,
    Medium,
}

#[derive(Debug)]
struct Video {
    pub quality: Quality,
    pub audio_quality: Option<AudioQuality>,
    pub format: Format,
    pub codecs: (Codec, Option<Codec>),
    pub url: String,
}
impl Display for Video {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "{:?} {:?} {:?}",
            self.quality, self.audio_quality, self.codecs
        ))
    }
}

fn main() {
    let url = get_url("N5kd-JIVCgg");
    // let url = get_url("dCkZMZuqnuI");

    match url {
        Ok(url) => {
            let res = minreq::get(url)
                .with_max_status_line_length(10_000_000)
                .send()
                .unwrap();

            fs::write("video.webm", res.as_bytes());
        }
        Err(_) => return,
    }
}

fn get_url(id: &str) -> Result<String, ()> {
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

    for tag in formats.iter().chain(adaptive_formats.iter()) {
        let quality = &tag["quality"].as_str().unwrap();
        let audio_quality = &tag["audioQuality"].as_str().unwrap_or("empty");
        let mime_type = &tag["mimeType"].as_str().unwrap();
        let url = &tag["url"].as_str().unwrap();

        let quality = match *quality {
            "tiny" => Quality::Tiny,
            "small" => Quality::Small,
            "medium" => Quality::Medium,
            "large" => Quality::Large,
            "hd720" => Quality::HD720,
            "hd1080" => Quality::HD1080,
            "hd1440" => Quality::HD1440,
            "hd2160" => Quality::HD2160,
            _ => panic!("{}", quality),
        };

        let audio_quality = match *audio_quality {
            "AUDIO_QUALITY_LOW" => Some(AudioQuality::Low),
            "AUDIO_QUALITY_MEDIUM" => Some(AudioQuality::Medium),
            "empty" => None,
            _ => panic!("{}", audio_quality),
        };

        let (av, codec) = mime_type.split_once("; ").unwrap();

        let (first, second) = if codec.contains(",") {
            let (first, second) = codec.split_once(",").unwrap();
            (first, Some(second))
        } else {
            (codec, None)
        };

        let format = if av.starts_with("video/") {
            Format::Video
        } else if av.starts_with("audio/") {
            Format::Audio
        } else {
            unreachable!();
        };

        let codec = |input: &str| -> Option<Codec> {
            if input.contains("vp9") {
                Some(Codec::Vp9)
            } else if input.contains("av01") {
                Some(Codec::Av01)
            } else if input.contains("avc1") {
                Some(Codec::Avc1)
            } else if input.contains("mp4a") {
                Some(Codec::Mp4a)
            } else if input.contains("mp4v") {
                Some(Codec::Mp4v)
            } else if input.contains("opus") {
                Some(Codec::Opus)
            } else {
                unreachable!()
            }
        };

        let codecs = match format {
            Format::Video => {
                let second = if let Some(second) = second {
                    codec(second)
                } else {
                    None
                };

                (codec(first).unwrap(), second)
            }
            Format::Audio => (codec(first).unwrap(), None),
        };

        let video = Video {
            quality,
            audio_quality,
            format,
            codecs,
            url: url.to_string(),
        };

        videos.push(video);
    }

    for video in videos {
        println!("{}", video);
    }

    Err(())
}

#![allow(unused)]
use serde_json::Value;
use std::{fmt::Display, fs};

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
enum Codec {}

#[derive(Eq, PartialEq, Debug)]
enum AudioQuality {
    Low,
    Medium,
}

#[derive(Debug)]
struct Video {
    pub quality: Quality,
    pub audio_quality: Option<AudioQuality>,
    // pub codec: Codec,
    pub url: String,
}
impl Display for Video {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "{:?} {:?} {}",
            self.quality, self.audio_quality, self.url
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
        let mime_type = &tag["mimeType"];
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

        //TODO: Parse codec.

        let video = Video {
            quality,
            audio_quality,
            url: url.to_string(),
        };

        videos.push(video);
    }

    for video in videos {
        println!("{}", video);
    }

    Err(())
}

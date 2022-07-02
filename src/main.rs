#![allow(unused)]
use serde_json::Value;
use std::fs;

#[derive(Eq, PartialEq)]
enum Quality {
    Tiny,
    Small,
    Medium,
    Large,
}

fn main() {
    let url = get_url("");

    let res = minreq::get(url)
        .with_max_status_line_length(10_000_000)
        .send()
        .unwrap();

    fs::write("video.webm", res.as_bytes());
}

fn get_url(_id: &str) -> String {
    let url = "https://youtubei.googleapis.com/youtubei/v1/player?key=AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8";
    let json = r#"{
        "videoId": "wLkfz10Srxw",
        "context": {
            "client": {
                "clientName": "ANDROID",
                "clientVersion": "16.02"
            }
        }
    }"#;

    let response = minreq::post(url).with_body(json).send().unwrap();
    let str = response.as_str().unwrap();
    let out: Value = serde_json::from_str(&str).unwrap();
    let data = &out["streamingData"];
    let formats = &data["formats"].as_array().unwrap();
    let adaptive_formats = &data["adaptiveFormats"].as_array().unwrap();

    let target_quality = Quality::Medium;

    for tag in formats.iter() {
        let quality = &tag["quality"].as_str().unwrap();
        let audio_quality = &tag["audioQuality"];
        let mime_type = &tag["mimeType"];
        let url = &tag["url"];

        let q = match *quality {
            "tiny" => Quality::Tiny,
            "small" => Quality::Small,
            "medium" => Quality::Medium,
            "large" => Quality::Large,
            _ => unreachable!(),
        };

        if q == target_quality {
            println!("{}\n{}\n{}\n", quality, audio_quality, mime_type);
            return url.as_str().unwrap().to_string();
        }
    }

    unreachable!()
}

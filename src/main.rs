#![allow(unused)]
use serde_json::Value;
use std::cmp::Ordering;

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
enum VideoOnlyCodec {
    //.MP4
    Av01,
    Avc1,

    //.WEBM
    Vp9,
}

#[derive(Eq, PartialEq, Debug, PartialOrd, Ord)]
enum VideoCodec {
    //.MP4
    Avc1,

    //.3gp
    Mp4v,
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
    pub video_codec: VideoCodec,
    pub video_quality: VideoQuality,
    pub audio_quality: AudioQuality,
    pub audio_codec: AudioCodec,
    pub url: String,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct VideoOnly {
    pub codec: VideoOnlyCodec,
    pub quality: VideoQuality,
    pub url: String,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct AudioOnly {
    pub audio_quality: AudioQuality,
    pub audio_codec: AudioCodec,
    pub url: String,
}

fn main() {
    let url = get_url("N5kd-JIVCgg");
    // let url = get_url("n4Ft4WDA3oU");

    match url {
        Ok(url) => {
            // let res = minreq::get(url)
            //     .with_max_status_line_length(10_000_000)
            //     .send()
            //     .unwrap();

            // fs::write("video.webm", res.as_bytes());
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
    let mut videos_only = Vec::new();
    let mut audios_only = Vec::new();

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

        let (first, audio) = if codec.contains(",") {
            let (first, audio) = codec.split_once(",").unwrap();
            (first, Some(audio))
        } else {
            (codec, None)
        };

        let video_only_codec = |input: &str| -> VideoOnlyCodec {
            if input.contains("avc1") {
                VideoOnlyCodec::Avc1
            } else if input.contains("av01") {
                VideoOnlyCodec::Av01
            } else if input.contains("vp9") {
                VideoOnlyCodec::Vp9
            } else {
                unreachable!("{}", input)
            }
        };

        let video_codec = |input: &str| -> VideoCodec {
            if input.contains("avc1") {
                VideoCodec::Avc1
            } else if input.contains("mp4v") {
                VideoCodec::Mp4v
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

        if av.starts_with("video/") {
            if let Some(audio_quality) = audio_quality {
                let video = Video {
                    video_codec: video_codec(first),
                    video_quality,
                    audio_quality,
                    url: url.to_string(),
                    audio_codec: audio_codec(audio.unwrap()),
                };
                videos.push(video);
            } else {
                let video = VideoOnly {
                    codec: video_only_codec(first),
                    quality: video_quality,
                    url: url.to_string(),
                };
                videos_only.push(video);
            }
        } else if av.starts_with("audio/") {
            let audio = AudioOnly {
                audio_codec: audio_codec(first),
                audio_quality: audio_quality.unwrap(),
                url: url.to_string(),
            };
            audios_only.push(audio);
        } else {
            unreachable!();
        };
    }

    videos_only.sort();
    videos_only.sort_by(|a, b| {
        if a.quality == b.quality {
            Ordering::Equal
        } else {
            a.quality.cmp(&b.quality)
        }
    });

    for video in videos_only {
        println!("{:?} {:?}\n{}", video.quality, video.codec, video.url);
    }

    Err(())
}

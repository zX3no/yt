#![allow(unused)]
use serde_json::Value;

#[derive(Eq, PartialEq, Debug)]
enum VideoQuality {
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
enum AudioQuality {
    Low,
    Medium,
}

#[derive(Eq, PartialEq, Debug)]
enum VideoCodec {
    Vp9,
    Mp4v,
    Mp4a,
    Av01,
    Avc1,
}

#[derive(Eq, PartialEq, Debug)]
enum AudioCodec {
    Opus,
    Mp4a,
}

#[derive(Debug)]
struct Video {
    pub video_codec: VideoCodec,
    pub video_quality: VideoQuality,
    pub audio_quality: AudioQuality,
    pub audio_codec: AudioCodec,
    pub url: String,
}

#[derive(Debug)]
struct VideoOnly {
    pub video_codec: VideoCodec,
    pub quality: VideoQuality,
    pub url: String,
}

#[derive(Debug)]
struct AudioOnly {
    pub audio_quality: AudioQuality,
    pub audio_codec: AudioCodec,
    pub url: String,
}

fn main() {
    let url = get_url("N5kd-JIVCgg");
    // let url = get_url("dCkZMZuqnuI");

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

        let video_codec = |input: &str| -> VideoCodec {
            if input.contains("vp9") {
                VideoCodec::Vp9
            } else if input.contains("av01") {
                VideoCodec::Av01
            } else if input.contains("avc1") {
                VideoCodec::Avc1
            } else if input.contains("mp4a") {
                VideoCodec::Mp4a
            } else if input.contains("mp4v") {
                VideoCodec::Mp4v
            } else {
                unreachable!()
            }
        };

        let audio_codec = |input: &str| -> AudioCodec {
            if input.contains("mp4a") {
                AudioCodec::Mp4a
            } else if input.contains("opus") {
                AudioCodec::Opus
            } else {
                unreachable!()
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
                    video_codec: video_codec(first),
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

    dbg!(videos_only);
    Err(())
}

use anyhow::Result;
use directories::ProjectDirs;
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::time::Duration;

use crate::app::LyricLine;
use super::LyricProvider;

// ── Static regex — compiled exactly once ─────────────────────────────────────

static LRC_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\[(\d{2}):(\d{2})\.(\d{2,3})\](.*)").expect("Hard-coded LRC regex is valid")
});

// ── Provider ──────────────────────────────────────────────────────────────────

pub struct LrclibProvider {
    client: Client,
}

impl LrclibProvider {
    pub fn new(timeout_secs: u64) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(timeout_secs))
                .user_agent("lyric-tui/0.1.0 (https://gitlab.com/malkhuzanie/lyric-tui)")
                .build()
                .expect("Failed to build reqwest client"),
        }
    }
}

// ── Cache helpers ─────────────────────────────────────────────────────────────

fn get_cache_path(artist: &str, title: &str) -> Option<PathBuf> {
    let dirs = ProjectDirs::from("com", "tomato", "lyric-tui")?;
    let cache_dir = dirs.cache_dir().to_owned();
    fs::create_dir_all(&cache_dir).ok()?;

    let safe_artist = artist.replace(|c: char| !c.is_alphanumeric(), "_");
    let safe_title = title.replace(|c: char| !c.is_alphanumeric(), "_");
    Some(cache_dir.join(format!("{safe_artist} - {safe_title}.txt")))
}

// ── LRC parser ────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct LrcResponse {
    #[serde(rename = "plainLyrics")]
    plain_lyrics: Option<String>,
    #[serde(rename = "syncedLyrics")]
    synced_lyrics: Option<String>,
}

pub fn parse_lrc(lrc: &str) -> Vec<LyricLine> {
    let mut lines: Vec<LyricLine> = lrc
        .lines()
        .map(|raw| {
            let raw = raw.trim();
            if let Some(caps) = LRC_REGEX.captures(raw) {
                let m: u64 = caps[1].parse().unwrap_or(0);
                let s: u64 = caps[2].parse().unwrap_or(0);
                let ms_str = &caps[3];
                let ms: u64 = if ms_str.len() == 2 {
                    ms_str.parse::<u64>().unwrap_or(0) * 10
                } else {
                    ms_str.parse::<u64>().unwrap_or(0)
                };
                LyricLine {
                    start_time: Some(Duration::from_millis(m * 60_000 + s * 1_000 + ms)),
                    text: caps[4].trim().to_string(),
                }
            } else {
                LyricLine {
                    start_time: None,
                    text: raw.to_string(),
                }
            }
        })
        .collect();

    if lines.is_empty() {
        lines.push(LyricLine {
            start_time: None,
            text: "No lyrics available.".to_string(),
        });
    }

    lines
}

// ── LyricProvider impl ────────────────────────────────────────────────────────

#[async_trait::async_trait]
impl LyricProvider for LrclibProvider {
    async fn fetch(&self, artist: &str, title: &str, force_refresh: bool) -> Result<Vec<LyricLine>> {
        let cache_file = get_cache_path(artist, title);

        // Cache read
        if !force_refresh {
            if let Some(ref path) = cache_file {
                if let Ok(content) = fs::read_to_string(path) {
                    log::info!("LRCLIB cache hit: {}", path.display());
                    return Ok(parse_lrc(&content));
                }
            }
        }

        let url = format!(
            "https://lrclib.net/api/get?artist_name={}&track_name={}",
            urlencoding::encode(artist),
            urlencoding::encode(title),
        );

        let response = self.client.get(&url).send().await?;

        if response.status() == 404 {
            return Ok(parse_lrc("Lyrics not found in LRCLIB database."));
        }

        if !response.status().is_success() {
            return Ok(parse_lrc(&format!(
                "Network error: HTTP {}",
                response.status()
            )));
        }

        let res_text = response.text().await?;
        let data: LrcResponse = match serde_json::from_str(&res_text) {
            Ok(v) => v,
            Err(e) => {
                log::error!(
                    "Failed to decode LRCLIB JSON. Error: {}. Body: {}",
                    e, res_text
                );
                return Err(e.into());
            }
        };

        let lyrics_text = data
            .synced_lyrics
            .or(data.plain_lyrics)
            .unwrap_or_else(|| "Instrumental / No lyrics found in LRCLIB database.".to_string());

        // Cache write — non-fatal if it fails
        if let Some(ref path) = cache_file {
            if let Err(e) = fs::write(path, &lyrics_text) {
                log::warn!("Could not write LRCLIB cache: {}", e);
            }
        }

        Ok(parse_lrc(&lyrics_text))
    }
}

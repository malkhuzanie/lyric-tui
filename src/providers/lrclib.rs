use anyhow::Result;
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;
use std::sync::LazyLock;
use std::time::Duration;

use crate::app::LyricLine;
use super::LyricProvider;

// ── Static regex — compiled exactly once ─────────────────────────────────────

static LRC_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\[(\d{2}):(\d{2})\.(\d{2,3})\](.*)").expect("Hard-coded LRC regex is valid")
});

static STRIP_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[.*?\]|\(.*?\)").expect("Hard-coded strip regex is valid")
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
        // Cache read
        if !force_refresh {
            if let Some(content) = crate::providers::cache::read_cache(artist, title, "") {
                if let Some(path) = crate::providers::cache::get_cache_path(artist, title, "") {
                    log::info!("LRCLIB cache hit: {}", path.display());
                }
                return Ok(parse_lrc(&content));
            }
        }

        let url = format!(
            "https://lrclib.net/api/get?artist_name={}&track_name={}",
            urlencoding::encode(artist),
            urlencoding::encode(title),
        );

        let response = self.client.get(&url).send().await?;
        let data: LrcResponse;

        if response.status() == 404 {
            // The strict `/api/get` endpoint failed to locate the track. 
            // We now employ a fallback strategy: querying the `/api/search` endpoint 
            // with a concatenated query string. This fuzzy search mitigates issues 
            // where metadata contains extraneous tags (e.g., "[Official Video]").
            log::info!("LRCLIB exact match failed. Attempting fuzzy search for: {} {}", artist, title);
            let raw_query = format!("{} {}", artist, title);
            let stripped_query = STRIP_REGEX.replace_all(&raw_query, "");
            let query = stripped_query.split_whitespace().collect::<Vec<_>>().join(" ");
            
            let search_url = format!(
                "https://lrclib.net/api/search?q={}",
                urlencoding::encode(&query)
            );

            let search_response = self.client.get(&search_url).send().await?;
            if !search_response.status().is_success() {
                return Ok(parse_lrc(&format!(
                    "Network error: HTTP {}",
                    search_response.status()
                )));
            }

            let search_text = search_response.text().await?;
            let search_results: Vec<LrcResponse> = match serde_json::from_str(&search_text) {
                Ok(v) => v,
                Err(e) => {
                    log::error!(
                        "Failed to decode LRCLIB search JSON. Error: {}. Body: {}",
                        e, search_text
                    );
                    return Err(e.into());
                }
            };

            if let Some(first_match) = search_results.into_iter().next() {
                data = first_match;
            } else {
                return Ok(parse_lrc("Lyrics not found in LRCLIB database."));
            }
        } else if !response.status().is_success() {
            return Ok(parse_lrc(&format!(
                "Network error: HTTP {}",
                response.status()
            )));
        } else {
            let res_text = response.text().await?;
            data = match serde_json::from_str(&res_text) {
                Ok(v) => v,
                Err(e) => {
                    log::error!(
                        "Failed to decode LRCLIB JSON. Error: {}. Body: {}",
                        e, res_text
                    );
                    return Err(e.into());
                }
            };
        }

        let lyrics_text = data
            .synced_lyrics
            .or(data.plain_lyrics)
            .unwrap_or_else(|| "Instrumental / No lyrics found in LRCLIB database.".to_string());

        // Cache write — non-fatal if it fails
        crate::providers::cache::write_cache(artist, title, &lyrics_text, "", "LRCLIB");

        Ok(parse_lrc(&lyrics_text))
    }
}

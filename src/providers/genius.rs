use anyhow::{Context, Result};
use directories::ProjectDirs;
use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::time::Duration;
use urlencoding::encode;

use crate::app::LyricLine;
use super::LyricProvider;

// ── Static CSS selector — compiled exactly once ───────────────────────────────

static LYRICS_SELECTOR: LazyLock<Selector> = LazyLock::new(|| {
    Selector::parse("div[data-lyrics-container=\"true\"]")
        .expect("Hard-coded CSS selector is valid")
});

// ── Provider ──────────────────────────────────────────────────────────────────

pub struct GeniusProvider {
    client: Client,
}

impl GeniusProvider {
    pub fn new(timeout_secs: u64) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(timeout_secs))
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
    Some(cache_dir.join(format!("{safe_artist} - {safe_title}_genius.txt")))
}

// ── LyricProvider impl ────────────────────────────────────────────────────────

#[async_trait::async_trait]
impl LyricProvider for GeniusProvider {
    async fn fetch(&self, artist: &str, title: &str, force_refresh: bool) -> Result<Vec<LyricLine>> {
        let cache_file = get_cache_path(artist, title);

        // Cache read
        if !force_refresh {
            if let Some(ref path) = cache_file {
                if let Ok(content) = fs::read_to_string(path) {
                    log::info!("Genius cache hit: {}", path.display());
                    return Ok(plain_lines(&content));
                }
            }
        }

        // ── Step 1: search API ────────────────────────────────────────────────
        let query = format!("{} {}", artist, title);
        let search_url = format!(
            "https://genius.com/api/search/multi?q={}",
            encode(&query)
        );

        let res_text = self
            .client
            .get(&search_url)
            .header("User-Agent", browser_ua())
            .header("Accept", "application/json")
            .send()
            .await
            .context("Genius search request failed")?
            .text()
            .await
            .context("Failed to read Genius search response")?;

        if looks_like_bot_block(&res_text) {
            log::warn!("Genius search was blocked by Cloudflare.");
            return Ok(status_line(
                "Lyrics blocked by Genius (Cloudflare). Please try LRCLIB.",
            ));
        }

        let res: Value = match serde_json::from_str(&res_text) {
            Ok(v) => v,
            Err(e) => {
                log::error!(
                    "Failed to decode Genius search JSON. Error: {}. Body: {}",
                    e, res_text
                );
                return Err(e.into());
            }
        };

        let path = res
            .get("response")
            .and_then(|r| r.get("sections"))
            .and_then(|s| s.get(0))
            .and_then(|s| s.get("hits"))
            .and_then(|h| h.as_array())
            .filter(|h| !h.is_empty())
            .and_then(|h| h[0].get("result"))
            .and_then(|r| r.get("path"))
            .and_then(|p| p.as_str())
            .unwrap_or("");

        if path.is_empty() {
            return Ok(status_line("Lyrics not found on Genius."));
        }

        // ── Step 2: scrape the lyrics page ───────────────────────────────────
        let page_url = format!("https://genius.com{}", path);
        let page_html = self
            .client
            .get(&page_url)
            .header("User-Agent", browser_ua())
            .header(
                "Accept",
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            )
            .header("Accept-Language", "en-US,en;q=0.5")
            .send()
            .await
            .context("Genius page request failed")?
            .text()
            .await
            .context("Failed to read Genius page")?;

        if looks_like_bot_block(&page_html) {
            log::warn!("Genius lyrics page was blocked by Cloudflare.");
            return Ok(status_line(
                "Lyrics blocked by Genius (Cloudflare). Please try LRCLIB.",
            ));
        }

        let document = Html::parse_document(&page_html);
        let mut lyrics = String::new();

        for element in document.select(&LYRICS_SELECTOR) {
            let html = element.html().replace("<br>", "\n");
            let frag = Html::parse_fragment(&html);
            let text: String = frag.root_element().text().collect();
            lyrics.push_str(&text);
            lyrics.push('\n');
        }

        if lyrics.trim().is_empty() {
            return Ok(status_line("Lyrics not found on Genius."));
        }

        let trimmed = lyrics.trim();

        // Cache write — non-fatal if it fails
        if let Some(ref path) = cache_file {
            if let Err(e) = fs::write(path, trimmed) {
                log::warn!("Could not write Genius cache: {}", e);
            }
        }

        Ok(plain_lines(trimmed))
    }
}

// ── Small helpers ─────────────────────────────────────────────────────────────

fn plain_lines(text: &str) -> Vec<LyricLine> {
    text.lines()
        .map(|l| LyricLine {
            start_time: None,
            text: l.to_string(),
        })
        .collect()
}

fn status_line(msg: &str) -> Vec<LyricLine> {
    vec![LyricLine {
        start_time: None,
        text: msg.to_string(),
    }]
}

fn looks_like_bot_block(body: &str) -> bool {
    body.trim().starts_with("<!DOCTYPE html>") || body.contains("cloudflare")
}

fn browser_ua() -> &'static str {
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
     AppleWebKit/537.36 (KHTML, like Gecko) \
     Chrome/124.0.0.0 Safari/537.36"
}

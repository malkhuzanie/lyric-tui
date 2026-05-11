/// Shared cache utilities for lyric providers.
///
/// The cache format is provider-agnostic: it stores raw lyric text (LRC format
/// or plain text) under a sanitised artist/title key. Both LRCLIB and Genius
/// providers use this same cache layer.

use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

/// Returns the filesystem path where lyrics should be cached for the given metadata.
/// Returns `None` if the cache directory cannot be determined.
pub fn get_cache_path(artist: &str, title: &str, suffix: &str) -> Option<PathBuf> {
    let dirs = ProjectDirs::from("com", "tomato", "lyric-tui")?;
    let cache_dir = dirs.cache_dir().to_owned();
    fs::create_dir_all(&cache_dir).ok()?;

    let safe_artist = artist.replace(|c: char| !c.is_alphanumeric(), "_");
    let safe_title = title.replace(|c: char| !c.is_alphanumeric(), "_");
    Some(cache_dir.join(format!("{safe_artist} - {safe_title}{suffix}.txt")))
}

/// Write a lyrics cache entry for the given artist/title.
///
/// This function writes the raw lyrics text to the cache file. It is used by both
/// providers and by the command processor for dual-caching (storing lyrics under
/// both search query metadata and original player metadata).
pub fn write_cache(artist: &str, title: &str, lyrics_text: &str, suffix: &str, provider_name: &str) {
    if let Some(path) = get_cache_path(artist, title, suffix) {
        if let Err(e) = fs::write(&path, lyrics_text) {
            log::warn!("Could not write {} cache for {} – {}: {}", provider_name, artist, title, e);
        }
    }
}

/// Read a lyrics cache entry for the given artist/title.
///
/// Returns the raw lyrics text if a cache hit occurs, or `None` if the file
/// does not exist or cannot be read.
pub fn read_cache(artist: &str, title: &str, suffix: &str) -> Option<String> {
    let path = get_cache_path(artist, title, suffix)?;
    fs::read_to_string(path).ok()
}


pub mod genius;
pub mod lrclib;
use crate::app::LyricLine;
use anyhow::Result;
#[async_trait::async_trait]
pub trait LyricProvider {
    async fn fetch(&self, artist: &str, title: &str, force_refresh: bool) -> Result<Vec<LyricLine>>;
}

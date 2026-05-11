/// A dedicated heuristic sanitisation module designed to aggressively purge 
/// malformed tags from media metadata prior to network propagation.
use std::sync::LazyLock;
use regex::Regex;

static TRACK_PREFIX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(?:\d+|[A-D]\d+)(?:\s*[.\-]\s*|\s+)").expect("Static track prefix regex is valid")
});

// Rationale: Upstream API providers rely on canonical string matching. 
// The presence of embedded parenthetical metadata (e.g., "[Official Audio]", "(Remastered 2009)") 
// practically guarantees a cache miss. We aggressively match and purge these patterns.
static PARENTHETICAL_NOISE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)[\[\(].*?(?:official|remaster|live|acoustic|instrumental|bonus|feat|ft\.|radio|edit|cover|mix|version|explicit|clean).*?[\]\)]").expect("Static parenthetical regex is valid")
});

// Rationale: Identical to parenthetical noise, but targets suffixes appended via dashes.
static SUFFIX_NOISE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\s+-\s+(?:remaster|live|acoustic|instrumental|bonus|radio|edit|cover|mix|version|explicit|clean).*$").expect("Static suffix regex is valid")
});

// Rationale: Lyrics databases heavily index by the primary artist. 
// We strip collaborative markers to isolate the primary entity.
static FEATURING_NOISE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\s+(?:feat\.?|ft\.?|featuring)\s+.*$").expect("Static featuring regex is valid")
});

// Rationale: Artists' lifespan or release years are sometimes incorrectly embedded 
// in tags. This breaks exact string matching.
static ARTIST_YEAR_NOISE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\s*[\[\(]\d{4}\s*(?:-\s*\d{2,4})?[\]\)]").expect("Static artist year regex is valid")
});

/// Sanitises the track title by purging numerical prefixes, extraneous parentheticals, and featuring suffixes.
pub fn clean_title(raw: &str) -> String {
    let mut cleaned = TRACK_PREFIX.replace(raw, "").into_owned();
    cleaned = PARENTHETICAL_NOISE.replace_all(&cleaned, "").into_owned();
    cleaned = SUFFIX_NOISE.replace_all(&cleaned, "").into_owned();
    cleaned = FEATURING_NOISE.replace_all(&cleaned, "").into_owned();
    
    // Re-collapse multiple spaces left behind by greedy regular expressions.
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Sanitises the artist string by stripping collaborative markers and secondary artists.
pub fn clean_artist(raw: &str) -> String {
    let mut cleaned = FEATURING_NOISE.replace_all(raw, "").into_owned();
    cleaned = ARTIST_YEAR_NOISE.replace_all(&cleaned, "").into_owned();
    
    // Rationale: Media players frequently supply a comma-separated or ampersand-delimited list of artists. 
    // Querying solely the primary artist significantly increases the probability of a successful API match.
    if let Some(primary) = cleaned.split(',').next() {
        cleaned = primary.to_string();
    }
    if let Some(primary) = cleaned.split(" & ").next() {
        cleaned = primary.to_string();
    }

    cleaned.trim().to_string()
}

/// Sanitises both artist and title, applying cross-field heuristics such as 
/// extracting the true artist from YouTube-style combined titles.
pub fn sanitize_metadata(artist: &str, title: &str) -> (String, String) {
    let mut current_artist = artist.to_string();
    let mut current_title = title.to_string();

    // Rationale: YouTube frequently sets the artist to the channel name (e.g., "TheBanglesVEVO") 
    // and concatenates the artist and title in the title field (e.g., "The Bangles - Eternal Flame").
    if let Some((extracted_artist, extracted_title)) = title.split_once(" - ") {
        let channel_name = artist.replace(" ", "").to_lowercase();
        let extracted_artist_no_spaces = extracted_artist.replace(" ", "").to_lowercase();
        
        let is_vevo = channel_name.ends_with("vevo");
        let is_topic = channel_name.ends_with("topic");
        let matches_channel = channel_name.contains(&extracted_artist_no_spaces) || extracted_artist_no_spaces.contains(&channel_name);

        if is_vevo || is_topic || matches_channel {
            current_artist = extracted_artist.to_string();
            current_title = extracted_title.to_string();
        }
    }

    (clean_artist(&current_artist), clean_title(&current_title))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_title() {
        assert_eq!(clean_title("01. Song (Official Audio)"), "Song");
        assert_eq!(clean_title("02 - Track [Remastered]"), "Track");
        assert_eq!(clean_title("A10. Song feat. Someone"), "Song");
        assert_eq!(clean_title("Song Name - Live at Wembley"), "Song Name");
        assert_eq!(clean_title("A1 In The Air Tonight"), "In The Air Tonight");
    }

    #[test]
    fn test_clean_artist() {
        assert_eq!(clean_artist("Primary Artist feat. Secondary"), "Primary Artist");
        assert_eq!(clean_artist("Artist A, Artist B"), "Artist A");
        assert_eq!(clean_artist("Artist One & Artist Two"), "Artist One");
        assert_eq!(clean_artist("Someone ft. Another"), "Someone");
        assert_eq!(clean_artist("Elvis Presley (1958-92)"), "Elvis Presley");
        assert_eq!(clean_artist("Some Artist [1984]"), "Some Artist");
    }

    #[test]
    fn test_sanitize_metadata() {
        let (a, t) = sanitize_metadata("TheBanglesVEVO", "The Bangles - Eternal Flame (Official Video)");
        assert_eq!(a, "The Bangles");
        assert_eq!(t, "Eternal Flame");

        let (a, t) = sanitize_metadata("Queen Official", "Queen - Bohemian Rhapsody");
        assert_eq!(a, "Queen");
        assert_eq!(t, "Bohemian Rhapsody");
    }
}

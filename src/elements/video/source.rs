use std::path::PathBuf;

/// Where a [`super::VideoPlayer`] loads its media from.
///
/// Only containers and codecs MediaKit decodes natively play
/// (see `mediakit` docs). URLs must point to a directly downloadable
/// file; HLS playlists (`.m3u8`) are rejected because their segments
/// use delivery codecs outside the native decoders.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VideoSource {
    /// Absolute file path on disk.
    File(PathBuf),
    /// `http(s)://` URL, downloaded to temp on a background thread.
    Url(String),
}

impl VideoSource {
    /// File name (or URL tail) for labels and errors.
    pub fn display_name(&self) -> String {
        match self {
            VideoSource::File(path) => path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string_lossy().to_string()),
            VideoSource::Url(url) => url
                .rsplit('/')
                .next()
                .filter(|s| !s.is_empty())
                .unwrap_or(url)
                .to_string(),
        }
    }

    /// Rejection reason for URLs we cannot play, if any.
    pub fn url_problem(url: &str) -> Option<String> {
        let lower = url.trim().to_ascii_lowercase();
        if !lower.starts_with("http://") && !lower.starts_with("https://") {
            return Some(format!("URL must start with http(s)://, got {url}"));
        }
        let path = lower.split(['?', '#']).next().unwrap_or("");
        if path.ends_with(".m3u8") {
            return Some(
                "HLS playlists (.m3u8) need segment codecs outside the native decoders".to_string(),
            );
        }
        None
    }
}

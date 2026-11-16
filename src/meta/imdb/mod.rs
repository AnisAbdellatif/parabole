//! IMDb's unofficial suggestion and GraphQL endpoints.
pub mod details;
pub mod suggest;

/// Rewrites an Amazon-hosted poster URL to a `width`-pixel rendition.
pub fn sized(url: &str, width: u32) -> String {
    match url.find("._V1_") {
        Some(i) if url.ends_with(".jpg") => format!("{}._V1_SX{width}.jpg", &url[..i]),
        _ => url.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::sized;

    #[test]
    fn resizes_amazon_posters() {
        assert_eq!(
            sized("https://m.media-amazon.com/images/M/MV5B.x._V1_.jpg", 600),
            "https://m.media-amazon.com/images/M/MV5B.x._V1_SX600.jpg"
        );
        assert_eq!(
            sized(
                "https://m.media-amazon.com/images/M/MV5B._V1_UX67_CR0,0,67,98_AL_.jpg",
                600
            ),
            "https://m.media-amazon.com/images/M/MV5B._V1_SX600.jpg"
        );
        let other = "https://example.com/p.png";
        assert_eq!(sized(other, 600), other);
        let no_marker = "https://example.com/p.jpg";
        assert_eq!(sized(no_marker, 600), no_marker);
    }
}

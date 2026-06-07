/// Shorten a macOS bundle identifier to a human-recognizable name.
///
/// Bundle IDs are reverse-DNS (`com.apple.Safari`). The last segment is
/// usually the app name, but some vendors end with a generic segment
/// (`com.cmuxterm.app`, `notion.id`, `dev.pencil.desktop`) — for those the
/// second-to-last segment is the recognizable name.
pub fn shorten_bundle_id(bundle_id: &str) -> &str {
    const GENERIC_SUFFIXES: [&str; 3] = ["app", "desktop", "id"];

    let mut segments = bundle_id.rsplit('.');
    let last = segments.next().unwrap_or(bundle_id);

    if GENERIC_SUFFIXES.contains(&last.to_ascii_lowercase().as_str()) {
        if let Some(previous) = segments.next() {
            return previous;
        }
    }

    last
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_bundle_id_uses_last_segment() {
        assert_eq!(shorten_bundle_id("com.apple.Safari"), "Safari");
        assert_eq!(shorten_bundle_id("md.obsidian"), "obsidian");
        assert_eq!(shorten_bundle_id("com.mitchellh.ghostty"), "ghostty");
    }

    #[test]
    fn test_generic_suffix_falls_back_to_previous_segment() {
        assert_eq!(shorten_bundle_id("com.cmuxterm.app"), "cmuxterm");
        assert_eq!(shorten_bundle_id("com.linear.app"), "linear");
        assert_eq!(shorten_bundle_id("com.conductor.app"), "conductor");
        assert_eq!(shorten_bundle_id("notion.id"), "notion");
        assert_eq!(shorten_bundle_id("dev.pencil.desktop"), "pencil");
    }

    #[test]
    fn test_generic_suffix_is_case_insensitive() {
        assert_eq!(shorten_bundle_id("com.example.App"), "example");
    }

    #[test]
    fn test_no_dots_returns_input_unchanged() {
        assert_eq!(shorten_bundle_id("Unknown"), "Unknown");
        // A bare generic word has no previous segment to fall back to.
        assert_eq!(shorten_bundle_id("app"), "app");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(shorten_bundle_id(""), "");
    }
}

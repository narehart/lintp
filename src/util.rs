/// Normalize a path-like string to forward slashes.
///
/// `Path::display()` on Windows renders `\`-separated paths, but every
/// DSL-exposed variable (`$PATH`, `$PARENT`) and every glob pattern built
/// from a path (in `siblings()`, `children()`, `exists()`, `find()`) is
/// documented and written assuming `/`. Without this, the same rule that
/// works on Unix would silently fail to match on Windows, and the `glob`
/// crate — which treats `\` as an escape character, not a separator — would
/// misinterpret the pattern entirely.
///
/// A no-op on Unix, where paths never contain `\` in practice.
pub fn forward_slashes(s: &str) -> String {
    s.replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forward_slashes_converts_backslashes() {
        assert_eq!(
            forward_slashes(r"src\components\Button.tsx"),
            "src/components/Button.tsx"
        );
    }

    #[test]
    fn test_forward_slashes_is_noop_on_unix_style_paths() {
        assert_eq!(
            forward_slashes("src/components/Button.tsx"),
            "src/components/Button.tsx"
        );
    }
}

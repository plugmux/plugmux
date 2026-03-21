pub fn slugify(name: &str) -> String {
    slug::slugify(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify_basic() {
        assert_eq!(slugify("My SaaS App"), "my-saas-app");
    }

    #[test]
    fn test_slugify_special_chars() {
        assert_eq!(slugify("Rust & Embedded"), "rust-embedded");
    }

    #[test]
    fn test_slugify_already_slug() {
        assert_eq!(slugify("my-project"), "my-project");
    }
}

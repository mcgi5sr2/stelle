pub fn slugify(input: &str) -> Option<String> {
    let slug = input
        .to_lowercase()
        .replace(" ", "-")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect::<String>();

    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        None
    } else {
        Some(slug)
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn lowercases_input() {
        assert_eq!(slugify("Roman Amphora"), Some("roman-amphora".to_string()));
    }

    #[test]
    fn strips_special_characters() {
        assert_eq!(slugify("Hello, World!"), Some("hello-world".to_string()));
    }

    #[test]
    fn rejects_empty_input() {
        assert_eq!(slugify(""), None);
    }

    #[test]
    fn rejects_only_special_chars() {
        assert_eq!(slugify("!!!"), None);
    }

    #[test]
    fn strips_leading_traling_hypens() {
        assert_eq!(slugify("-hello-"), Some("hello".to_string()));
    }
}
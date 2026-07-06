use unicode_normalization::UnicodeNormalization;

#[must_use]
pub fn normalise_name(name: &str) -> String {
    let normalised: String = name
        .nfkc()
        .collect::<String>()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();

    if normalised.len() < 2 {
        format!("{normalised}_")
    } else {
        normalised
    }
}

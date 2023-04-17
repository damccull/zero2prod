use unicode_segmentation::UnicodeSegmentation;

pub struct SubscriberName(String);

impl SubscriberName {
    /// Construct a valid [`SubscriberName`] from a String.
    pub fn parse(value: String) -> Result<SubscriberName, String> {
        // Check if the string is empty or just whitespace characters
        let is_empty_or_whitespace = value.trim().is_empty();

        // Ensure less than 256 graphemes
        //(graphemes are basically fully-built characters made from 1 or more unicode pieces)
        let is_too_long = value.graphemes(true).count() > 256;

        // Check for forbidden characters
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters =
            value.chars().any(|g| forbidden_characters.contains(&g));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            Err(format!("{} is not a valid subscriber name", value))
        } else {
            Ok(Self(value))
        }
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

pub struct NewSubscriber {
    pub email: String,
    pub name: SubscriberName,
}

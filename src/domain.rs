use unicode_segmentation::UnicodeSegmentation;

pub struct NewSubscriber {
    pub email: String,
    pub name: SubscriberName,
}

pub struct SubscriberName(String);
impl SubscriberName {
    /// Return an instance of `SubscriberName` if the input satisfies all
    /// of the validation constraints on subscriber names.
    pub fn parse(s: String) -> SubscriberName {
        // Trim the string and ensure it's not empty
        let is_empty_or_whitespace = s.trim().is_empty();

        // A grapheme is defined by the Unicode standard as a "user-perceived"
        // character: `å` is a single grapheme, but it is composed of two characters
        // (`a` and `̊`).
        //
        // `graphemes` returns an iterator over the graphemes in the input `s`.
        // `true` specifies that we want to use the extended grapheme definition set,
        // the recommended one.
        let is_too_long = s.graphemes(true).count() > 256;

        // Ensure there are no invalid characters
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            panic!(format!("{} is not a valid subscriber name.", s))
        } else {
            Self(s)
        }
    }

    // /// Returns the inner `String`, consuming the `SubscriberName` in the process.
    // pub fn inner(self) -> String {
    //     // The caller gets the inner string,
    //     // but they do not have a SubscriberName anymore!
    //     // That's because `inner` takes `self` by value,
    //     // consuming it according to the move semantics.
    //     self.0
    // }

    /// Returns a reference to the inner `String`.
    pub fn inner_ref(&self) -> &str {
        // The caller gets a shared reference to the inner string.
        // This gives them to *read only* access, allowing now way
        // to compromise the invariants.
        &self.0
    }
}

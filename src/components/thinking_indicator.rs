use std::time::Duration;

use iocraft::prelude::*;
use rand::seq::IteratorRandom;

use crate::components::COLOR_PRIMARY;

const ENCHANTING_TEXT: &'static str =
    "⍑ᒷᓵ∷ᔑ∴ꖎᒷ↸∴╎ℸ⍑ᓵᔑ∷ᒷᔑꖎ𝙹リ⊣ℸ⍑ᒷꖎᒷ↸⊣ᒷ.ℸ⍑ᒷ⎓╎リ∴ᔑᓭᓭ⍑ᔑ∷!¡ᔑリ↸ᓵ⚍ℸ⍑ᓵꖎᒷᔑ∷∴ᔑℸᒷ∷.";

#[component]
pub fn ThinkingIndicator(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut indicator = hooks.use_state(|| " ".to_string());

    element! {
        Text (content: format!("{} Enchanting...", indicator), color: COLOR_PRIMARY)
    }
}

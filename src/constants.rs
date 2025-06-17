use std::collections::HashSet;
use std::sync::LazyLock;
pub static HER_KEY_WORDS: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| HashSet::from_iter(["女性", "her", "女", "female", "woman", "girl", "lady"]));

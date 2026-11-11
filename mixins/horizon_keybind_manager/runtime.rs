pub const ID: &str = "horizon_keybind_manager";
pub const VERSION: &str = "0.1.0";
pub const TITLE: &str = "Horizon Keybind Manager";
pub const UNBOUND: &str = "key.keyboard.unknown";

pub struct Search {
    text: String,
    exact_key: bool,
}

impl Search {
    pub fn new(query: &str) -> Self {
        let query = query.trim().to_lowercase();
        let key = query
            .strip_prefix("key:")
            .or_else(|| query.strip_prefix("клавиша:"));
        Self {
            text: key.unwrap_or(&query).trim().to_owned(),
            exact_key: key.is_some(),
        }
    }

    pub fn matches(&self, label: &str, group: &str, key: &str, key_label: &str) -> bool {
        if self.text.is_empty() {
            return true;
        }
        let key_name = key
            .strip_prefix("key.keyboard.")
            .or_else(|| key.strip_prefix("key.mouse."))
            .unwrap_or(key);
        let key_label = key_label.to_lowercase();
        if self.exact_key {
            return self.text == key || self.text == key_name || self.text == key_label;
        }
        let label = label.to_lowercase();
        let group = group.to_lowercase();
        self.text.split_whitespace().all(|word| {
            label.contains(word)
                || group.contains(word)
                || key_name == word
                || key_label.contains(word)
                || key == word
        })
    }
}

#[derive(Clone, Copy)]
pub struct Chord<'a> {
    pub key: &'a str,
    pub with_debug: bool,
    pub default_game_key: bool,
}

impl Chord<'_> {
    pub fn conflicts(self, other: Self) -> bool {
        !self.key.is_empty()
            && self.key != UNBOUND
            && self.key == other.key
            && self.with_debug == other.with_debug
            && !(self.default_game_key && other.default_game_key)
    }
}

pub fn next_conflict(bindings: &[Chord<'_>], current: usize) -> Option<usize> {
    let binding = *bindings.get(current)?;
    let (before, at_and_after) = bindings.split_at(current);
    at_and_after[1..]
        .iter()
        .enumerate()
        .map(|(index, other)| (current + index + 1, other))
        .chain(before.iter().enumerate())
        .find_map(|(index, other)| binding.conflicts(*other).then_some(index))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn localized_actions_and_mod_names_are_searchable() {
        assert!(Search::new("кАрТ  world").matches(
            "Открыть карту",
            "World Map",
            "key.keyboard.m",
            "M"
        ));
        assert!(Search::new("колесо").matches(
            "Подбор блока",
            "Игра",
            "key.mouse.middle",
            "Колесо мыши"
        ));
    }

    #[test]
    fn exact_key_search_does_not_match_action_names_or_partial_keys() {
        let search = Search::new(" key: F1 ");
        assert!(search.matches("GUI", "Misc", "key.keyboard.f1", "F1"));
        assert!(!search.matches("F1 action", "Misc", "key.keyboard.f11", "F11"));
        assert!(Search::new("Клавиша: M").matches("Карта", "Моды", "key.keyboard.m", "Ь"));
    }

    fn chord(key: &str, default_game_key: bool, with_debug: bool) -> Chord<'_> {
        Chord {
            key,
            with_debug,
            default_game_key,
        }
    }

    #[test]
    fn jumping_wraps_across_game_and_mod_keys_and_skips_debug_chords() {
        let bindings = [
            chord("key.keyboard.m", true, false),
            chord("key.keyboard.m", true, true),
            chord("key.keyboard.m", false, false),
            chord("key.keyboard.m", false, false),
        ];
        assert_eq!(next_conflict(&bindings, 0), Some(2));
        assert_eq!(next_conflict(&bindings, 1), None);
        assert_eq!(next_conflict(&bindings, 2), Some(3));
        assert_eq!(next_conflict(&bindings, 3), Some(0));
    }

    #[test]
    fn ignores_vanilla_default_overlaps_and_unbound_slots() {
        let default = chord("key.mouse.middle", true, false);
        let assigned = chord("key.mouse.middle", false, false);
        let unbound = chord(UNBOUND, false, false);
        assert!(!default.conflicts(default));
        assert!(assigned.conflicts(default));
        assert!(!unbound.conflicts(unbound));
        assert_eq!(next_conflict(&[default], 0), None);
        assert_eq!(next_conflict(&[], 0), None);
    }
}

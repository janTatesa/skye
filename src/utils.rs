use indexmap::IndexMap;
use owo_colors::{AnsiColors, OwoColorize};

pub struct GetMutFilter<T> {
    pub function: fn(&T) -> bool,
    pub error: &'static str
}

pub fn get_mut<'a, T>(
    store: &'a mut IndexMap<String, T>,
    name: &str,
    filter: Option<&GetMutFilter<T>>
) -> &'a mut T {
    let Some(item) = store.get_mut(name) else {
        exit_with_error("Doesn't exist!");
    };

    if let Some(GetMutFilter { function, error }) = filter
        && !function(item)
    {
        exit_with_error(error)
    }

    item
}

pub fn remove<T>(store: &mut IndexMap<String, T>, name: &str) {
    if store.shift_remove(name).is_none() {
        exit_with_error("Doesn't exist!");
    }
}

pub fn add<T>(store: &mut IndexMap<String, T>, name: String, item: T) {
    if store.contains_key(&name) {
        exit_with_error("Already exists!");
    }

    store.insert(name, item);
}

pub fn rename<T>(store: &mut IndexMap<String, T>, old: &str, new: String) {
    if store.contains_key(&new) {
        exit_with_error("Item with the new name already exists!");
    }

    let (Some(idx), Some(old)) = (store.get_index_of(old), store.shift_remove(old)) else {
        exit_with_error("Item with the old name doesnt exit!");
    };

    store.shift_insert(idx, new, old);
}

pub fn exit_with_error(error: &str) -> ! {
    eprintln!("{}", error.red());
    std::process::exit(1)
}

#[expect(clippy::match_overlapping_arm, reason = "pwettyness")]
pub fn days_to_color(days: u32) -> AnsiColors {
    match days {
        ..1 => AnsiColors::Red,
        ..3 => AnsiColors::Yellow,
        ..7 => AnsiColors::Green,
        ..14 => AnsiColors::Cyan,
        ..21 => AnsiColors::Blue,
        _ => AnsiColors::Magenta
    }
}

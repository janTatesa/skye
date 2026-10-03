use dialoguer::theme::ColorfulTheme;
use indexmap::IndexMap;
use owo_colors::{AnsiColors, OwoColorize};

pub struct GetMutFilter<T> {
    pub function: fn(&T) -> bool,
    pub error: &'static str
}

pub fn get_mut<'a, T>(
    store: &'a mut IndexMap<String, T>,
    name: Option<String>,
    filter: Option<&GetMutFilter<T>>
) -> color_eyre::Result<&'a mut T> {
    let values = store
        .iter()
        .filter(|(_, val)| filter.as_ref().is_none_or(|filter| (filter.function)(val)))
        .map(|(name, _)| name.as_str());
    let name = unwrap_or_ask(name, values)?;

    let Some(item) = store.get_mut(&name) else {
        exit_with_error("Doesn't exist!");
    };

    if let Some(GetMutFilter { function, error }) = filter
        && !function(item)
    {
        exit_with_error(error)
    }

    Ok(item)
}

pub fn remove<T>(store: &mut IndexMap<String, T>, name: Option<String>) -> color_eyre::Result {
    let name = unwrap_or_ask(name, store.keys().map(String::as_str))?;
    if store.shift_remove(&name).is_none() {
        exit_with_error("Doesn't exist!");
    }

    Ok(())
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

fn exit_with_error(error: &str) -> ! {
    eprintln!("{}", error.red());
    std::process::exit(1)
}

// has to return string rn because indexmap problems
fn unwrap_or_ask<'a>(
    name: Option<String>,
    items: impl IntoIterator<Item = &'a str>
) -> color_eyre::Result<String> {
    let items: Vec<_> = items.into_iter().collect();
    Ok(if let Some(name) = name {
        name
    } else {
        if items.is_empty() {
            exit_with_error("There are none");
        }

        let idx = dialoguer::Select::with_theme(&ColorfulTheme::default())
            .default(0)
            .items(items.iter())
            .interact()?;
        items[idx].to_string()
    })
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

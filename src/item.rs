use std::cmp::Ordering;

use indexmap::IndexMap;
use jiff::Zoned;

use crate::{store::Store, utils::exit_with_error};

#[derive(Clone, Copy)]
pub struct NoFilter;
impl AsRef<str> for NoFilter {
    fn as_ref(&self) -> &str {
        unimplemented!()
    }
}

pub trait Item: Sized {
    type Filter: AsRef<str> + Copy + Send + Sync;

    fn get_items_mut(store: &mut Store) -> &mut IndexMap<String, Self>;

    fn show(&self, name: &str, now: &Zoned);

    fn filter(&self, filter: Self::Filter, now: &Zoned) -> bool {
        let _ = (filter, now);
        true
    }

    fn sort(&self, other: &Self, now: &Zoned) -> Ordering {
        let _ = (other, now);
        Ordering::Equal
    }

    fn completion_help(&self, filter: Option<Self::Filter>, now: &Zoned) -> Option<String> {
        let _ = (filter, now);
        None
    }
}

pub fn get_mut<'a, T: Item>(
    store: &'a mut Store,
    name: &str,
    filter: Option<T::Filter>,
    now: &Zoned
) -> &'a mut T {
    let Some(item) = T::get_items_mut(store).get_mut(name) else {
        exit_with_error("Doesn't exist!");
    };

    if let Some(filter) = filter
        && !item.filter(filter, now)
    {
        exit_with_error(filter.as_ref())
    }

    item
}

pub fn remove<T: Item>(store: &mut Store, name: &str) {
    if T::get_items_mut(store).shift_remove(name).is_none() {
        exit_with_error("Doesn't exist!");
    }
}

pub fn add<T: Item>(store: &mut Store, name: String, item: T) {
    let items = T::get_items_mut(store);
    if items.contains_key(&name) {
        exit_with_error("Already exists!");
    }

    items.insert(name, item);
}

pub fn rename<T: Item>(store: &mut Store, old: &str, new: String) {
    let items = T::get_items_mut(store);
    if items.contains_key(&new) {
        exit_with_error("Item with the new name already exists!");
    }

    let (Some(idx), Some(old)) = (items.get_index_of(old), items.shift_remove(old)) else {
        exit_with_error("Item with the old name doesnt exit!");
    };

    items.shift_insert(idx, new, old);
}

pub fn show<T: Item>(store: &mut Store, filter: Option<T::Filter>, now: &Zoned) {
    let mut items: Vec<_> = T::get_items_mut(store)
        .iter_mut()
        .filter(|(_, item)| filter.is_none_or(|filter| item.filter(filter, now)))
        .collect();
    items.sort_by(|(_, a), (_, b)| b.sort(a, now));
    for (name, item) in items {
        item.show(name, now);
    }
}

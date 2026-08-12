use std::fmt;

pub(crate) fn option_text(value: Option<&str>) -> String {
    value.map_or_else(|| "none".to_owned(), |item| format!("some:{item}"))
}

pub(crate) fn option_number<T: fmt::Display>(value: Option<T>) -> String {
    value.map_or_else(|| "none".to_owned(), |item| format!("some:{item}"))
}

pub(crate) fn option_debug<T: fmt::Debug>(value: Option<T>) -> String {
    value.map_or_else(|| "none".to_owned(), |item| format!("some:{item:?}"))
}

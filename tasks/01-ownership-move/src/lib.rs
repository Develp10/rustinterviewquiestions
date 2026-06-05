//! # Задача 01: Владение, move и роль Cow в дизайне API
//!
//! Напишите функцию normalize, которая принимает строку и возвращает её
//! в нижнем регистре без краевых пробелов. Если входная строка уже
//! нормализована, функция не должна выделять память.
//!
//! Запуск тестов: cargo test -p ownership-move
//! Проверка Miri:  cargo +nightly miri test -p ownership-move

use std::borrow::Cow;

/// Нормализует строку: trim + to_lowercase.
/// Если строка уже нормализована — возвращает Borrowed (без аллокации).
pub fn normalize(s: &str) -> Cow<'_, str> {
    let trimmed = s.trim();
    let needs_lower = trimmed.chars().any(|c| c.is_uppercase());
    let needs_trim = trimmed.len() != s.len();

    match (needs_lower, needs_trim) {
        (false, false) => Cow::Borrowed(s),
        (false, true) => Cow::Borrowed(trimmed),
        (true, _) => Cow::Owned(trimmed.to_lowercase()),
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn already_normalized_borrows() {
        let got = normalize("hello");
        assert!(matches!(got, Cow::Borrowed(_)));
        assert_eq!(got, "hello");
    }

    #[test]
    fn trims_without_alloc() {
        let got = normalize("  hello  ");
        assert!(matches!(got, Cow::Borrowed(_)));
        assert_eq!(got, "hello");
    }

    #[test]
    fn lowercases_allocs() {
        let got = normalize("Hello");
        assert!(matches!(got, Cow::Owned(_)));
        assert_eq!(got, "hello");
    }

    #[test]
    fn trim_and_lower_allocs() {
        let got = normalize("  World  ");
        assert!(matches!(got, Cow::Owned(_)));
        assert_eq!(got, "world");
    }

    #[test]
    fn empty_borrows() {
        let got = normalize("");
        assert!(matches!(got, Cow::Borrowed(_)));
        assert_eq!(got, "");
    }
}

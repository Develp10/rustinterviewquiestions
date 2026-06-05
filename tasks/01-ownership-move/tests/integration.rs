//! Интеграционные тесты для задачи 01: normalize (Cow<str>)
//!
//! Запуск: cargo test -p ownership-move

use ownership_move::normalize;
use std::borrow::Cow;

#[test]
fn integration_already_normalized() {
    let s = "rust is fast";
    let result = normalize(s);
    assert_eq!(result, "rust is fast");
    // Нет аллокации: результат заимствует входную строку
    assert!(matches!(result, Cow::Borrowed(_)));
}

#[test]
fn integration_trim_only() {
    let result = normalize("   hello   ");
    assert_eq!(result, "hello");
    assert!(matches!(result, Cow::Borrowed(_)));
}

#[test]
fn integration_lowercase_only() {
    let result = normalize("Rust");
    assert_eq!(result, "rust");
    assert!(matches!(result, Cow::Owned(_)));
}

#[test]
fn integration_trim_and_lower() {
    let result = normalize("  Rust Is Great  ");
    assert_eq!(result, "rust is great");
    assert!(matches!(result, Cow::Owned(_)));
}

#[test]
fn integration_unicode_remains_borrowed() {
    let result = normalize("привет");
    assert_eq!(result, "привет");
    assert!(matches!(result, Cow::Borrowed(_)));
}

#[test]
fn integration_unicode_uppercase_allocates() {
    let result = normalize("Привет");
    assert_eq!(result, "привет");
    assert!(matches!(result, Cow::Owned(_)));
}

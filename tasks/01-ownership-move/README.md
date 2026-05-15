# 01. Владение, move и роль `Cow` в дизайне API

## Условие

Напишите функцию `normalize`, которая принимает строку и возвращает её в нижнем регистре без краевых пробелов. Если входная строка уже нормализована, функция не должна выделять память. Предложите сигнатуру, подходящую для библиотечного API.

## Наивная попытка

```rust
pub fn normalize(s: String) -> String {
    s.trim().to_lowercase()
}
```

Проблем здесь две. Первая — `String` в качестве входа принуждает вызывающий код отдавать владение или клонировать. Если вызывающий передаёт `&str`, ему придётся писать `s.to_string()` в каждом вызове. Вторая — `to_lowercase()` всегда аллоцирует новый `String`, даже если строка уже в lowercase и без пробелов.

## Разбор

Сигнатуры функций в Rust выражают контракт владения. `fn(String) -> String` говорит вызывающему: ты отдаёшь мне свою строку навсегда, получишь взамен новую. `fn(&str) -> String` — ты даёшь мне посмотреть, я верну свежевыделенную копию. Ни один из этих контрактов не позволяет избежать аллокации, когда вход уже хорош.

`Cow<'_, str>` решает это через два варианта в одном типе: `Borrowed(&'a str)` или `Owned(String)`. Вызывающий передаёт `&str`, функция на горячем пути возвращает тот же срез без аллокации. На холодном — аллоцирует и возвращает `Owned`.

## Рабочее решение

```rust
use std::borrow::Cow;

pub fn normalize(s: &str) -> Cow<'_, str> {
    let trimmed = s.trim();
    let needs_lower = trimmed.chars().any(|c| c.is_uppercase());
    let needs_trim = trimmed.len() != s.len();

    match (needs_lower, needs_trim) {
        (false, false) => Cow::Borrowed(s),
        (false, true)  => Cow::Borrowed(trimmed),
        (true, _)      => Cow::Owned(trimmed.to_lowercase()),
    }
}
```

Тесты:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    #[test]
    fn already_normalized_does_not_allocate() {
        let input = "hello";
        let got = normalize(input);
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
    fn lowercases_with_alloc() {
        let got = normalize("Hello");
        assert!(matches!(got, Cow::Owned(_)));
        assert_eq!(got, "hello");
    }
}
```

## Когда что выбирать

- `&str -> String`: просто и понятно, но всегда аллокация. ОК для внутренних функций, где вызовов мало.
- - `String -> String`: имеет смысл, когда функция может переиспользовать буфер (например `s.make_ascii_lowercase()` на месте).
  - - `&str -> Cow<'_, str>`: библиотечный стандарт для операций, которые иногда модифицируют строку, иногда нет. Именно так сделан `str::replace`, `Path::to_string_lossy`, `percent_encoding`, HTML/URL-энкодеры.
   
    - ## Подводные камни
   
    - `Cow<'a, str>` имеет лайфтайм, завязанный на вход. Если вы хотите вернуть `Cow`, не связанный с аргументом, используйте `Cow<'static, str>` или явный `String`. Не используйте `Cow` внутри структур без ясного понимания лайфтаймов: `struct Config { name: Cow<'static, str> }` работает и для литералов, и для рунтайм-строк, но пытаться засунуть `Cow<'a, str>` в долгоживущий owned-тип — плохая идея.
   
    - ## Где встречается в проде
   
    - - `serde::Deserialize` для строк по умолчанию возвращает `Cow<'de, str>`, чтобы zero-copy работало на JSON без escape-символов.
      - - `tokenizers` (HuggingFace, Rust) берёт `Cow<'_, str>` в normalizer'ах.
        - - `axum::extract::Path` и query-экстракторы используют тот же приём.
         
          - ## Ссылки
         
          - - std::borrow::Cow: https://doc.rust-lang.org/std/borrow/enum.Cow.html
            - - Rust API Guidelines, раздел C-COMMON-TRAITS
              - - serde zero-copy: https://serde.rs/lifetimes.html
                - 

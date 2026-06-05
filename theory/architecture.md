# Архитектура и дизайн API

← [Макросы и метапрограммирование](./macros.md) | [Продвинутые темы](./advanced.md) →

Вопросы 96–100 из 100.

## Builder pattern

Для структур со многими полями, часть опциональна. Реализация: отдельный тип-билдер, методы с цепочечным возвратом self. Для обязательных полей — typestate или build() -> Result. Повышает читаемость вызовов.

```rust
struct ClientBuilder { url: Option<String>, timeout_ms: u32 }
impl ClientBuilder {
    fn new() -> Self { Self { url: None, timeout_ms: 1000 } }
    fn url(mut self, u: impl Into<String>) -> Self { self.url = Some(u.into()); self }
    fn build(self) -> Result<Client, &'static str> {
        Ok(Client { url: self.url.ok_or("url required")?, timeout_ms: self.timeout_ms })
    }
}
```

## Error handling: anyhow vs thiserror

**thiserror** — для библиотечных ошибок. derive для enum с std::error::Error, источниками и сообщениями. **anyhow** — для приложений, сбор ошибок из разных источников с контекстом. Правило: внутри библиотеки — thiserror + осмысленный enum. В main и интеграционном слое — anyhow.

## Дизайн API на трейтах

Скрывать реализацию: impl Trait в возвращаемой позиции. Принимать обобщённо: impl AsRef<Path>, impl IntoIterator. Не светить лишние generics. Минимальный набор методов в трейте — легче реализовывать и мокать.

## Semver и breaking changes

Patch — совместимые исправления. Minor — добавление API. Major — breaking changes. Breaking: удаление/переименование публичного элемента, изменение сигнатуры, новые обязательные методы в pub трейте. Не breaking: новые pub функции, новые варианты в #[non_exhaustive] enum. Помечайте enum как #[non_exhaustive] с первой версии.

## Cargo workspace

Для проектов, переросших один крейт. Типовая структура: core (бизнес-логика, без IO), db (хранилище), http (транспорт), bin (точка входа). Общие версии зависимостей — в [workspace.dependencies]. Ускоряет инкрементальную сборку, упрощает тестирование.

```toml
[workspace]
members = ["core", "db", "http", "bin"]

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
```

---

← [Макросы и метапрограммирование](./macros.md) | [Продвинутые темы](./advanced.md) →

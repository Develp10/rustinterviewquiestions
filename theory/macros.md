# Макросы и метапрограммирование

← [Производительность](./performance.md) | [Архитектура и дизайн API](./architecture.md) →

Вопросы 90–95 из 100.

## Декларативные макросы (macro_rules!)

Набор правил matcher → transcriber. Фрагменты: expr, ident, ty, tt, stmt, pat. Применяется до тайпчека. Для DSL, устранения бойлерплейта, конструкторов коллекций.

```rust
macro_rules! hashmap {
    ($($k:expr => $v:expr),* $(,)?) => {{
        let mut m = std::collections::HashMap::new();
        $( m.insert($k, $v); )*
        m
    }};
}
```

## Гигиена макросов

Декларативные макросы гигиеничны для локальных имён — let x внутри не перекрывает x снаружи. Не гигиеничны для типов и трейтов. В путях используйте $crate::path::to::Item вместо просто Item.

## Процедурные макросы — три вида

- **Function-like**: вызываются как make_thing!(). 
- **Derive**: #[derive(Foo)] на структуре или enum.  
- **Attribute**: #[my_attr] модифицирует элемент.

Все три: TokenStream → TokenStream. Крейты: syn (парсинг), quote (генерация), proc-macro-error (ошибки). proc-macro увеличивает время сборки — применять когда macro_rules! не хватает.

## Build script (build.rs)

Запускается перед сборкой крейта. Генерация кода, подключение C-библиотек (cc), генерация констант. Вывод: cargo:rerun-if-changed=, cargo:rustc-cfg=, cargo:rustc-env=. OUT_DIR — для generated файлов.

```rust
// build.rs
fn main() {
    println!("cargo:rerun-if-changed=schema.proto");
    println!("cargo:rustc-env=BUILD_VERSION=1.0.0");
}
```

## const fn

Вычисляется в compile time. Для констант, размеров массивов, статической логики. Ограничения постепенно снимаются: if/match, циклы — разрешены. Аллокации — ограничены. Многие методы std уже const.

```rust
const fn square(n: u32) -> u32 { n * n }
const N: u32 = square(7); // вычислено при компиляции
```

## Type-state pattern

Состояние объекта закодировано в типе. Невозможные переходы не компилируются. Реализация через generic-параметры + PhantomData. Применяется в API протоколов, builder-ах с обязательными полями.

```rust
struct Open; struct Closed;
struct Socket<S> { _s: std::marker::PhantomData<S> }

impl Socket<Closed> { fn open(self) -> Socket<Open> { todo!() } }
impl Socket<Open>   { fn send(&self, _data: &[u8]) {} }
```

---

← [Производительность](./performance.md) | [Архитектура и дизайн API](./architecture.md) →

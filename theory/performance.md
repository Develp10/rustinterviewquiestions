# Производительность

← [Unsafe, FFI, низкий уровень](./unsafe-ffi.md) | [Макросы и метапрограммирование](./macros.md) →

Вопросы 79–89 из 100.

## Профилирование

Linux: perf + flamegraph. cargo-flamegraph — svg из flamegraph. Аллокации: heaptrack, dhat, крейт dhat. Async: tokio-console. Микробенчмарки: criterion. Сначала измерить, найти горячее — потом исправлять.

## Inlining

#[inline] — подсказка для библиотечных функций (без него межкрейтовый инлайн невозможен). #[inline(always)] — для тонких обёрток над интринзиками. #[inline(never)] — для холодного кода (error reporting). Не переусердствуйте с always.

## Аллокации в горячем коде

Каждая аллокация — поход в аллокатор. Решения: буферы с clear вместо new, String::with_capacity / Vec::with_capacity, SmallVec для коротких коллекций, возврат &str / Cow вместо String, collect_into / extend_from_slice.

## Vec и рост

Удвоение при переполнении. with_capacity(n) — одна аллокация. shrink_to_fit — освободить лишнее. Для переиспользуемых буферов держать ёмкость, вызывать clear.

## String, str и UTF-8

String — владеющий UTF-8 буфер. &str — срез. Индексация по байтам, а не символам (паника на середине Unicode). chars().count() — O(n) по байтам. Длину в символах кэшируйте.

## HashMap и хеш DOS

SipHash 1-3 с рандомизацией — защита от hash flooding. Для внутренних карт: ahash, fxhash (2–3x быстрее, не криптостойкие). На API-границе с пользователем — SipHash.

## Branch prediction

#[cold] на функциях редких случаев. likely/unlikely через крейт likely_stable. Горячий путь — прямой, без вложенных match. match со многими вариантами → jump table лучше цепочки if-else.

## Cache-friendly структуры

Vec<T> с маленькими T быстрее Vec<Box<T>>. SoA (struct of arrays) быстрее AoS при работе с подмножеством полей. SmallVec, tinyvec — данные на стеке, меньше промахов кэша.

## Monomorphization cost

Каждый дженерик-инстанс раздувает бинарник и время компиляции. Паттерн: тонкая generic обёртка + толстая non-generic реализация через &str / &[u8] / &dyn.

## LTO и codegen-units

LTO (thin/fat) — оптимизации через границы крейтов. +5–15% скорости, дольше линкование. codegen-units=1 — больше контекста для оптимизатора. PGO даёт +5–20% на реальной нагрузке.

## Criterion

De facto стандарт микробенчмарков. black_box блокирует оптимизацию до константы. Сохраняет результаты для сравнения между прогонами. Запуск: cargo bench.

---

← [Unsafe, FFI, низкий уровень](./unsafe-ffi.md) | [Макросы и метапрограммирование](./macros.md) →

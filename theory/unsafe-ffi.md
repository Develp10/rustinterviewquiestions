# Unsafe, FFI, низкий уровень

← [Async и runtime](./async-runtime.md) | [Производительность](./performance.md) →

Вопросы 63–78 из 100.

## Что разрешает unsafe

Пять вещей: разыменование сырого указателя, вызов unsafe-функции, доступ к mutable static, реализация unsafe-трейта, обращение к union. Borrow checker и типобезопасность — остаются.

## Undefined behavior

UB — поведение без гарантий. Компилятор оптимизирует, предполагая что UB нет. Частые источники: гонка данных через unsafe, висячий указатель, aliasing ссылок (*mut + & одновременно), чтение неинициализированной памяти, невалидные значения примитивов (bool = 2).

## Сырые указатели vs ссылки

*const T, *mut T — без гарантий валидности и алиасинга, без лайфтайма, могут быть null. Ссылки — с гарантиями. Сырые указатели: FFI, реализации коллекций, низкоуровневая работа с памятью.

## MaybeUninit

Обёртка для легальной работы с неинициализированной памятью. mem::uninitialized — UB для типов с непустыми инвариантами. MaybeUninit обходит это — компилятор знает «внутри может быть мусор».

## UnsafeCell

Единственный легальный способ получить &mut T через &T. Основа всей interior mutability. Cell, RefCell, Mutex, Atomic — построены на UnsafeCell.

## FFI и repr(C)

По умолчанию layout структуры в Rust не определён. #[repr(C)] — поля в порядке объявления с C padding. Для enum: #[repr(C)] или #[repr(u32)]. String, Vec — нельзя передавать напрямую. Только сырой указатель + длина.

## extern fn и паника через FFI

extern "C" задаёт calling convention. Паника через FFI — UB. Оборачивайте в std::panic::catch_unwind. В Rust 2021+ unwinding через extern "C" — abort.

## bindgen / cbindgen / cxx

bindgen — Rust биндинги из C заголовков. cbindgen — C заголовки из Rust. cxx — безопасный интероп с C++, проверка совместимости на этапе сборки.

## repr(transparent)

Структура с одним непрозрачным полем имеет то же ABI и layout. Для newtype-паттернов в FFI и для interior mutability типов.

## no_std

Без стандартной библиотеки. Только core и (опционально) alloc. Embedded, ядра ОС, WASM рантаймы. Нужен #[panic_handler]. Без аллокатора — heapless.

## Inline assembly

asm! — ассемблер в Rust. Для интринзиков без crate, управления регистрами. Использовать только после измерений.

## SIMD

core::arch — платформенно-специфичные интринзики. std::simd — nightly. Крейты wide, packed_simd. Проще — автовекторизация LLVM (регулярные циклы без зависимостей).

## Allocator API

#[global_allocator] — глобальная замена аллокатора (jemalloc, mimalloc, snmalloc). Allocator trait — локальный аллокатор для одной коллекции. Нередко даёт 5–15% ускорения без изменения логики.

## mem::transmute

Побитовая реинтерпретация. Безопасна только при одинаковом layout и валидном значении в новом типе. Альтернативы: from_bits/to_bits, bytemuck::cast, zerocopy. Изолируйте в unsafe fn с документированными инвариантами.

## Volatile операции

read_volatile / write_volatile — без оптимизации/реордеринга компилятором. Для memory-mapped регистров устройств. Не заменяет atomics для многопоточной синхронизации.

## Sound unsafe API

unsafe fn — документировать инварианты (Safety: раздел в doc-комментарии). Safe fn с unsafe внутри — гарантировать, что никакая комбинация валидных аргументов не приведёт к UB. Проверять через Miri.

---

← [Async и runtime](./async-runtime.md) | [Производительность](./performance.md) →

# Продвинутые темы (Staff / Expert)

← [Архитектура и дизайн API](./architecture.md)

Вопросы A1–A21. Для тех, кто уже свободно ориентируется в основных 100 вопросах.

## A1. GAT (Generic Associated Types)

Ассоциированные типы, параметризованные лайфтаймами или типами. Решают проблему LendingIterator — итератора, возвращающего ссылку на самого себя. До стабилизации (Rust 1.65) невозможно было написать без unsafe.

## A2. HRTB и for<'a>

Higher-Ranked Trait Bound: for<'a> F: Fn(&'a T). «F реализует Fn для любого лайфтайма». Без него компилятор фиксирует конкретный 'a на сайте объявления. Обычно выводится автоматически через elision.

## A3. Variance

Ковариантность, контравариантность, инвариантность. &'a T — ковариантна по 'a и T. &'a mut T — ковариантна по 'a, инвариантна по T. fn(T)->U — контравариантна по T, ковариантна по U. PhantomData контролирует variance unsafe-структур.

## A4. Лайфтайм subtyping

'a: 'b — 'a живёт не короче 'b. Основа проверок assignability ссылок. В трейтах с GAT: where Self: 'a. В async-футурах сложные HRTB + subtyping могут давать загадочные ошибки.

## A5. *mut T vs *const T и variance

*mut T — инвариантен по T. *const T — ковариантен. NonNull<T> — ковариантен. Для правильной variance unsafe-обёртки используйте PhantomData.

## A6. Pin и async fn

Pin<P> — гарантия, что значение не переместится. Нужно для self-referential типов (стейт-машины async fn). Unpin — auto trait, означает «безопасно перемещать». Стейт-машины async fn обычно не Unpin.

## A7. Структурная pin projection и pin-project

Pin<&mut Outer> → Pin<&mut Field> — только если автор Outer взял обязательство. Крейт pin-project генерирует безопасные проекции. #[pin] поле → Pin<&mut Field>, без — &mut Field.

## A8. Cancel safety

Future отменяется дропом. Cancel-safe: прогресс теряется, но состояние согласовано. read_exact — не cancel-safe (частично потребляет буфер). В select! — только cancel-safe операции или spawn для изоляции.

## A9. Стейт-машина async fn

Анонимная enum-подобная структура: один вариант на await-точку + начало + конец. Локальные переменные через await → поля структуры. Размер — сумма «толстейших» путей. Send: если любое поле !Send — future !Send.

## A10. async fn в трейтах

RPITIT (Rust 2024) — стабильно, но без object safety для dyn. async-trait крейт — аллокация на вызов. #[trait_variant] для Send-bound на dyn. Для горячих путей без dyn — нативный async fn в трейте.

## A11. Stacked Borrows / Tree Borrows и Miri

Stacked Borrows — формальная модель алиасинга. Miri проверяет по ней. Создание новой ссылки кладёт тег на стек; использование «выкинутого» тега — UB. Tree Borrows — более либеральная замена. cargo +nightly miri test — запустить.

## A12. Memory ordering на atomic

Relaxed — только атомарность. Acquire/Release — happens-before пара. AcqRel — для RMW. SeqCst — глобальный порядок. Ошибка: Relaxed декремент Arc без финального Acquire → data race.

## A13. Loom

Model checker конкурентного кода. Перебирает все возможные interleaving-и. RUSTFLAGS="--cfg loom" cargo test. Тесты — маленькие (2-3 потока). Не подменяет Miri: loom — конкурентность, Miri — aliasing UB.

## A14. Niche optimization

Niche — значение, которое тип не принимает. Option<&T> = размер указателя. Option<NonZeroU32> без накладных. #[repr(transparent)] сохраняет niche. #[repr(C)] обычно теряет.

## A15. Custom allocator и GlobalAlloc

GlobalAlloc: alloc, dealloc — unsafe трейт. Allocator trait (nightly) — локально для одной коллекции. Применения: real-time без mmap, счётчики аллокаций, arena, jemalloc/mimalloc.

## A16. Dropck и #[may_dangle]

Dropck требует, чтобы в момент дропа все ссылки внутри были живы. Иногда отвергает корректные программы. #[may_dangle] (nightly) — «не трогаю это поле в drop». Так сделан Vec<T> и Box<T>.

## A17. Sealed trait

Публичный трейт, который нельзя реализовать снаружи крейта. mod private { pub trait Sealed {} } + pub trait MyTrait: private::Sealed. Гарантирует, что добавление методов не сломает сторонний код.

## A18. Type-state pattern (детали)

Typestate через generic-параметры + PhantomData. Методы, валидные только в одном состоянии, доступны только в том типе. Для общих методов — impl<S: SomeBound> MyType<S>. Применяется в протоколах, OAuth-флоу, pipeline-ах.

## A19. mem::transmute и trait objects

transmute побитово реинтерпретирует. &dyn Trait — толстый указатель (data, vtable), layout не стабилен между версиями rustc. Безопасные альтернативы: from_bits/to_bits, bytemuck::cast, zerocopy. Изолируйте с assert_eq!(size_of...) и Safety-комментарием.

## A20. Send/Sync soundness

unsafe impl Send/Sync — вы берёте ответственность. Send: перемещение между потоками без UB, деструктор корректен в любом потоке. Sync: разделённый доступ без гонок. Всегда документируйте обоснование рядом с unsafe impl.

## A21. Lock-free SPSC ring buffer

Два AtomicUsize: head и tail. Producer: читает tail (Relaxed), читает head (Acquire), пишет данные, store tail (Release). Consumer: симметрично. False sharing: head и tail на разных кеш-линиях — CachePadded. ABA в SPSC отсутствует.

---

← [Архитектура и дизайн API](./architecture.md)

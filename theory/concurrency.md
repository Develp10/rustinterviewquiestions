# Конкурентность и параллелизм

← [Типы, трейты, обобщения](./types-traits-generics.md) | [Async и runtime](./async-runtime.md) →

Вопросы 31–45 из 100.

## Send и Sync

`Send` — значение можно передать в другой поток. `Sync` — `&T` можно делить между потоками. Оба — auto traits. Не `Send`: `Rc`, `Cell`, `RefCell`.

## Mutex vs RwLock

`Mutex` — один поток. `RwLock` — один писатель или много читателей. `RwLock` дороже, оправдан при долгих чтениях. По умолчанию — `Mutex`.

## Poisoning у Mutex

Паника под `Mutex` делает лок «отравленным». `lock()` возвращает `Err`. В `parking_lot::Mutex` poisoning отсутствует.

## std::sync::mpsc

Многопроизводитель / один потребитель. Неограниченная очередь. Для bounded: `crossbeam-channel` или `tokio::sync::mpsc`.

## Scoped threads

`std::thread::scope` гарантирует завершение потоков до выхода — разрешает заимствовать стек без `'static`.

## Atomic и memory ordering

Relaxed — только атомарность. Acquire/Release — happens-before пара. SeqCst — глобальный порядок. Для lock-free: Release (store) + Acquire (load).

## Гонка данных vs race condition

**Гонка данных** — два потока без синхронизации, хотя бы один пишет. В безопасном Rust невозможна. **Race condition** — логическая ошибка по времени.

## Deadlock

Разные порядки захвата локов. Решения: фиксированный порядок, try_lock, минимизация секций. В async — не держать лок через .await.

## Rayon

Work-stealing пул для CPU-bound задач. par_iter() — главный интерфейс. Не подходит для IO.

## Work stealing

Каждый поток — собственная deque. Берёт из своей головы, крадёт с хвоста чужой. Rayon, Tokio, async-std.

## Barrier

Синхронизирует группу: все ждут пока не соберутся все, затем продолжают одновременно.

## Condvar

Поток ждёт условия с отпусканием лока, другой будит. Проверяйте условие в while (spurious wakeups).

## Spinlock

Крутится в цикле вместо блокировки. Эффективен при очень коротких секциях. Обычно parking_lot::Mutex предпочтительнее.

## Thread local

thread_local! — отдельный экземпляр в каждом потоке. Для per-thread кэшей, статистики, RNG.

---

← [Типы, трейты, обобщения](./types-traits-generics.md) | [Async и runtime](./async-runtime.md) →

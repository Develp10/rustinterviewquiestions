# Rust Interview Deep Dive

Репозиторий с разбором 40 задач с собеседований Rust-разработчика уровня senior и staff. Каждая задача оформлена как разбор от практикующего разработчика: условие, наивная попытка, объяснение почему компилятор недоволен или почему рантайм проседает, корректное решение с тестами, и где уместно — бенчмарк или `cargo asm`.

Это не "угадай вывод программы". Это сценарии, которые встречаются в проде: lock-free структуры, self-referential типы в async, FFI с тензорными библиотеками, корректный `Send` на гардах через await, memory ordering под `loom`, soundness кастомных коллекций.

## Структура

```
rust-interview-deepdive/
├── README.md
├── CONTRIBUTING.md
├── LICENSE
├── Cargo.toml                  # workspace
├── .github/workflows/ci.yml    # cargo test + clippy + fmt + miri
├── tasks/
│   ├── 01-ownership-move/
│   │   ├── README.md           # условие, разбор, подводные камни
│   │   ├── src/lib.rs          # реализация
│   │   ├── tests/it.rs         # интеграционные тесты
│   │   └── benches/bench.rs    # где уместно
│   └── ... до 40/
└── theory/
    ├── memory-model.md
    ├── send-sync.md
    ├── pin-unpin.md
    ├── async-runtime.md
    └── unsafe-invariants.md
```

## Уровни сложности

- middle — задачи 1–10, типовые вопросы про владение и трейты
- - senior — задачи 11–30, конкурентность, async, unsafe, производительность
  - - staff — задачи 31–40, дизайн API, тёмные углы компилятора, макросы
   
    - ## Как работать с репозиторием
   
    - ```bash
      git clone https://github.com/Develp10/rustinterviewquiestions.git
      cd rustinterviewquiestions
      cargo test --workspace
      cargo test -p task-17       # отдельная задача
      cargo bench -p task-26      # бенчмарк, если есть
      ```

      Для задач с unsafe прогоняется miri:

      ```bash
      cargo +nightly miri test -p task-22
      ```

      ## Список задач

      ### Владение и заимствования

      1. Почему `Vec<String>` нельзя клонировать через `&self` без `Clone`, и как `Cow<'_, str>` меняет дизайн публичного API.
      2. 2. Реализовать `split_at_mut` для своего типа сначала без `unsafe`, потом через `slice::from_raw_parts_mut`, обосновать инвариант алиасинга.
         3. 3. NLL и Polonius: пример, который не компилируется в Rust 2018, но работает в 2021.
            4. 4. Reborrow `&mut T -> &mut T`: когда явный reborrow `&mut *x` обязателен.
               5. 5. Порядок Drop в структурах и кортежах, влияние на RAII-гарды вроде `MutexGuard`.
                 
                  6. ### Типы, дженерики, трейты
                 
                  7. 6. Когерентность и orphan rule: почему нельзя `impl Display for Vec<MyType>`, обход через newtype.
                     7. 7. `dyn Trait` против `impl Trait`: размер, vtable, object safety, ограничение `Self: Sized`.
                        8. 8. GATs на примере `LendingIterator`, чем он отличается от обычного `Iterator`.
                           9. 9. Higher-Ranked Trait Bounds `for<'a> Fn(&'a T)`: почему обычный лайфтайм-параметр не подходит для замыканий.
                              10. 10. Вариативность лайфтаймов: ковариантность, контравариантность, инвариантность, разбор `PhantomData<fn(T)>`.
                                 
                                  11. ### Конкурентность
                                 
                                  12. 11. `Send` и `Sync` руками: написать `unsafe impl` для собственного lock-free стека и доказать корректность.
                                      12. 12. `Arc<Mutex<T>>` против `Arc<RwLock<T>>` против `parking_lot`: когда какой выбрать, цена справедливости.
                                          13. 13. mpsc в Tokio и std: backpressure, что происходит при переполнении канала.
                                              14. 14. Spinlock на `AtomicBool` с правильными `Ordering`, почему `Relaxed` ломает корректность.
                                                  15. 15. Memory ordering: воспроизвести race, который ловится только под `loom`.
                                                     
                                                      16. ### Async
                                                     
                                                      17. 16. Future как стейт-машина: руками раскрутить `async fn` в `enum` состояний.
                                                          17. 17. `Pin<&mut T>`: зачем, и как `!Unpin` self-referential структуры используют его.
                                                              18. 18. Send-граница на async-функциях, история с `MutexGuard` через await.
                                                                  19. 19. Cancel safety: почему `tokio::select!` требует cancellation-safe веток.
                                                                      20. 20. Свой минимальный executor на `Waker` и `VecDeque<Task>`.
                                                                         
                                                                          21. ### Unsafe и FFI
                                                                         
                                                                          22. 21. Инварианты `&T`: aliasing, dereferenceability, validity, что ломает UB.
                                                                              22. 22. `MaybeUninit<T>` правильно: построение массива поэлементно без `transmute`.
                                                                                  23. 23. `repr(C)` и `repr(transparent)`: когда что использовать для FFI.
                                                                                      24. 24. Передача замыкания в C через `extern "C" fn` и `*mut c_void` (trampoline-приём).
                                                                                          25. 25. Soundness своего Vec-подобного контейнера: capacity, length, drop, ZST.
                                                                                             
                                                                                              26. ### Производительность
                                                                                             
                                                                                              27. 26. `Vec::with_capacity` против `Vec::new`: amortized cost, замер в `cargo bench`.
                                                                                                  27. 27. `Box<dyn Trait>` против enum dispatch: branch prediction и inlining.
                                                                                                      28. 28. `String`, `&str`, `Cow<'_, str>` в публичных API библиотек.
                                                                                                          29. 29. False sharing: `#[repr(align(64))]` и `crossbeam::utils::CachePadded`.
                                                                                                              30. 30. `HashMap` с `ahash` против дефолтного `SipHash`: где DoS-стойкость важна, а где платим за неё впустую.
                                                                                                                 
                                                                                                                  31. ### Идиомы и дизайн API
                                                                                                                 
                                                                                                                  32. 31. Typestate pattern: HTTP-клиент, который компилируется только при правильной последовательности вызовов.
                                                                                                                      32. 32. Builder с обязательными полями через типы, а не через `Option`.
                                                                                                                          33. 33. Error handling: `thiserror` для библиотек, `anyhow` для бинарей, почему не наоборот.
                                                                                                                              34. 34. Sealed trait через приватный супертрейт.
                                                                                                                                  35. 35. Extension trait: добавить метод к `Result<T, E>` из чужого крейта.
                                                                                                                                     
                                                                                                                                      36. ### Тёмные углы стдлайбы и компилятора
                                                                                                                                     
                                                                                                                                      37. 36. `Iterator::fold` против `for`: где LLVM не справляется с автовекторизацией.
                                                                                                                                          37. 37. `Drop` и `mem::forget`: ABA-проблема при работе с `Vec::set_len`.
                                                                                                                                              38. 38. Оператор `?` и `From`: как работает coercion, зачем нужен `Result<_, Box<dyn Error>>`.
                                                                                                                                                  39. 39. Const generics: `Matrix<const R: usize, const C: usize>` с перемножением, проверяемым на компиляции.
                                                                                                                                                      40. 40. Гигиена макросов: написать `macro_rules!`, который не ломается, если пользователь определил свою `Option`.
                                                                                                                                                         
                                                                                                                                                          41. ## Шаблон отдельной задачи
                                                                                                                                                         
                                                                                                                                                          42. Каждая задача в `tasks/NN-slug/README.md` оформлена единообразно:
                                                                                                                                                         
                                                                                                                                                          43. - условие
                                                                                                                                                              - - наивная попытка с кодом
                                                                                                                                                                - - что говорит компилятор и почему
                                                                                                                                                                  - - рабочее решение
                                                                                                                                                                    - - когда что выбирать в реальном коде
                                                                                                                                                                      - - где это встречается в проде (tokio, hyper, rayon и т.д.)
                                                                                                                                                                        - - ссылки на Rustonomicon, Reference, RFC
                                                                                                                                                                         
                                                                                                                                                                          - ## Дорожная карта
                                                                                                                                                                         
                                                                                                                                                                          - Сейчас 40 задач. Дальше будут разделы:
                                                                                                                                                                         
                                                                                                                                                                          - - `tokio-uring` и io_uring
                                                                                                                                                                            - - кастомные аллокаторы (`GlobalAlloc`, `Allocator API`)
                                                                                                                                                                              - - интеграция с CUDA через `cust` и кастомные kernels
                                                                                                                                                                                - - профилирование через `perf`, `flamegraph`, `cargo-pgo`
                                                                                                                                                                                  - - разбор внутренностей популярных крейтов: `tokio`, `hyper`, `axum`, `serde`
                                                                                                                                                                                   
                                                                                                                                                                                    - ## Как контрибьютить
                                                                                                                                                                                   
                                                                                                                                                                                    - PR с новыми задачами приветствуются, особенно из реальных собеседований и из ML-инфраструктуры, где Rust встречается с C++, Python и GPU. Шаблон в `scripts/new_task.sh`. Перед PR прогоните `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`.
                                                                                                                                                                                   
                                                                                                                                                                                    - ## Лицензия
                                                                                                                                                                                   
                                                                                                                                                                                    - MIT OR Apache-2.0 на выбор.
                                                                                                                                                                                    - 

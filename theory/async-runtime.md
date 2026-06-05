# Async и runtime

← [Конкурентность и параллелизм](./concurrency.md) | [Unsafe, FFI, низкий уровень](./unsafe-ffi.md) →

Вопросы 46–62 из 100.

## async fn и стейт-машина

async fn — синтаксический сахар над impl Future. Тело превращается в стейт-машину. Каждое .await — точка приостановки. Локальные переменные через .await становятся полями машины. При вызове работа не начинается — возвращается Future.

## Future и poll

Future имеет один метод poll(Pin<&mut Self>, &mut Context) -> Poll<Output>. Если готово — Ready(value), иначе — Pending с сохранённым Waker. Рантайм не опрашивает в цикле — реагирует на пробуждения.

## Tokio vs async-std

Tokio — де-факто стандарт. Большой набор примитивов, work-stealing планировщик, экосистема. async-std — legacy. Для embedded: smol, embassy.

## Executor и reactor

Executor опрашивает futures. Reactor регистрирует IO-события в ядре и будит Wakers. В Tokio — через mio. Разделение ответственности объясняет большинство странностей async.

## Кооперативная многозадачность

Один поток — много задач. Пока задача не отдала управление через await, другие стоят. Блокирующий вызов в async — использовать tokio::task::spawn_blocking.

## tokio::select!

Ждёт нескольких futures, продолжает при первой готовой. Остальные дропаются — cancel safety важна. Biased обрабатывает ветви сверху вниз.

## Cancel safety

Future «отменяется» дропом. Cancel-safe future корректна при дропе в любой точке. read_exact — не cancel-safe (частично потребляет буфер). В select! использовать только cancel-safe операции.

## tokio::spawn и JoinHandle

spawn отправляет future в рантайм и возвращает JoinHandle. Задача начинает работать сразу. Дроп хендла — задача продолжается в фоне. abort() прерывает задачу.

## JoinSet

Группа задач. join_next() ждёт первой завершившейся. Удобнее FuturesUnordered для tokio-задач.

## async trait

В трейтах async fn работает через RPITIT (Rust 2024). Для dyn — async-trait крейт (аллокация на вызов) или #[trait_variant]. Send-bound для dyn в Send-контексте — #[trait_variant::make(Send)].

## Backpressure

Bounded канал (tokio::sync::mpsc с capacity) — send ждёт места. Автоматически тормозит продюсера. Без этого быстрый продюсер переполняет память.

## Streams

Асинхронный Iterator. poll_next -> Poll<Option<Item>>. Адаптеры: map, filter, try_next, chunks, throttle (крейты futures, tokio_stream).

## pin_project

Генерирует безопасные проекции полей через Pin. Без него — unsafe + Pin::map_unchecked_mut. Поле с #[pin] проектируется как Pin<&mut F>, без — как &mut F.

## Executor budget (Tokio)

Каждой задаче начисляется бюджет poll-операций. При исчерпании — кооперативный yield. Защита от монополизации потока. Явный yield: tokio::task::yield_now().

## LocalSet

Запуск !Send futures на одном потоке. Для интеграции с не-Send библиотеками. spawn_local вместо spawn.

## Timeout

tokio::time::timeout оборачивает future. Result<T, Elapsed>. При Err future дропается — учитывайте cancel safety.

## Структурированная конкурентность

Дочерние задачи завершаются до выхода породившей функции. В std: thread::scope. В async: JoinSet, токен отмены (CancellationToken).

---

← [Конкурентность и параллелизм](./concurrency.md) | [Unsafe, FFI, низкий уровень](./unsafe-ffi.md) →

## 02. Lock-free стек на атомиках

**Уровень:** Senior  
**Темы:** `AtomicPtr`, `compare_exchange`, memory ordering, ABA-проблема

---

### Условие

Реализуйте потокобезопасный стек без мьютексов. Операции `push` и `pop` должны работать корректно при конкурентном вызове из произвольного числа потоков. Никаких `Mutex`, `RwLock`, `parking_lot`.

```rust
pub struct TreiberStack<T> {
    // ваша реализация
}

impl<T> TreiberStack<T> {
    pub fn new() -> Self { todo!() }
    pub fn push(&self, val: T) { todo!() }
    pub fn pop(&self) -> Option<T> { todo!() }
}
```

Бонус: объясните, почему наивная реализация через `compare_exchange` уязвима к ABA-проблеме и что с этим делать.

---

### Наивная попытка

```rust
use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr;

pub struct Stack<T> {
    head: AtomicPtr<Node<T>>,
}

struct Node<T> {
    val: T,
    next: *mut Node<T>,
}

impl<T> Stack<T> {
    pub fn push(&self, val: T) {
        let node = Box::into_raw(Box::new(Node { val, next: ptr::null_mut() }));
        loop {
            let head = self.head.load(Ordering::Relaxed);
            unsafe { (*node).next = head; }
            // Проблема: Relaxed везде — нет happens-before с pop
            if self.head.compare_exchange(head, node, Ordering::Relaxed, Ordering::Relaxed).is_ok() {
                break;
            }
        }
    }
}
```

Проблем две. Первая — неправильные ordering: `Relaxed` не создаёт синхронизацию, другой поток может читать старые данные узла. Вторая — утечка памяти и UB при `pop`: освобождённый узел мог быть прочитан параллельным потоком.

---

### Разбор

**Стек Трайбера** — классический lock-free алгоритм. Голова стека хранится как атомарный указатель. `push` делает новый узел и пытается установить его как голову через CAS. `pop` читает голову, берёт следующий узел и пытается через CAS сдвинуть голову.

**Memory ordering для CAS:**
- `push`: успешный `compare_exchange` — `Release`, иначе `Relaxed`. Release гарантирует, что запись в `next` видна потоку, который прочитает голову через `Acquire`.
- `pop`: `load` — `Acquire`. Это формирует пару с `Release` из `push` и гарантирует видимость данных узла.
- Провальный `compare_exchange` может быть `Relaxed` — нам всё равно, мы перечитаем.

**ABA-проблема:** поток A читает голову P1, засыпает. Потоки B и C делают `pop`/`push`, и P1 оказывается снова на вершине с другим `next`. Поток A просыпается, CAS видит тот же адрес, считает всё ок — но `next` уже не тот. Итог: потерянные элементы.

Решения: epoch-based reclamation (crossbeam-epoch), hazard pointers, tagged pointers (версионирование адреса в битах).

---

### Рабочее решение

```rust
use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr;

pub struct TreiberStack<T> {
    head: AtomicPtr<Node<T>>,
}

struct Node<T> {
    val: T,
    next: *mut Node<T>,
}

unsafe impl<T: Send> Send for TreiberStack<T> {}
unsafe impl<T: Send> Sync for TreiberStack<T> {}

impl<T> TreiberStack<T> {
    pub fn new() -> Self {
        Self { head: AtomicPtr::new(ptr::null_mut()) }
    }

    pub fn push(&self, val: T) {
        let node = Box::into_raw(Box::new(Node { val, next: ptr::null_mut() }));
        loop {
            let head = self.head.load(Ordering::Relaxed);
            unsafe { (*node).next = head; }
            // Release: запись в node.next должна быть видна тому, кто сделает Acquire-load
            match self.head.compare_exchange_weak(head, node, Ordering::Release, Ordering::Relaxed) {
                Ok(_) => break,
                Err(_) => continue,
            }
        }
    }

    pub fn pop(&self) -> Option<T> {
        loop {
            // Acquire: видим данные узла, записанные с Release в push
            let head = self.head.load(Ordering::Acquire);
            if head.is_null() { return None; }
            let next = unsafe { (*head).next };
            match self.head.compare_exchange_weak(head, next, Ordering::Acquire, Ordering::Relaxed) {
                Ok(_) => {
                    // Только мы владеем этим узлом — безопасно забрать значение
                    let val = unsafe { ptr::read(&(*head).val) };
                    // Освобождаем память без вызова Drop на val (уже вынули)
                    unsafe { std::alloc::dealloc(head as *mut u8, std::alloc::Layout::new::<Node<T>>()) };
                    return Some(val);
                }
                Err(_) => continue,
            }
        }
    }
}

impl<T> Drop for TreiberStack<T> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}
```

---

### Тесты

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn single_thread() {
        let s = TreiberStack::new();
        s.push(1); s.push(2); s.push(3);
        assert_eq!(s.pop(), Some(3));
        assert_eq!(s.pop(), Some(2));
        assert_eq!(s.pop(), Some(1));
        assert_eq!(s.pop(), None);
    }

    #[test]
    fn concurrent_pushes() {
        let stack = Arc::new(TreiberStack::new());
        let mut handles = vec![];
        for i in 0..8 {
            let s = stack.clone();
            handles.push(thread::spawn(move || {
                for j in 0..1000 { s.push(i * 1000 + j); }
            }));
        }
        for h in handles { h.join().unwrap(); }
        let mut count = 0;
        while stack.pop().is_some() { count += 1; }
        assert_eq!(count, 8000);
    }

    #[test]
    fn concurrent_push_pop() {
        let stack = Arc::new(TreiberStack::<i32>::new());
        let producers: Vec<_> = (0..4).map(|_| {
            let s = stack.clone();
            thread::spawn(move || { for i in 0..500 { s.push(i); } })
        }).collect();
        let consumers: Vec<_> = (0..4).map(|_| {
            let s = stack.clone();
            thread::spawn(move || {
                let mut got = 0;
                for _ in 0..500 { if s.pop().is_some() { got += 1; } }
                got
            })
        }).collect();
        for h in producers { h.join().unwrap(); }
        let _total: i32 = consumers.into_iter().map(|h| h.join().unwrap()).sum();
        // Проверяем отсутствие паники и UB (Miri покрывает остальное)
    }
}
```

Запускайте под Miri: `cargo +nightly miri test` — он поймает use-after-free и неправильный aliasing.

---

### Подводные камни

- `compare_exchange_weak` может давать spurious failures на ARM — поэтому всегда в цикле.
- Освобождение памяти через `dealloc` без вызова `drop`: если `T` реализует `Drop`, нужно сначала `ptr::drop_in_place`. В нашем случае мы делаем `ptr::read` (перемещение), поэтому drop вызовет владелец значения.
- `unsafe impl Send/Sync`: необходимо, потому что `*mut Node<T>` снимает автовывод. Обоснование: доступ к данным всегда через CAS, никакого aliasing.
- Без epoch-based reclamation код уязвим к ABA. Для продакшна используйте `crossbeam::stack::SegQueue` или `crossbeam-deque`.

---

### Где встречается в проде

- Внутренности `crossbeam`, `tokio` (injection queue)
- LMAX Disruptor-подобные очереди в HFT
- Аллокаторы с per-thread кэшами (jemalloc tcache, mimalloc)

---

### Ссылки

- [Treiber Stack (Wikipedia)](https://en.wikipedia.org/wiki/Treiber_stack)  
- [crossbeam-epoch](https://docs.rs/crossbeam-epoch)  
- Rust Atomics and Locks, глава 4 (Mara Bos)

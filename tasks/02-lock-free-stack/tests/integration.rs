//! Интеграционные тесты для задачи 02: TreiberStack (lock-free)
//!
//! Запуск: cargo test -p lock-free-stack
//! Miri:   cargo +nightly miri test -p lock-free-stack

use lock_free_stack::TreiberStack;
use std::sync::Arc;
use std::thread;

#[test]
fn concurrent_pushes_count() {
    let stack = Arc::new(TreiberStack::new());
    let threads: Vec<_> = (0..8)
        .map(|i| {
            let s = Arc::clone(&stack);
            thread::spawn(move || {
                for j in 0..1000 {
                    s.push(i * 1000 + j);
                }
            })
        })
        .collect();
    for t in threads {
        t.join().unwrap();
    }
    let mut count = 0;
    while stack.pop().is_some() {
        count += 1;
    }
    assert_eq!(count, 8000);
}

#[test]
fn concurrent_push_pop_no_loss() {
    let stack = Arc::new(TreiberStack::<i32>::new());

    let producers: Vec<_> = (0..4)
        .map(|_| {
            let s = Arc::clone(&stack);
            thread::spawn(move || {
                for i in 0..500 {
                    s.push(i);
                }
            })
        })
        .collect();

    let consumers: Vec<_> = (0..4)
        .map(|_| {
            let s = Arc::clone(&stack);
            thread::spawn(move || {
                let mut count = 0usize;
                for _ in 0..1000 {
                    if s.pop().is_some() {
                        count += 1;
                    }
                }
                count
            })
        })
        .collect();

    for t in producers {
        t.join().unwrap();
    }
    let consumed: usize = consumers.into_iter().map(|t| t.join().unwrap()).sum();

    // Всё что осталось — дочитаем
    let mut remaining = 0usize;
    while stack.pop().is_some() {
        remaining += 1;
    }

    assert_eq!(consumed + remaining, 2000, "должно быть ровно 4*500 элементов");
}

#[test]
fn drop_with_non_copy_elements() {
    let stack = TreiberStack::new();
    for _ in 0..100 {
        stack.push(String::from("test"));
    }
    // drop(stack) вызывается здесь — проверяем отсутствие утечек и double-free
}

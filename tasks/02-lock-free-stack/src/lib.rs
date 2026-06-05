//! # Задача 02: Lock-free стек (Treiber Stack)
//!
//! Потокобезопасный стек без мьютексов на AtomicPtr + CAS.
//!
//! Запуск тестов: cargo test -p lock-free-stack
//! Miri:          cargo +nightly miri test -p lock-free-stack
//! Loom:          RUSTFLAGS="--cfg loom" cargo test -p lock-free-stack

use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

struct Node<T> {
    val: T,
    next: *mut Node<T>,
}

/// Потокобезопасный lock-free стек (алгоритм Трайбера).
///
/// # Safety
/// Реализует Send + Sync, потому что доступ к данным всегда через CAS,
/// а освобождение узла происходит только у единственного победившего потока.
pub struct TreiberStack<T> {
    head: AtomicPtr<Node<T>>,
}

unsafe impl<T: Send> Send for TreiberStack<T> {}
unsafe impl<T: Send> Sync for TreiberStack<T> {}

impl<T> TreiberStack<T> {
    pub fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
        }
    }

    /// Добавить элемент в стек. Потокобезопасно.
    pub fn push(&self, val: T) {
        let node = Box::into_raw(Box::new(Node {
            val,
            next: ptr::null_mut(),
        }));
        loop {
            let head = self.head.load(Ordering::Relaxed);
            // SAFETY: мы только что выделили node, никто его не читает ещё
            unsafe { (*node).next = head }
            // Release: запись в node.val / node.next видна через Acquire в pop
            match self.head.compare_exchange_weak(
                head,
                node,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(_) => continue,
            }
        }
    }

    /// Извлечь элемент с вершины стека.
    pub fn pop(&self) -> Option<T> {
        loop {
            // Acquire: видим данные узла, записанные с Release в push
            let head = self.head.load(Ordering::Acquire);
            if head.is_null() {
                return None;
            }
            // SAFETY: head не null, мы держим Acquire, данные валидны
            let next = unsafe { (*head).next };
            match self.head.compare_exchange_weak(
                head,
                next,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    // SAFETY: только мы выиграли CAS — единственные владельцы узла
                    let val = unsafe { ptr::read(&(*head).val) };
                    // Освобождаем память без вызова Drop на val (уже переместили)
                    unsafe {
                        std::alloc::dealloc(
                            head as *mut u8,
                            std::alloc::Layout::new::<Node<T>>(),
                        )
                    };
                    return Some(val);
                }
                Err(_) => continue,
            }
        }
    }

    /// Проверить, пуст ли стек.
    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Relaxed).is_null()
    }
}

impl<T> Default for TreiberStack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for TreiberStack<T> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn push_pop_lifo() {
        let s = TreiberStack::new();
        s.push(1i32);
        s.push(2);
        s.push(3);
        assert_eq!(s.pop(), Some(3));
        assert_eq!(s.pop(), Some(2));
        assert_eq!(s.pop(), Some(1));
        assert_eq!(s.pop(), None);
    }

    #[test]
    fn empty_stack_returns_none() {
        let s: TreiberStack<i32> = TreiberStack::new();
        assert!(s.is_empty());
        assert_eq!(s.pop(), None);
    }

    #[test]
    fn drop_non_copy_type() {
        // String реализует Drop — проверяем отсутствие double-free
        let s = TreiberStack::new();
        s.push(String::from("hello"));
        s.push(String::from("world"));
        // drop(s) — должен пройти без паники
    }
}

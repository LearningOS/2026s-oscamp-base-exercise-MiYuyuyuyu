//! # Mutex Shared State
//!
//! In this exercise, you will use `Arc<Mutex<T>>` to safely share and modify data between multiple threads.
//!
//! ## Concepts
//! - `Mutex<T>` mutex protects shared data
//! - `Arc<T>` atomic reference counting enables cross-thread sharing
//! - `lock()` acquires the lock and accesses data

use std::sync::{Arc, Mutex};
use std::thread;

/// Increment a counter concurrently using `n_threads` threads.
/// Each thread increments the counter `count_per_thread` times.
/// Returns the final counter value.
///
/// Hint: Use `Arc<Mutex<usize>>` as the shared counter.
pub fn concurrent_counter(n_threads: usize, count_per_thread: usize) -> usize {
    // TODO: Create Arc<Mutex<usize>> with initial value 0
    // TODO: Spawn n_threads threads
    // TODO: In each thread, lock() and increment count_per_thread times
    // TODO: Join all threads, return final value
    let val = Arc::new(Mutex::new(0));
    let mut threads = vec![];
    for i in 0..n_threads {
        let temp_val = val.clone();
        let temp_sp = std::thread::Builder::new()
            .name(format!("thread : {}",i))
            .spawn(move||{
                *temp_val.lock().unwrap() += count_per_thread;
            })
            .unwrap();
        threads.push(temp_sp);
    }
    for i in threads {
        i.join().unwrap();
    }
    let result = *val.lock().unwrap();
    result
}

/// Add elements to a shared vector concurrently using multiple threads.
/// Each thread pushes its own id (0..n_threads) to the vector.
/// Returns the sorted vector.
///
/// Hint: Use `Arc<Mutex<Vec<usize>>>`.
pub fn concurrent_collect(n_threads: usize) -> Vec<usize> {
    // TODO: Create Arc<Mutex<Vec<usize>>>
    // TODO: Each thread pushes its own id
    // TODO: After joining all threads, sort the result and return
    let val: Arc<Mutex<Vec<usize>>> = Arc::new(Mutex::new(vec![]));
    let mut threads = vec![];
    for i in 0..n_threads {
        let temp_val = val.clone();
        let temp_th = std::thread::Builder::new()
            .name(format!("thread : {}",i))
            .spawn(move||{
                temp_val.lock().unwrap().push(i);
            }).unwrap();
        threads.push(temp_th);
    }
    for i in threads {
        i.join().unwrap();
    }
    let mut x = val.lock().unwrap().clone();
    x.sort();
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_single_thread() {
        assert_eq!(concurrent_counter(1, 100), 100);
    }

    #[test]
    fn test_counter_multi_thread() {
        assert_eq!(concurrent_counter(10, 100), 1000);
    }

    #[test]
    fn test_counter_zero() {
        assert_eq!(concurrent_counter(5, 0), 0);
    }

    #[test]
    fn test_collect() {
        let result = concurrent_collect(5);
        assert_eq!(result, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_collect_single() {
        assert_eq!(concurrent_collect(1), vec![0]);
    }
}

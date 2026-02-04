// Rust 代码样本 - 用于 Valhalla 训练

/// 计算斐波那契数列
fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

/// 快速排序
fn quicksort<T: Ord>(arr: &mut [T]) {
    if arr.len() <= 1 {
        return;
    }
    let pivot = partition(arr);
    quicksort(&mut arr[..pivot]);
    quicksort(&mut arr[pivot + 1..]);
}

fn partition<T: Ord>(arr: &mut [T]) -> usize {
    let len = arr.len();
    let pivot_index = len / 2;
    arr.swap(pivot_index, len - 1);
    let mut store_index = 0;
    for i in 0..len - 1 {
        if arr[i] < arr[len - 1] {
            arr.swap(i, store_index);
            store_index += 1;
        }
    }
    arr.swap(store_index, len - 1);
    store_index
}

/// 二叉树节点
struct TreeNode<T> {
    value: T,
    left: Option<Box<TreeNode<T>>>,
    right: Option<Box<TreeNode<T>>>,
}

impl<T> TreeNode<T> {
    fn new(value: T) -> Self {
        TreeNode {
            value,
            left: None,
            right: None,
        }
    }
    
    fn insert(&mut self, value: T) where T: Ord {
        if value < self.value {
            match self.left {
                Some(ref mut node) => node.insert(value),
                None => self.left = Some(Box::new(TreeNode::new(value))),
            }
        } else {
            match self.right {
                Some(ref mut node) => node.insert(value),
                None => self.right = Some(Box::new(TreeNode::new(value))),
            }
        }
    }
}

/// 异步 HTTP 客户端
async fn fetch_data(url: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    Ok(body)
}

/// 并发处理
use std::sync::{Arc, Mutex};
use std::thread;

fn parallel_sum(numbers: Vec<i32>) -> i32 {
    let sum = Arc::new(Mutex::new(0));
    let chunk_size = numbers.len() / 4;
    let mut handles = vec![];
    
    for chunk in numbers.chunks(chunk_size) {
        let sum = Arc::clone(&sum);
        let chunk = chunk.to_vec();
        let handle = thread::spawn(move || {
            let partial: i32 = chunk.iter().sum();
            let mut total = sum.lock().unwrap();
            *total += partial;
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    *sum.lock().unwrap()
}

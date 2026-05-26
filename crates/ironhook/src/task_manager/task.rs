use std::sync::{Mutex, LazyLock};

// 1. 定义任务的状态（状态机）
#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,   // 等待目标库加载
    Hooking,   // 正在执行 Hook
    Hooked,    // Hook 成功
    Failed,    // Hook 失败
}

// 2. 定义任务的实体
#[derive(Debug, Clone)]
pub struct HookTask {
    pub lib_name: String,       // 目标动态库名，例如 "libart.so"
    pub sym_name: String,       // 目标函数名，例如 "open"
    pub proxy_addr: usize,      // 我们自己写的代理函数的地址
    pub status: TaskStatus,     // 当前状态
}

// 3. 创建一个全局的、线程安全的任务队列！
// LazyLock 是 Rust 1.80+ 的新特性，不需要任何第三方库就能创建全局静态变量
// Mutex 保证了多线程同时 Hook 时的绝对安全
pub static TASK_QUEUE: LazyLock<Mutex<Vec<HookTask>>> = LazyLock::new(|| Mutex::new(Vec::new()));

// 4. 对外暴露的 API：添加 Hook 任务
pub fn add_hook_task(lib_name: &str, sym_name: &str, proxy_addr: usize) -> usize {
    // 获取全局锁（C++ 里很容易忘了解锁，但在 Rust 里，锁会在函数结束时自动释放！）
    let mut queue = TASK_QUEUE.lock().unwrap();
    let task_id = queue.len(); // 用当前队列长度作为任务 ID
    
    let task = HookTask {
        lib_name: lib_name.to_string(),
        sym_name: sym_name.to_string(),
        proxy_addr,
        status: TaskStatus::Pending,
    };
    
    queue.push(task);
    println!("✅ 成功添加任务: [{}] {}", lib_name, sym_name);
    
    task_id
}

// ==========================================
// 5. 本地单元测试：假装我们是多线程的高并发应用
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_concurrent_add_task() {
        let mut handles = vec![];

        // 模拟开 5 个线程，同时向队列里塞任务
        for i in 0..5 {
            let handle = thread::spawn(move || {
                add_hook_task("libc.so", &format!("open_variant_{}", i), 0x1000 + i);
            });
            handles.push(handle);
        }

        // 等待所有线程执行完毕
        for handle in handles {
            handle.join().unwrap();
        }

        // 检查队列里的任务数是不是正好 5 个
        let queue = TASK_QUEUE.lock().unwrap();
        assert_eq!(queue.len(), 5);
        println!("🎉 队列中共有 {} 个任务，高并发测试通过！没有任何数据竞争！", queue.len());
    }
}
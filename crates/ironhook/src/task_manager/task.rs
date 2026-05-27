use std::sync::{Mutex, LazyLock};
// 引入你写的 linker 模块！
use super::linker::resolve_symbol;

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    Hooked,
    Failed(String), // 带上具体的失败原因，方便队友排查！
}

#[derive(Debug, Clone)]
pub struct HookTask {
    pub lib_name: String,
    pub sym_name: String,
    pub proxy_addr: usize,
    pub status: TaskStatus,
}

pub static TASK_QUEUE: LazyLock<Mutex<Vec<HookTask>>> = LazyLock::new(|| Mutex::new(Vec::new()));

pub fn add_hook_task(lib_name: &str, sym_name: &str, proxy_addr: usize) -> usize {
    let mut queue = TASK_QUEUE.lock().unwrap();
    let task_id = queue.len();
    queue.push(HookTask {
        lib_name: lib_name.to_string(),
        sym_name: sym_name.to_string(),
        proxy_addr,
        status: TaskStatus::Pending,
    });
    task_id
}

// ==========================================
// 🚀 终极杀器：状态机驱动引擎（给队友调用的总控）
// 每次调用这个函数，它会自动检查所有 Pending 的任务，去找地址并下发 Hook
pub fn process_pending_tasks() {
    let mut queue = TASK_QUEUE.lock().unwrap();
    
    for task in queue.iter_mut() {
        if task.status == TaskStatus::Pending {
            println!("🔍 正在处理挂起任务: {}::{}", task.lib_name, task.sym_name);
            
            // 调用你写的 Linker 去找绝对地址
            if let Some(target_addr) = resolve_symbol(&task.lib_name, &task.sym_name) {
                println!("✅ 找到真实地址 {:#x}，开始呼叫队友...", target_addr);
                
                // 模拟呼叫队友的执行函数
                match mock_execute_hook(target_addr, task.proxy_addr) {
                    Ok(_) => {
                        println!("🎉 队友执行成功！状态变更为 Hooked");
                        task.status = TaskStatus::Hooked;
                    }
                    Err(e) => {
                        println!("❌ 队友搞砸了: {}", e);
                        task.status = TaskStatus::Failed(e);
                    }
                }
            } else {
                println!("⏳ 没找到 {}，可能是库还没加载，继续挂起等待...", task.lib_name);
            }
        }
    }
}

// 模拟其他队友的底层 Hook 模块
fn mock_execute_hook(target_addr: usize, _proxy_addr: usize) -> Result<(), String> {
    // 假装队友 C（内存跳板模块）在干活
    println!("   -> [队友C] 正在对地址 {:#x} 执行 mprotect 解锁内存...", target_addr);
    // 假装队友 B（指令重写模块）在干活
    println!("   -> [队友B] 正在解析机器码，写入跳板指令...");
    Ok(())
}

// ==========================================
// 🎯 联合测试：测试整套链路
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_pipeline() {
        // 1. 用户下发一个任务 (借用我们前面测试成功的 Linux libc)
        add_hook_task("libc.so.6", "puts", 0x9999);
        add_hook_task("libnot_exist.so", "fake_func", 0x8888); // 搞个假的测试健壮性

        // 2. 模拟系统主循环，定期驱动任务
        println!("\n=== 第一次轮询 ===");
        process_pending_tasks();

        println!("\n=== 第二次轮询 ===");
        process_pending_tasks();
        
        // 3. 检查队列最终状态
        let queue = TASK_QUEUE.lock().unwrap();
        assert_eq!(queue[0].status, TaskStatus::Hooked);
        assert_eq!(queue[1].status, TaskStatus::Pending); // 假的那个应该一直 Pending
    }
}
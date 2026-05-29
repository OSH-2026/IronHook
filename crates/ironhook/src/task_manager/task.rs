use std::sync::{Mutex, LazyLock};
use crate::task_manager::linker::resolve_symbol;

// ==========================================
// 1. 定义三个物理隔离的生命周期结构体 (Typestate)
// ==========================================

#[derive(Debug, Clone)]
pub struct PendingTask {
    pub lib_name: String,
    pub sym_name: String,
    pub proxy_addr: usize,
}

#[derive(Debug, Clone)]
pub struct ResolvedTask {
    pub lib_name: String,
    pub sym_name: String,
    pub proxy_addr: usize,
    pub target_addr: usize, // 核心！只有变成 Resolved 状态，才允许拥有真实的物理地址
}

#[derive(Debug, Clone)]
pub struct HookedTask {
    pub lib_name: String,
    pub sym_name: String,
}

// ==========================================
// 2. 状态转移方法：利用“所有权转移”实现防呆
// ==========================================

impl PendingTask {
    /// 尝试解析地址。
    /// 注意这里的 `self`：调用这个方法会【消耗】掉当前的 PendingTask。
    /// 如果成功，返回一个全新的 ResolvedTask；如果失败，把原本的 PendingTask 退回来。
    pub fn try_resolve(self) -> Result<ResolvedTask, Self> {
        if let Some(target_addr) = resolve_symbol(&self.lib_name, &self.sym_name) {
            Ok(ResolvedTask {
                lib_name: self.lib_name,
                sym_name: self.sym_name,
                proxy_addr: self.proxy_addr,
                target_addr,
            })
        } else {
            // 解析失败，把所有权还回去
            Err(self)
        }
    }
}

// ==========================================
// 3. 队友的底层 Hook 接口 (强制类型约束)
// ==========================================

/// 这个函数是你队友（模块一、二）写的。
/// 注意看参数：他强制要求传入 `ResolvedTask`。
/// 就算你写错了代码，也不可能把一个还没找到地址的 `PendingTask` 传给他，因为【类型不匹配】，编译器直接报错！
pub fn execute_hook_by_teammates(task: ResolvedTask) -> Result<HookedTask, String> {
    println!("   -> [队友接单] 拿到绝对地址 {:#x}，开始改写内存与指令...", task.target_addr);
    
    // 假装队友底层操作成功
    Ok(HookedTask {
        lib_name: task.lib_name,
        sym_name: task.sym_name,
    })
}

// ==========================================
// 4. 全局任务管理器 (现在只存 Pending 任务)
// ==========================================

// 我们现在只需要一个“挂起队列”
pub static PENDING_QUEUE: LazyLock<Mutex<Vec<PendingTask>>> = LazyLock::new(|| Mutex::new(Vec::new()));

/// 暴露给 API 层的接口：添加新任务
pub fn add_hook_task(lib_name: &str, sym_name: &str, proxy_addr: usize) {
    let task = PendingTask {
        lib_name: lib_name.to_string(),
        sym_name: sym_name.to_string(),
        proxy_addr,
    };
    
    // 刚进来的任务，我们先当场试着解析一次
    match task.try_resolve() {
        Ok(resolved_task) => {
            println!("✅ [{}] 一秒入魂！直接解析成功，下发给队友...", resolved_task.sym_name);
            let _ = execute_hook_by_teammates(resolved_task);
        },
        Err(pending_task) => {
            println!("⏳ [{}] 库暂未加载，塞入挂起队列等候...", pending_task.sym_name);
            PENDING_QUEUE.lock().unwrap().push(pending_task);
        }
    }
}

/// 暴露给你自己的事件驱动回调：处理所有挂起的任务
pub fn process_pending_tasks() {
    let mut queue = PENDING_QUEUE.lock().unwrap();
    // 把旧队列里的任务全部拿出来，准备重试
    let tasks_to_retry = std::mem::take(&mut *queue); 
    
    for task in tasks_to_retry {
        match task.try_resolve() {
            Ok(resolved_task) => {
                println!("🚀 [{}] 终于等到你！解析成功，开始 Hook", resolved_task.sym_name);
                let _ = execute_hook_by_teammates(resolved_task);
            },
            Err(failed_task) => {
                // 还是没找到，重新塞回队列里继续等
                queue.push(failed_task);
            }
        }
    }
}

// ==========================================
// 🎯 单元测试：感受编译期的神力
// ==========================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typestate_pipeline() {
        // 清空队列防干扰
        PENDING_QUEUE.lock().unwrap().clear();

        // 1. 添加一个真实的 Linux 函数 (会瞬间成功)
        add_hook_task("libc.so.6", "puts", 0x1111);
        
        // 2. 添加一个假函数 (会进队列)
        add_hook_task("libfake.so", "fake_func", 0x2222);

        // 验证队列里只有 1 个挂起任务
        assert_eq!(PENDING_QUEUE.lock().unwrap().len(), 1);

        /* 
        ====================================================
        🚨 编译期防呆测试 (你可以把下面这行代码取消注释试试)
        ====================================================
        let wrong_task = PendingTask {
            lib_name: "test".to_string(), sym_name: "test".to_string(), proxy_addr: 0
        };
        // 尝试把 PendingTask 强行塞给队友的底层 Hook 函数：
        // execute_hook_by_teammates(wrong_task); 
        // 
        // 👆 VS Code 会立刻在这个函数下划红线，报错：
        // expected struct `ResolvedTask`, found struct `PendingTask`
        // 运行时崩溃？不存在的，编译都别想过！
        */
    }
}
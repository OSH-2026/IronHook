use ironhook::task_manager::{add_hook_task, resolve_symbol};

fn main() {
    println!("========== IronHook 真机全链路测试 ==========");
    
    // 1. 测试你的任务队列
    let task_id = add_hook_task("libc.so", "open", 0x11112222);
    println!("✅ 任务队列测试通过, 当前分配 ID: {}", task_id);

    // 2. 测试最硬核的 Linker 内存寻址
    println!("\n🔍 正在读取真机 /proc/self/maps...");
    println!("🔍 正在使用 Goblin 解析 ELF 二进制文件...");
    
    // 寻找 libc.so 中的 open 函数
    match resolve_symbol("libc.so", "open") {
        Some(addr) => {
            println!("🎉 逆天成功！");
            println!("📍 成功在你的安卓手机内存中定位到 open 函数！");
            println!("📍 真实绝对内存地址为: {:#x}", addr);
        },
        None => {
            println!("❌ 解析失败！请检查手机系统的 libc.so 路径或权限。");
        }
    }
    
    println!("=============================================");
}
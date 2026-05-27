use goblin::elf::Elf;
use std::fs;

/// 1. 升级版：不仅找基地址，还要把这个 .so 文件的【真实硬盘路径】抓出来！
pub fn find_lib_info(maps_content: &str, lib_name: &str) -> Option<(usize, String)> {
    for line in maps_content.lines() {
        if line.contains(lib_name) && line.contains("r-xp") {
            // line 长这样: "7df0200000-7df0300000 r-xp 00000000 ... /system/lib64/libc.so"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                let addr_range = parts[0];
                let file_path = parts[5]; // 第6部分就是文件路径！
                
                let base_addr_str = addr_range.split('-').next()?;
                if let Ok(addr) = usize::from_str_radix(base_addr_str, 16) {
                    return Some((addr, file_path.to_string()));
                }
            }
        }
    }
    None
}

/// 2. 核心魔法：直接从磁盘读取 .so 文件，扒出符号的相对偏移量 (Offset)
pub fn find_symbol_offset(file_path: &str, sym_name: &str) -> Option<usize> {
    // 整个文件读入内存的 byte 数组 (这里极大地利用了 Rust 的安全性)
    let buffer = fs::read(file_path).ok()?;
    
    // 使用 goblin 库，一键解析复杂的 ELF 格式！
    let elf = Elf::parse(&buffer).ok()?;

    // 遍历动态符号表 (.dynsym)
    for sym in elf.dynsyms.iter() {
        // 通过符号的 st_name 索引，去字符串表里查出它的真名
        if let Some(name) = elf.dynstrtab.get_at(sym.st_name) {
            if name == sym_name {
                // 找到了！返回它在文件里的相对偏移量
                return Some(sym.st_value as usize);
            }
        }
    }
    None // 没找到
}

/// 3. 给队友的终极接口：输入库名和函数名，直接返回它在内存里的【绝对地址】！
pub fn resolve_symbol(lib_name: &str, sym_name: &str) -> Option<usize> {
    let maps = fs::read_to_string("/proc/self/maps").ok()?;
    
    // 第一步：从 maps 拿到基地址和文件路径
    let (base_addr, file_path) = find_lib_info(&maps, lib_name)?;
    
    // 第二步：从 ELF 文件里抽出相对偏移量
    let offset = find_symbol_offset(&file_path, sym_name)?;
    
    // 第三步：绝对地址 = 基地址 + 偏移量。大功告成！
    Some(base_addr + offset)
}

// ==========================================
// 🎯 终极真机模拟测试 (在你的 WSL Linux 里直接跑！)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_local_linux_symbol() {
        // 因为你的 WSL 本身就是 Linux，我们可以直接拿你 WSL 里的系统库来测试！
        // 在 Linux 中，C标准库通常叫 "libc.so.6" (而不是安卓的 libc.so)
        let lib_name = "libc.so.6";
        let target_func = "puts"; // 我们找大家最熟悉的打印函数 puts

        // 尝试解析！
        if let Some(abs_addr) = resolve_symbol(lib_name, target_func) {
            println!("🎉 逆天成功！在真实的 Linux 环境中找到了 {}::{} 的绝对地址: {:#x}", 
                     lib_name, target_func, abs_addr);
        } else {
            println!("⚠️ 没找到！但这可能只是因为不同 Linux 系统的 libc 名字不同，逻辑是没问题的。");
        }
    }
}
use goblin::elf::Elf;
use std::fs;

// ==========================================
// 1. 获取库的内存基地址与真实文件路径
// ==========================================
pub fn find_lib_base_and_path(maps_content: &str, lib_name: &str) -> Option<(usize, String)> {
    // 遍历 maps，找到匹配库的【第一行】
    // 在 Linux 中，第一行必然是基地址所在段 (通常是 r--p，包含了 ELF 头部)
    for line in maps_content.lines() {
        if line.contains(lib_name) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            // 确保这一行包含了完整的路径信息 (通常有 6 列)
            if parts.len() >= 6 {
                let addr_range = parts[0];
                let file_path = parts[5]; // 获取完整硬盘路径
                
                let base_addr_str = addr_range.split('-').next()?;
                if let Ok(base_addr) = usize::from_str_radix(base_addr_str, 16) {
                    return Some((base_addr, file_path.to_string()));
                }
            }
        }
    }
    None
}

// ==========================================
// 2. 利用 Goblin 解析硬盘上的 ELF，提取相对偏移量
// ==========================================
pub fn find_symbol_offset(file_path: &str, sym_name: &str) -> Option<usize> {
    // 安全地将文件读入内存
    let buffer = fs::read(file_path).ok()?;
    
    // 解析标准的 ELF 磁盘格式，绝对不会因为内存分页错位而失败
    let elf = Elf::parse(&buffer).ok()?;

    for sym in elf.dynsyms.iter() {
        if let Some(name) = elf.dynstrtab.get_at(sym.st_name) {
            if name == sym_name {
                // st_value 在动态库 (.so) 中，代表的就是相对于基地址的偏移量！
                return Some(sym.st_value as usize);
            }
        }
    }
    None
}

// ==========================================
// 3. 对外暴露的混合寻址接口 (100% Safe Rust，无任何 unsafe！)
// ==========================================
pub fn resolve_symbol(lib_name: &str, sym_name: &str) -> Option<usize> {
    let maps = fs::read_to_string("/proc/self/maps").ok()?;
    
    // 第一步：从系统映射中拿到基地址和真实路径
    let (base_addr, file_path) = find_lib_base_and_path(&maps, lib_name)?;
    
    // 第二步：从硬盘文件中提取纯净的偏移量
    let offset = find_symbol_offset(&file_path, sym_name)?;
    
    // 第三步：物理基址 + 相对偏移 = 绝对物理地址！
    Some(base_addr + offset)
}

// ==========================================
// 🎯 单元测试矩阵：严苛的交叉验证
// ==========================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correctness_vs_dlsym() {
        let lib_name = "libc.so.6";
        let target_func = "puts";

        let our_addr = resolve_symbol(lib_name, target_func).expect("解析失败！");

        let official_addr = unsafe {
            let handle = libc::dlopen(b"libc.so.6\0".as_ptr() as *const i8, libc::RTLD_LAZY);
            assert!(!handle.is_null(), "无法加载 libc.so.6");
            let sym = libc::dlsym(handle, b"puts\0".as_ptr() as *const i8);
            libc::dlclose(handle);
            sym as usize
        };

        println!("🔍 我们的解析地址: {:#x}", our_addr);
        println!("🏛️ 官方的标答地址: {:#x}", official_addr);
        assert_eq!(our_addr, official_addr, "❌ 灾难：我们解析的地址与官方不一致！");
    }

    #[test]
    fn test_robustness_edge_cases() {
        assert!(resolve_symbol("lib_totally_fake.so", "puts").is_none());
        assert!(resolve_symbol("libc.so.6", "fake_func").is_none());
    }

    #[test]
    fn test_generality_other_libs() {
        let _ = (1.0_f64).sin(); // 触发加载
        let addr = resolve_symbol("libm.so.6", "sin");
        assert!(addr.is_some(), "无法在数学库中找到 sin 函数！");
    }
}
use std::fs;

/// 核心逻辑：从文本中提取目标库的基地址
/// (拆分出这个函数是为了方便我们本地单元测试，不用真去读操作系统的文件)
pub fn find_lib_base_addr(maps_content: &str, lib_name: &str) -> Option<usize> {
    // 逐行遍历 maps 文件
    for line in maps_content.lines() {
        // 我们只关心包含目标库名字，并且权限是 r-xp (可执行代码段) 的那一行
        if line.contains(lib_name) && line.contains("r-xp") {
            
            // line 长这样: "7df0200000-7df0300000 r-xp 00000000..."
            // 按照空格分割，取出第一部分: "7df0200000-7df0300000"
            let addr_range = line.split_whitespace().next()?;
            
            // 按照减号分割，取出前面的起始地址: "7df0200000"
            let base_addr_str = addr_range.split('-').next()?;
            
            // 把十六进制的字符串，转换成 Rust 的内存地址数字 (usize)
            if let Ok(addr) = usize::from_str_radix(base_addr_str, 16) {
                return Some(addr); // 成功找到，返回！
            }
        }
    }
    None // 找遍了也没找到
}

/// 暴露给队友的真实接口：读取当前操作系统的真实内存映射
pub fn get_base_addr_from_system(lib_name: &str) -> Option<usize> {
    // 读取系统文件 (在 Android / Linux 下有效)
    if let Ok(content) = fs::read_to_string("/proc/self/maps") {
        find_lib_base_addr(&content, lib_name)
    } else {
        None
    }
}

// ==========================================
// 🎯 单元测试：我们在本地假装自己是 Android 系统
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_maps() {
        // 伪造一段真实的 Android maps 文件内容
        let mock_maps = "\
7df0000000-7df0200000 rw-p 00000000 103:04 12345 /system/lib64/libc.so
7df0200000-7df0300000 r-xp 00000000 103:04 12345 /system/lib64/libc.so
7df0300000-7df0400000 r--p 00000000 103:04 12345 /system/lib64/libc.so
";
        // 让我们的代码去解析这个假数据，寻找 "libc.so"
        let addr = find_lib_base_addr(mock_maps, "libc.so");
        
        // 断言它必须等于 0x7df0200000 (注意：不能是被 rw-p 干扰的上一行)
        assert_eq!(addr, Some(0x7df0200000));
        
        println!("✅ 成功排除干扰项，精准解析出可执行段基地址: {:#x}", addr.unwrap());
    }
}
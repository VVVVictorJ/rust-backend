//! 过滤 A 股 ST、*ST 等风险管理类证券简称。

/// 将简称开头的 `*`、`S`、`T` 全角字母等规整为 ASCII，直到遇到首个「非前缀」字符为止。
fn st_markers_prefix_normalized(name: &str) -> String {
    let mut out = String::new();
    for c in name.trim().chars() {
        match c {
            '*' | '＊' | '﹡' => out.push('*'),
            'S' | 's' | 'Ｓ' | 'ｓ' => out.push('S'),
            'T' | 't' | 'Ｔ' | 'ｔ' => out.push('T'),
            _ => break,
        }
        if out.len() >= 5 {
            break;
        }
    }
    out
}

/// `*ST`、`ST`、`S*ST`、`SST` 等前缀形态（跳过 ST 类）。
pub fn is_st_special_stock_name(name: &str) -> bool {
    let p = st_markers_prefix_normalized(name);
    p.starts_with("*ST") || p.starts_with("S*ST") || p.starts_with("SST") || p.starts_with("ST")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_st_prefixed_common_forms() {
        assert!(is_st_special_stock_name("*ST钒钛"));
        assert!(is_st_special_stock_name("＊ST钒钛"));
        assert!(is_st_special_stock_name("ST钒钛"));
        assert!(is_st_special_stock_name("st钒钛")); // ascii lower
        assert!(is_st_special_stock_name("S*ST股"));
        assert!(is_st_special_stock_name("S＊ST股"));
        assert!(is_st_special_stock_name("SST股"));
        assert!(is_st_special_stock_name("  SST股"));
    }

    #[test]
    fn accepts_normal_cn_names() {
        assert!(!is_st_special_stock_name("浦发银行"));
        assert!(!is_st_special_stock_name("东方财富"));
        assert!(!is_st_special_stock_name("")); // Edge
    }

    #[test]
    fn st_in_middle_only_is_not_filtered() {
        // 仅以常见「前缀标记」判定，避免误判名称中间含英文字母的品种
        assert!(!is_st_special_stock_name("某ST字样在中间"));
        assert!(!is_st_special_stock_name("科特估"));
    }
}

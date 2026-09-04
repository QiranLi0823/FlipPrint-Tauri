//! 双面打印算法模块
//!
//! 计算手动双面打印的页面顺序

use serde::{Deserialize, Serialize};

/// 双面打印计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplexPlan {
    /// 第一遍打印的页面（偶数页倒序）
    pub first_pass: Vec<usize>,
    /// 第二遍打印的页面（奇数页正序）
    pub second_pass: Vec<usize>,
    /// 需要的纸张数
    pub sheet_count: usize,
    /// PDF 总页数
    pub page_count: usize,
}

/// 计算双面打印计划
///
/// # 算法说明
///
/// 对于 N 页 PDF：
/// - **第一遍**：打印偶数页，按倒序排列（N-1, N-3, ..., 4, 2）
/// - **第二遍**：打印奇数页，按正序排列（1, 3, 5, ..., N）
/// - **纸张数**：(N + 1) / 2（向上取整）
///
/// # 示例
///
/// N = 5 时：
/// - 第一遍：4, 2
/// - 第二遍：1, 3, 5
/// - 纸张数：3
///
/// 最终效果（翻页后阅读顺序正确）：
/// - 纸张 1：4 / 1
/// - 纸张 2：2 / 3
/// - 纸张 3：5 / 空白
pub fn calculate_duplex(page_count: usize) -> DuplexPlan {
    if page_count == 0 {
        return DuplexPlan {
            first_pass: vec![],
            second_pass: vec![],
            sheet_count: 0,
            page_count: 0,
        };
    }

    // 第一遍：偶数页倒序 (N, N-2, ..., 4, 2)
    let first_pass: Vec<usize> = (1..=page_count)
        .filter(|&x| x % 2 == 0)  // 过滤出偶数
        .rev()                     // 倒序
        .collect();

    // 第二遍：奇数页正序 (1, 3, 5, ..., N)
    let second_pass: Vec<usize> = (1..=page_count)
        .filter(|&x| x % 2 == 1)  // 过滤出奇数
        .collect();

    // 纸张数：向上取整
    let sheet_count = (page_count + 1) / 2;

    DuplexPlan {
        first_pass,
        second_pass,
        sheet_count,
        page_count,
    }
}

/// 获取页面顺序的字符串表示
pub fn format_page_order(pages: &[usize]) -> String {
    pages
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join(" → ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplex_1_page() {
        let plan = calculate_duplex(1);
        assert_eq!(plan.first_pass, vec![]);
        assert_eq!(plan.second_pass, vec![1]);
        assert_eq!(plan.sheet_count, 1);
    }

    #[test]
    fn test_duplex_2_pages() {
        let plan = calculate_duplex(2);
        assert_eq!(plan.first_pass, vec![2]);
        assert_eq!(plan.second_pass, vec![1]);
        assert_eq!(plan.sheet_count, 1);
    }

    #[test]
    fn test_duplex_3_pages() {
        let plan = calculate_duplex(3);
        assert_eq!(plan.first_pass, vec![2]);
        assert_eq!(plan.second_pass, vec![1, 3]);
        assert_eq!(plan.sheet_count, 2);
    }

    #[test]
    fn test_duplex_4_pages() {
        let plan = calculate_duplex(4);
        assert_eq!(plan.first_pass, vec![4, 2]);
        assert_eq!(plan.second_pass, vec![1, 3]);
        assert_eq!(plan.sheet_count, 2);
    }

    #[test]
    fn test_duplex_5_pages() {
        let plan = calculate_duplex(5);
        assert_eq!(plan.first_pass, vec![4, 2]);
        assert_eq!(plan.second_pass, vec![1, 3, 5]);
        assert_eq!(plan.sheet_count, 3);
    }

    #[test]
    fn test_duplex_6_pages() {
        let plan = calculate_duplex(6);
        assert_eq!(plan.first_pass, vec![6, 4, 2]);
        assert_eq!(plan.second_pass, vec![1, 3, 5]);
        assert_eq!(plan.sheet_count, 3);
    }

    #[test]
    fn test_duplex_10_pages() {
        let plan = calculate_duplex(10);
        assert_eq!(plan.first_pass, vec![10, 8, 6, 4, 2]);
        assert_eq!(plan.second_pass, vec![1, 3, 5, 7, 9]);
        assert_eq!(plan.sheet_count, 5);
    }

    #[test]
    fn test_duplex_0_pages() {
        let plan = calculate_duplex(0);
        assert_eq!(plan.first_pass, vec![]);
        assert_eq!(plan.second_pass, vec![]);
        assert_eq!(plan.sheet_count, 0);
    }

    #[test]
    fn test_format_page_order() {
        assert_eq!(format_page_order(&[4, 2]), "4 → 2");
        assert_eq!(format_page_order(&[1, 3, 5]), "1 → 3 → 5");
        assert_eq!(format_page_order(&[]), "");
    }
}

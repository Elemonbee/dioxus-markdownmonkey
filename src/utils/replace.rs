//! 文本替换工具 / Text replace utilities

use regex::{Regex, RegexBuilder};

/// 构建正则表达式 / Build regex from query
pub fn build_regex(query: &str, case_insensitive: bool) -> Result<Regex, regex::Error> {
    let mut builder = RegexBuilder::new(query);
    builder.case_insensitive(case_insensitive);
    builder.build()
}

/// 统计匹配数量 / Count matches
pub fn count_matches(content: &str, query: &str, case_insensitive: bool, use_regex: bool) -> usize {
    if query.is_empty() {
        return 0;
    }
    if use_regex {
        if let Ok(re) = build_regex(query, case_insensitive) {
            re.find_iter(content).count()
        } else {
            0
        }
    } else {
        let (search_content, search_query) = if case_insensitive {
            (content.to_lowercase(), query.to_lowercase())
        } else {
            (content.to_string(), query.to_string())
        };
        search_content.matches(&search_query).count()
    }
}

/// 大小写不敏感的全部字面替换 / Case-insensitive replace-all for literal queries
pub fn replace_all_case_insensitive(content: &str, query: &str, replacement: &str) -> String {
    if query.is_empty() {
        return content.to_string();
    }
    let lower_content = content.to_lowercase();
    let lower_query = query.to_lowercase();
    let mut result = String::with_capacity(content.len());
    let mut pos = 0;
    while let Some(idx) = lower_content[pos..].find(&lower_query) {
        let abs = pos + idx;
        result.push_str(&content[pos..abs]);
        result.push_str(replacement);
        pos = abs + query.len();
    }
    result.push_str(&content[pos..]);
    result
}

/// 全部替换（字面或正则）/ Replace all (literal or regex)
pub fn replace_all_in_text(
    content: &str,
    query: &str,
    replacement: &str,
    case_insensitive: bool,
    use_regex: bool,
) -> String {
    if query.is_empty() {
        return content.to_string();
    }
    if use_regex {
        if let Ok(re) = build_regex(query, case_insensitive) {
            return re.replace_all(content, replacement).into_owned();
        }
        return content.to_string();
    }
    if case_insensitive {
        replace_all_case_insensitive(content, query, replacement)
    } else {
        content.replace(query, replacement)
    }
}

/// 替换第 n 个匹配（1-based；非法索引则原样返回）
/// Replace the n-th match (1-based; returns original text if the index is invalid)
pub fn replace_nth_match(
    content: &str,
    query: &str,
    replacement: &str,
    index: usize,
    case_insensitive: bool,
    use_regex: bool,
) -> String {
    if index == 0 || content.is_empty() || query.is_empty() {
        return content.to_string();
    }
    if use_regex {
        let Ok(re) = build_regex(query, case_insensitive) else {
            return content.to_string();
        };
        let mut result = String::with_capacity(content.len());
        let mut last_end = 0;
        for (i, m) in re.find_iter(content).enumerate() {
            if i + 1 == index {
                result.push_str(&content[last_end..m.start()]);
                result.push_str(replacement);
                last_end = m.end();
                break;
            }
        }
        result.push_str(&content[last_end..]);
        result
    } else {
        let (search_content, search_query) = if case_insensitive {
            (content.to_lowercase(), query.to_lowercase())
        } else {
            (content.to_string(), query.to_string())
        };
        let mut count = 0;
        let mut pos = 0;
        while let Some(idx) = search_content[pos..].find(&search_query) {
            count += 1;
            if count == index {
                let abs_idx = pos + idx;
                return format!(
                    "{}{}{}",
                    &content[..abs_idx],
                    replacement,
                    &content[abs_idx + query.len()..]
                );
            }
            pos += idx + 1;
        }
        content.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_all_literal_case_insensitive() {
        let out = replace_all_in_text("Foo foo FOO", "foo", "bar", true, false);
        assert_eq!(out, "bar bar bar");
    }

    #[test]
    fn test_count_matches_literal() {
        assert_eq!(count_matches("a a a", "a", false, false), 3);
        assert_eq!(count_matches("AaA", "a", true, false), 3);
    }

    #[test]
    fn test_replace_nth_match_literal() {
        let out = replace_nth_match("a a a", "a", "b", 2, false, false);
        assert_eq!(out, "a b a");
    }
}

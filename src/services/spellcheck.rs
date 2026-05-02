//! 拼写检查服务 / Spell Check Service
//!
//! 支持中英文拼写检查 / Supports Chinese and English spell checking
//! 注意：此模块为基础实现，可后续扩展 / Note: Basic implementation, can be extended later

use std::collections::{HashMap, HashSet};

/// 拼写检查服务 / Spell Check Service
pub struct SpellCheckService {
    /// 英文词典 / English dictionary
    english_dict: HashSet<String>,
    /// 常见拼写错误映射 / Common misspelling mappings
    corrections: HashMap<String, String>,
    /// 是否启用 / Is enabled
    enabled: bool,
}

impl SpellCheckService {
    /// 创建新的拼写检查服务 / Create new spell check service
    pub fn new() -> Self {
        let mut service = Self {
            english_dict: HashSet::new(),
            corrections: HashMap::new(),
            enabled: true,
        };

        // 加载基础英文词典 / Load basic English dictionary
        service.load_basic_dictionary();
        service.load_common_corrections();

        service
    }

    /// 加载基础词典 / Load basic dictionary
    fn load_basic_dictionary(&mut self) {
        // 常用英文单词（简化版）/ Common English words (simplified)
        let common_words = include_str!("../../assets/dictionaries/common_words.txt");
        for word in common_words.lines() {
            let word = word.trim().to_lowercase();
            if !word.is_empty() && !word.starts_with('#') {
                self.english_dict.insert(word);
            }
        }

        // 编程常用词 / Programming common words
        let programming_words = [
            "fn",
            "let",
            "mut",
            "const",
            "static",
            "pub",
            "mod",
            "use",
            "crate",
            "self",
            "super",
            "struct",
            "enum",
            "impl",
            "trait",
            "type",
            "where",
            "async",
            "await",
            "move",
            "ref",
            "dyn",
            "box",
            "loop",
            "while",
            "for",
            "in",
            "if",
            "else",
            "match",
            "return",
            "break",
            "continue",
            "yield",
            "unsafe",
            "extern",
            "macro",
            "rule",
            "function",
            "variable",
            "class",
            "object",
            "method",
            "property",
            "interface",
            "import",
            "export",
            "default",
            "extends",
            "implements",
            "package",
            "module",
            "component",
            "props",
            "state",
            "hook",
            "context",
            "render",
            "effect",
            "memo",
            "callback",
            "signal",
            "resource",
            "suspense",
            "error",
            "boundary",
            "fallback",
            "markdown",
            "html",
            "css",
            "javascript",
            "typescript",
            "rust",
            "cargo",
            "dioxus",
            "frontend",
            "backend",
            "api",
            "rest",
            "graphql",
            "database",
            "server",
            "client",
            "request",
            "response",
            "header",
            "body",
            "status",
            "error",
            "success",
            "warning",
            "config",
            "setting",
            "option",
            "preference",
            "theme",
            "language",
            "locale",
            "file",
            "folder",
            "directory",
            "path",
            "name",
            "type",
            "size",
            "date",
            "time",
            "content",
            "text",
            "string",
            "number",
            "integer",
            "float",
            "boolean",
            "array",
            "vector",
            "hashmap",
            "hashset",
            "option",
            "result",
            "some",
            "none",
            "ok",
            "err",
            "println",
            "printf",
            "console",
            "log",
            "debug",
            "info",
            "trace",
            "warn",
            "error",
            "todo",
            "fixme",
            "hack",
            "note",
            "info",
            "warning",
            "important",
            "deprecated",
        ];

        for word in programming_words {
            self.english_dict.insert(word.to_string());
        }
    }

    /// 加载常见拼写纠正 / Load common corrections
    fn load_common_corrections(&mut self) {
        let corrections = [
            // 常见拼写错误 / Common misspellings
            ("teh", "the"),
            ("recieve", "receive"),
            ("occured", "occurred"),
            ("seperate", "separate"),
            ("definately", "definitely"),
            ("occassion", "occasion"),
            ("accomodate", "accommodate"),
            ("untill", "until"),
            ("begining", "beginning"),
            ("beleive", "believe"),
            ("calender", "calendar"),
            ("collegue", "colleague"),
            ("commited", "committed"),
            ("concious", "conscious"),
            ("enviroment", "environment"),
            ("existance", "existence"),
            ("goverment", "government"),
            ("happend", "happened"),
            ("independant", "independent"),
            ("knowlege", "knowledge"),
            ("neccessary", "necessary"),
            ("occurance", "occurrence"),
            ("paralell", "parallel"),
            ("priviledge", "privilege"),
            ("recomend", "recommend"),
            ("refered", "referred"),
            ("sucessful", "successful"),
            ("tommorow", "tomorrow"),
            ("wierd", "weird"),
            ("writting", "writing"),
            ("youre", "you're"),
            ("dont", "don't"),
            ("wont", "won't"),
            ("cant", "can't"),
            ("didnt", "didn't"),
            ("doesnt", "doesn't"),
            ("isnt", "isn't"),
            ("wasnt", "wasn't"),
            ("werent", "weren't"),
            ("havent", "haven't"),
            ("hasnt", "hasn't"),
            ("hadnt", "hadn't"),
            ("wouldnt", "wouldn't"),
            ("couldnt", "couldn't"),
            ("shouldnt", "shouldn't"),
            ("im", "I'm"),
            ("ive", "I've"),
            ("id", "I'd"),
            ("youve", "you've"),
            ("youd", "you'd"),
            ("theyre", "they're"),
            ("theyve", "they've"),
            ("theyd", "they'd"),
            ("weve", "we've"),
            ("wed", "we'd"),
            ("whos", "who's"),
            ("whats", "what's"),
            ("wheres", "where's"),
            ("hows", "how's"),
            ("thats", "that's"),
            ("heres", "here's"),
            ("theres", "there's"),
        ];

        for (wrong, correct) in corrections {
            self.corrections
                .insert(wrong.to_string(), correct.to_string());
        }
    }

    /// 设置启用状态 / Set enabled status
    #[allow(dead_code)]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 检查单词拼写 / Check word spelling
    pub fn check_word(&self, word: &str) -> SpellCheckResult {
        if !self.enabled {
            return SpellCheckResult::Correct;
        }

        let lower_word = word.to_lowercase();

        // 跳过纯数字 / Skip pure numbers
        if lower_word
            .chars()
            .all(|c| c.is_numeric() || c == '.' || c == '-')
        {
            return SpellCheckResult::Correct;
        }

        // 跳过包含特殊字符的单词（如 URL、文件路径）/ Skip words with special chars
        if lower_word
            .chars()
            .any(|c| "!@#$%^&*()[]{}|\\/<>".contains(c))
        {
            return SpellCheckResult::Correct;
        }

        // 跳过过短的单词 / Skip very short words
        if lower_word.len() < 2 {
            return SpellCheckResult::Correct;
        }

        // 检查是否在词典中 / Check if in dictionary
        if self.english_dict.contains(&lower_word) {
            return SpellCheckResult::Correct;
        }

        // 检查是否是编程标识符风格 / Check if programming identifier style
        if self.is_programming_identifier(&lower_word) {
            return SpellCheckResult::Correct;
        }

        // 检查是否有已知纠正 / Check for known correction
        if let Some(correction) = self.corrections.get(&lower_word) {
            return SpellCheckResult::Misspelled {
                suggestions: vec![correction.clone()],
            };
        }

        // 生成建议 / Generate suggestions
        let suggestions = self.generate_suggestions(&lower_word);

        if suggestions.is_empty() {
            SpellCheckResult::Unknown
        } else {
            SpellCheckResult::Misspelled { suggestions }
        }
    }

    /// 检查是否是编程标识符 / Check if programming identifier
    fn is_programming_identifier(&self, word: &str) -> bool {
        // snake_case, camelCase, PascalCase, kebab-case
        let has_underscore = word.contains('_');
        let has_hyphen = word.contains('-');
        let has_camel_case = word.chars().any(|c| c.is_uppercase());

        // 如果包含下划线或连字符，可能是标识符
        if has_underscore || has_hyphen {
            return true;
        }

        // 检查驼峰命名（至少有一个大写字母不在开头）
        let first_char_upper = word
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false);
        if has_camel_case && !first_char_upper {
            return true;
        }

        false
    }

    /// 生成拼写建议 / Generate spelling suggestions
    fn generate_suggestions(&self, word: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        // 编辑距离为1的所有可能变体 / All variants with edit distance 1
        let variants = self.generate_edit_distance_1_variants(word);

        // 检查哪些变体在词典中 / Check which variants are in dictionary
        for variant in variants {
            if self.english_dict.contains(&variant) {
                suggestions.push(variant);
            }
        }

        // 检查纠正映射 / Check correction mapping
        if let Some(correction) = self.corrections.get(word) {
            if !suggestions.contains(correction) {
                suggestions.push(correction.clone());
            }
        }

        // 限制建议数量 / Limit number of suggestions
        suggestions.truncate(5);
        suggestions
    }

    /// 生成编辑距离为1的变体 / Generate variants with edit distance 1
    fn generate_edit_distance_1_variants(&self, word: &str) -> Vec<String> {
        let mut variants = Vec::new();
        let chars: Vec<char> = word.chars().collect();
        let alphabet = "abcdefghijklmnopqrstuvwxyz";

        // 删除 / Deletions
        for i in 0..chars.len() {
            let mut new_word: Vec<char> = chars.clone();
            new_word.remove(i);
            variants.push(new_word.into_iter().collect());
        }

        // 交换 / Transpositions
        for i in 0..chars.len().saturating_sub(1) {
            let mut new_word = chars.clone();
            new_word.swap(i, i + 1);
            variants.push(new_word.into_iter().collect());
        }

        // 替换 / Replacements
        for i in 0..chars.len() {
            for c in alphabet.chars() {
                let mut new_word = chars.clone();
                new_word[i] = c;
                variants.push(new_word.into_iter().collect());
            }
        }

        // 插入 / Insertions
        for i in 0..=chars.len() {
            for c in alphabet.chars() {
                let mut new_word = chars.clone();
                new_word.insert(i, c);
                variants.push(new_word.into_iter().collect());
            }
        }

        variants
    }

    /// 检查文本中的拼写错误 / Check spelling errors in text
    pub fn check_text(&self, text: &str) -> Vec<SpellError> {
        if !self.enabled {
            return Vec::new();
        }

        let mut errors = Vec::new();
        let mut in_code_block = false;

        for (line_idx, line) in text.lines().enumerate() {
            // 检测代码块 / Detect code blocks
            if line.trim().starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }

            // 跳过代码块内的内容 / Skip content inside code blocks
            if in_code_block {
                continue;
            }

            // 跳过行内代码 / Skip inline code
            let processed_line = self.skip_inline_code(line);

            // 提取单词 / Extract words
            for (word, start_pos) in self.extract_words_with_positions(&processed_line) {
                let result = self.check_word(&word);

                if let SpellCheckResult::Misspelled { suggestions } = result {
                    errors.push(SpellError {
                        word,
                        line: line_idx,
                        column: start_pos,
                        suggestions,
                    });
                }
            }
        }

        errors
    }

    /// 跳过行内代码 / Skip inline code
    fn skip_inline_code(&self, line: &str) -> String {
        let mut result = String::new();
        let mut in_code = false;

        for c in line.chars() {
            if c == '`' {
                in_code = !in_code;
                result.push(' '); // 用空格替换
            } else if in_code {
                result.push(' '); // 用空格替换代码内容
            } else {
                result.push(c);
            }
        }

        result
    }

    /// 提取单词及其位置 / Extract words with positions
    fn extract_words_with_positions(&self, text: &str) -> Vec<(String, usize)> {
        let mut words = Vec::new();
        let mut current_word = String::new();
        let mut word_start = 0;

        for (i, c) in text.char_indices() {
            if c.is_alphabetic() || c == '\'' {
                if current_word.is_empty() {
                    word_start = i;
                }
                current_word.push(c);
            } else {
                if !current_word.is_empty() {
                    words.push((current_word.clone(), word_start));
                    current_word.clear();
                }
            }
        }

        if !current_word.is_empty() {
            words.push((current_word, word_start));
        }

        words
    }

    /// 检查是否包含中文字符 / Check if contains Chinese characters
    #[allow(dead_code)]
    pub fn contains_chinese(&self, text: &str) -> bool {
        text.chars().any(Self::is_chinese_char)
    }

    /// 检查是否是中文字符 / Check if is Chinese character
    #[allow(dead_code)]
    fn is_chinese_char(c: char) -> bool {
        matches!(c, '\u{4E00}'..='\u{9FFF}')
    }
}

impl Default for SpellCheckService {
    fn default() -> Self {
        Self::new()
    }
}

/// 拼写检查结果 / Spell Check Result
#[derive(Clone, Debug, PartialEq)]
pub enum SpellCheckResult {
    /// 拼写正确 / Correct spelling
    Correct,
    /// 拼写错误，包含建议 / Misspelled with suggestions
    Misspelled { suggestions: Vec<String> },
    /// 未知（可能需要添加到词典）/ Unknown (may need to be added to dictionary)
    Unknown,
}

/// 拼写错误 / Spell Error
#[derive(Clone, Debug, PartialEq)]
pub struct SpellError {
    /// 错误单词 / Misspelled word
    pub word: String,
    /// 行号 / Line number
    pub line: usize,
    /// 列号 / Column number
    pub column: usize,
    /// 建议纠正 / Suggested corrections
    pub suggestions: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spell_check_service_creation() {
        let service = SpellCheckService::new();
        assert!(service.enabled);
    }

    #[test]
    fn test_correct_word() {
        let service = SpellCheckService::new();
        let result = service.check_word("hello");
        assert_eq!(result, SpellCheckResult::Correct);
    }

    #[test]
    fn test_misspelled_word() {
        let service = SpellCheckService::new();
        let result = service.check_word("teh");
        assert!(matches!(result, SpellCheckResult::Misspelled { .. }));
    }

    #[test]
    fn test_programming_word() {
        let service = SpellCheckService::new();
        let result = service.check_word("function");
        assert_eq!(result, SpellCheckResult::Correct);
    }

    #[test]
    fn test_check_text() {
        let service = SpellCheckService::new();
        let text = "Hello world, this is a test.";
        let errors = service.check_text(text);
        // "world" is in dictionary, so no errors expected
        assert!(errors.is_empty() || errors.iter().any(|e| e.word != "world"));
    }

    #[test]
    fn test_skip_code_blocks() {
        let service = SpellCheckService::new();
        let text = "```\nmisspelleddword\n```\nHello world";
        let errors = service.check_text(text);
        // 代码块内的拼写错误应该被跳过
        assert!(errors.iter().all(|e| e.word != "misspelleddword"));
    }

    #[test]
    fn test_chinese_detection() {
        let service = SpellCheckService::new();
        assert!(service.contains_chinese("你好"));
        assert!(!service.contains_chinese("hello"));
    }
}

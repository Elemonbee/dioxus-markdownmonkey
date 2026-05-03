//! 表格可视化编辑器组件 / Table Visual Editor Component
//!
//! 可视化表格创建和编辑，支持动态行列操作
//! Visual table creation and editing with dynamic row/column operations

use crate::state::AppState;
use crate::utils::i18n::t;
use dioxus::prelude::*;

/// 表格数据 / Table Data
#[derive(Clone, Debug, PartialEq, Default)]
pub struct TableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl TableData {
    /// 从 Markdown 解析表格 / Parse table from Markdown
    #[allow(dead_code)]
    pub fn from_markdown(markdown: &str) -> Option<Self> {
        let lines: Vec<&str> = markdown.trim().lines().collect();
        if lines.len() < 2 {
            return None;
        }

        // 解析表头 / Parse headers
        let header_line = lines[0];
        if !header_line.starts_with('|') || !header_line.ends_with('|') {
            return None;
        }

        let headers: Vec<String> = header_line
            .trim_matches('|')
            .split('|')
            .map(|s| s.trim().to_string())
            .collect();

        // 跳过分隔行 / Skip separator line
        // 解析数据行 / Parse data rows
        let mut rows = Vec::new();
        for line in lines.iter().skip(2) {
            if line.starts_with('|') && line.ends_with('|') {
                let row: Vec<String> = line
                    .trim_matches('|')
                    .split('|')
                    .map(|s| s.trim().to_string())
                    .collect();
                rows.push(row);
            }
        }

        Some(Self { headers, rows })
    }

    /// 转换为 Markdown / Convert to Markdown
    pub fn to_markdown(&self) -> String {
        let mut result = String::new();

        // 表头 / Headers
        result.push('|');
        for header in &self.headers {
            result.push_str(&format!(" {} |", header));
        }
        result.push('\n');

        // 分隔行 / Separator
        result.push('|');
        for _ in &self.headers {
            result.push_str(" --- |");
        }
        result.push('\n');

        // 数据行 / Data rows
        for row in &self.rows {
            result.push('|');
            for cell in row {
                result.push_str(&format!(" {} |", cell));
            }
            result.push('\n');
        }

        result
    }

    /// 创建空表格 / Create empty table
    #[allow(dead_code)]
    pub fn new(columns: usize, rows: usize) -> Self {
        Self::new_with_lang(columns, rows, crate::state::Language::ZhCN)
    }

    /// 创建空表格（带语言）/ Create empty table (with language)
    pub fn new_with_lang(columns: usize, rows: usize, lang: crate::state::Language) -> Self {
        let headers = (0..columns)
            .map(|i| format!("{} {}", t("column", lang), i + 1))
            .collect();
        let rows = (0..rows).map(|_| vec![String::new(); columns]).collect();
        Self { headers, rows }
    }

    /// 添加列 / Add column
    #[allow(dead_code)]
    pub fn add_column(&mut self) {
        self.add_column_with_lang(crate::state::Language::ZhCN)
    }

    /// 添加列（带语言）/ Add column (with language)
    pub fn add_column_with_lang(&mut self, lang: crate::state::Language) {
        self.headers
            .push(format!("{} {}", t("column", lang), self.headers.len() + 1));
        for row in &mut self.rows {
            row.push(String::new());
        }
    }

    /// 添加行 / Add row
    pub fn add_row(&mut self) {
        self.rows.push(vec![String::new(); self.headers.len()]);
    }

    /// 删除列 / Remove column
    pub fn remove_column(&mut self, index: usize) {
        if index < self.headers.len() && self.headers.len() > 1 {
            self.headers.remove(index);
            for row in &mut self.rows {
                if index < row.len() {
                    row.remove(index);
                }
            }
        }
    }

    /// 删除行 / Remove row
    pub fn remove_row(&mut self, index: usize) {
        if index < self.rows.len() && !self.rows.is_empty() {
            self.rows.remove(index);
        }
    }
}

/// 表格可视化编辑器 / Table Visual Editor
#[component]
pub fn TableEditorModal() -> Element {
    let mut state = use_context::<AppState>();
    let show = *state.show_table_editor.read();
    let lang = *state.language.read();

    let mut table_data = use_signal(move || TableData::new_with_lang(3, 3, lang));

    if !show {
        return rsx! {};
    }

    let title = t("table_editor", lang);
    let add_row_text = t("add_row", lang);
    let cancel_text = t("cancel", lang);
    let insert_text = t("insert_table", lang);

    let data = table_data.read();
    let headers = data.headers.clone();
    let rows = data.rows.clone();
    drop(data);

    rsx! {
        div {
            class: "modal-overlay",
            onclick: move |_| {
                *state.show_table_editor.write() = false;
            },

            div {
                class: "modal table-editor-modal",
                onclick: move |e| e.stop_propagation(),

                div { class: "modal-header",
                    h3 { "{title}" }
                    button {
                        class: "modal-close",
                        onclick: move |_| {
                            *state.show_table_editor.write() = false;
                        },
                        "×"
                    }
                }

                div { class: "modal-body",
                    div { class: "table-container",
                        table { class: "visual-table",
                            // 表头 / Headers
                            thead {
                                tr {
                                    for (col_index, header) in headers.iter().enumerate() {
                                        th { key: "{col_index}",
                                            input {
                                                r#type: "text",
                                                value: "{header}",
                                                oninput: move |e| {
                                                    let mut data = table_data.write();
                                                    if col_index < data.headers.len() {
                                                        data.headers[col_index] = e.value();
                                                    }
                                                },
                                            }
                                            button {
                                                class: "btn-remove-col",
                                                onclick: move |_| {
                                                    let mut data = table_data.write();
                                                    data.remove_column(col_index);
                                                },
                                                "×"
                                            }
                                        }
                                    }
                                    th { class: "action-cell",
                                        button {
                                            class: "btn-add-col",
                                            onclick: move |_| {
                                                let mut data = table_data.write();
                                                data.add_column_with_lang(lang);
                                            },
                                            "+"
                                        }
                                    }
                                }
                            }

                            // 数据行 / Data rows
                            tbody {
                                for (row_index, row) in rows.iter().enumerate() {
                                    tr { key: "{row_index}",
                                        for (col_index, cell) in row.iter().enumerate() {
                                            td { key: "{col_index}",
                                                input {
                                                    r#type: "text",
                                                    value: "{cell}",
                                                    oninput: move |e| {
                                                        let mut data = table_data.write();
                                                        if row_index < data.rows.len() && col_index < data.rows[row_index].len() {
                                                            data.rows[row_index][col_index] = e.value();
                                                        }
                                                    },
                                                }
                                            }
                                        }
                                        td { class: "action-cell",
                                            button {
                                                class: "btn-remove-row",
                                                onclick: move |_| {
                                                    let mut data = table_data.write();
                                                    data.remove_row(row_index);
                                                },
                                                "×"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    div { class: "table-actions",
                        button {
                            class: "btn-add-row",
                            onclick: move |_| {
                                let mut data = table_data.write();
                                data.add_row();
                            },
                            "+ {add_row_text}"
                        }
                    }
                }

                div { class: "modal-footer",
                    button {
                        class: "btn-secondary",
                        onclick: move |_| {
                            *state.show_table_editor.write() = false;
                        },
                        "{cancel_text}"
                    }
                    button {
                        class: "btn-primary",
                        onclick: move |_| {
                            let data = table_data.read().clone();
                            let markdown = data.to_markdown();
                            // 在光标位置插入表格
                            state.insert_at_cursor(&markdown);
                            *state.show_table_editor.write() = false;
                            // 重置表格数据
                            *table_data.write() = TableData::new_with_lang(3, 3, lang);
                        },
                        "{insert_text}"
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_from_markdown() {
        let md = "| Name | Age |\n|---|---|\n| Alice | 30 |\n| Bob | 25 |";
        let table = TableData::from_markdown(md).unwrap();
        assert_eq!(table.headers, vec!["Name", "Age"]);
        assert_eq!(table.rows, vec![vec!["Alice", "30"], vec!["Bob", "25"]]);
    }

    #[test]
    fn test_table_from_markdown_invalid() {
        assert!(TableData::from_markdown("not a table").is_none());
        assert!(TableData::from_markdown("| only header |").is_none());
    }

    #[test]
    fn test_table_to_markdown() {
        let table = TableData {
            headers: vec!["A".into(), "B".into()],
            rows: vec![vec!["1".into(), "2".into()]],
        };
        let md = table.to_markdown();
        assert!(md.contains("| A | B |"));
        assert!(md.contains("| --- | --- |"));
        assert!(md.contains("| 1 | 2 |"));
    }

    #[test]
    fn test_table_roundtrip() {
        let original = "| X | Y |\n|---|---|\n| 1 | 2 |\n";
        let table = TableData::from_markdown(original).unwrap();
        let roundtrip = table.to_markdown();
        let table2 = TableData::from_markdown(&roundtrip).unwrap();
        assert_eq!(table.headers, table2.headers);
        assert_eq!(table.rows, table2.rows);
    }

    #[test]
    fn test_table_add_row() {
        let mut table = TableData::new(2, 1);
        table.add_row();
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.rows[1].len(), 2);
    }

    #[test]
    fn test_table_remove_row() {
        let mut table = TableData::new(2, 3);
        table.remove_row(1);
        assert_eq!(table.rows.len(), 2);
    }

    #[test]
    fn test_table_remove_column() {
        let mut table = TableData::new(3, 1);
        table.remove_column(1);
        assert_eq!(table.headers.len(), 2);
        assert_eq!(table.rows[0].len(), 2);
    }
}

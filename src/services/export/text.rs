//! 纯文本导出 / Plain Text Export

use super::shared::*;
use std::path::Path;

/// 导出为纯文本 / Export to Plain Text
pub fn export_to_text(content: &str, output_path: &Path) -> Result<(), ExportError> {
    std::fs::write(output_path, content)?;
    Ok(())
}

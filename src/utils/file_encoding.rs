//! 文件编码检测与写入 / File encoding detection and writing

use std::fmt;
use std::fs;
use std::path::Path;

/// 编辑器支持的文件编码 / File encodings supported by the editor
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FileEncoding {
    /// 不带 BOM 的 UTF-8 / UTF-8 without BOM
    #[default]
    Utf8,
    /// 带 BOM 的 UTF-8 / UTF-8 with BOM
    Utf8Bom,
    /// 带 BOM 的小端 UTF-16 / Little-endian UTF-16 with BOM
    Utf16Le,
    /// 带 BOM 的大端 UTF-16 / Big-endian UTF-16 with BOM
    Utf16Be,
    /// GBK（Windows-936）/ GBK (Windows-936)
    Gbk,
}

impl fmt::Display for FileEncoding {
    /// 返回状态栏使用的稳定编码名称 / Return the stable encoding name used by the status bar
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Utf8 => "UTF-8",
            Self::Utf8Bom => "UTF-8 BOM",
            Self::Utf16Le => "UTF-16 LE",
            Self::Utf16Be => "UTF-16 BE",
            Self::Gbk => "GBK",
        })
    }
}

/// 从磁盘读取文件并检测其编码 / Read a file from disk and detect its encoding
pub fn read_file(path: &Path) -> Result<(String, FileEncoding), String> {
    let bytes =
        fs::read(path).map_err(|error| format!("无法读取文件 / Cannot read file: {}", error))?;
    decode_bytes(&bytes)
}

/// 检测并严格解码文件字节 / Detect and strictly decode file bytes
pub fn decode_bytes(bytes: &[u8]) -> Result<(String, FileEncoding), String> {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        let content = std::str::from_utf8(&bytes[3..])
            .map_err(|error| format!("UTF-8 BOM 解码失败 / UTF-8 BOM decode failed: {}", error))?;
        return Ok((content.to_owned(), FileEncoding::Utf8Bom));
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        return decode_utf16(&bytes[2..], true).map(|content| (content, FileEncoding::Utf16Le));
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        return decode_utf16(&bytes[2..], false).map(|content| (content, FileEncoding::Utf16Be));
    }
    if let Ok(content) = std::str::from_utf8(bytes) {
        return Ok((content.to_owned(), FileEncoding::Utf8));
    }

    let (content, _, had_errors) = encoding_rs::GBK.decode(bytes);
    if had_errors {
        return Err(
            "文件既不是有效 UTF-8，也不是有效 GBK / File is neither valid UTF-8 nor valid GBK"
                .to_string(),
        );
    }
    Ok((content.into_owned(), FileEncoding::Gbk))
}

/// 按指定编码严格编码文本 / Strictly encode text using the specified encoding
pub fn encode_text(content: &str, encoding: FileEncoding) -> Result<Vec<u8>, String> {
    match encoding {
        FileEncoding::Utf8 => Ok(content.as_bytes().to_vec()),
        FileEncoding::Utf8Bom => {
            let mut bytes = Vec::with_capacity(3 + content.len());
            bytes.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
            bytes.extend_from_slice(content.as_bytes());
            Ok(bytes)
        }
        FileEncoding::Utf16Le => {
            let mut bytes = Vec::with_capacity(2 + content.len() * 2);
            bytes.extend_from_slice(&[0xFF, 0xFE]);
            bytes.extend(content.encode_utf16().flat_map(u16::to_le_bytes));
            Ok(bytes)
        }
        FileEncoding::Utf16Be => {
            let mut bytes = Vec::with_capacity(2 + content.len() * 2);
            bytes.extend_from_slice(&[0xFE, 0xFF]);
            bytes.extend(content.encode_utf16().flat_map(u16::to_be_bytes));
            Ok(bytes)
        }
        FileEncoding::Gbk => {
            let (bytes, _, had_errors) = encoding_rs::GBK.encode(content);
            if had_errors {
                return Err(
                    "内容包含 GBK 无法表示的字符 / Content contains characters not representable in GBK"
                        .to_string(),
                );
            }
            Ok(bytes.into_owned())
        }
    }
}

/// 使用指定编码将文本写入磁盘 / Write text to disk using the specified encoding
pub fn write_file(path: &Path, content: &str, encoding: FileEncoding) -> Result<(), String> {
    let bytes = encode_text(content, encoding)?;
    fs::write(path, bytes).map_err(|error| format!("无法保存文件 / Cannot save file: {}", error))
}

/// 按指定字节序严格解码 UTF-16 / Strictly decode UTF-16 in the specified byte order
fn decode_utf16(data: &[u8], little_endian: bool) -> Result<String, String> {
    if !data.len().is_multiple_of(2) {
        return Err("UTF-16 字节数不是偶数 / UTF-16 byte length is not even".to_string());
    }
    let units = data.chunks_exact(2).map(|chunk| {
        if little_endian {
            u16::from_le_bytes([chunk[0], chunk[1]])
        } else {
            u16::from_be_bytes([chunk[0], chunk[1]])
        }
    });
    std::char::decode_utf16(units)
        .map(|item| {
            item.map_err(|error| format!("UTF-16 解码失败 / UTF-16 decode failed: {}", error))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{decode_bytes, encode_text, FileEncoding};

    /// 所有支持编码应保留正文并精确保留 BOM / All supported encodings preserve content and exact BOMs
    #[test]
    fn supported_encodings_round_trip_with_expected_boms() {
        let cases = [
            (FileEncoding::Utf8, &[][..]),
            (FileEncoding::Utf8Bom, &[0xEF, 0xBB, 0xBF][..]),
            (FileEncoding::Utf16Le, &[0xFF, 0xFE][..]),
            (FileEncoding::Utf16Be, &[0xFE, 0xFF][..]),
            (FileEncoding::Gbk, &[][..]),
        ];

        for (encoding, bom) in cases {
            let content = "标题 ABC";
            let bytes = encode_text(content, encoding).unwrap();
            assert!(bytes.starts_with(bom), "{encoding}");
            let (decoded, detected) = decode_bytes(&bytes).unwrap();
            assert_eq!(decoded, content);
            assert_eq!(detected, encoding);
        }
    }

    /// GBK 编码遇到不可表示字符时必须失败 / GBK encoding must fail for unrepresentable characters
    #[test]
    fn gbk_rejects_unrepresentable_characters() {
        let result = encode_text("emoji 😀", FileEncoding::Gbk);
        assert!(result.is_err());
    }

    /// 损坏的 UTF-16 不应被静默截断 / Malformed UTF-16 must not be silently truncated
    #[test]
    fn malformed_utf16_is_rejected() {
        assert!(decode_bytes(&[0xFF, 0xFE, 0x41]).is_err());
    }
}

//! API Key 安全存储服务 / API Key Secure Storage Service
//!
//! 使用系统密钥环（keyring）安全存储 API Key
//! Uses system keyring for secure API Key storage

#![allow(dead_code)]

use keyring::Entry;

const KEYRING_SERVICE: &str = "markdownmonkey";

/// 存储 API Key / Store API Key
///
/// # 参数 / Arguments
/// * `provider` - AI 提供商名称 / AI provider name
/// * `api_key` - API 密钥 / API key
pub fn store_api_key(provider: &str, api_key: &str) -> Result<(), String> {
    let entry = Entry::new(KEYRING_SERVICE, provider)
        .map_err(|e| format!("创建密钥环条目失败 / Failed to create keyring entry: {}", e))?;
    entry
        .set_password(api_key)
        .map_err(|e| format!("存储 API Key 失败 / Failed to store API Key: {}", e))?;
    Ok(())
}

/// 获取 API Key / Get API Key
///
/// # 参数 / Arguments
/// * `provider` - AI 提供商名称 / AI provider name
///
/// # 返回 / Returns
/// API 密钥，如果不存在则返回错误 / API key, or error if not exists
pub fn get_api_key(provider: &str) -> Result<String, String> {
    let entry = Entry::new(KEYRING_SERVICE, provider)
        .map_err(|e| format!("创建密钥环条目失败 / Failed to create keyring entry: {}", e))?;
    entry
        .get_password()
        .map_err(|e| format!("获取 API Key 失败 / Failed to get API Key: {}", e))
}

/// 删除 API Key / Delete API Key
///
/// # 参数 / Arguments
/// * `provider` - AI 提供商名称 / AI provider name
pub fn delete_api_key(provider: &str) -> Result<(), String> {
    let entry = Entry::new(KEYRING_SERVICE, provider)
        .map_err(|e| format!("创建密钥环条目失败 / Failed to create keyring entry: {}", e))?;
    entry
        .delete_credential()
        .map_err(|e| format!("删除 API Key 失败 / Failed to delete API Key: {}", e))?;
    Ok(())
}

/// 检查 API Key 是否存在 / Check if API Key exists
///
/// # 参数 / Arguments
/// * `provider` - AI 提供商名称 / AI provider name
pub fn has_api_key(provider: &str) -> bool {
    Entry::new(KEYRING_SERVICE, provider)
        .ok()
        .and_then(|e| e.get_password().ok())
        .is_some()
}

/// 迁移 API Key 到密钥环 / Migrate API Key to keyring
///
/// 如果密钥环中没有 API Key，但提供了 plaintext_key，则将其存储到密钥环
/// If keyring has no API key but plaintext_key is provided, store it to keyring
///
/// # 参数 / Arguments
/// * `provider` - AI 提供商名称 / AI provider name
/// * `plaintext_key` - 明文 API Key（来自配置文件）/ Plaintext API Key (from config file)
///
/// # 返回 / Returns
/// 如果迁移成功返回 true / Returns true if migration was successful
pub fn migrate_api_key_if_needed(provider: &str, plaintext_key: Option<&str>) -> bool {
    // 如果密钥环中已有，无需迁移 / If keyring already has the key, no migration needed
    if has_api_key(provider) {
        return true;
    }

    // 如果明文密钥为空，无法迁移 / If plaintext key is empty, cannot migrate
    let key = match plaintext_key {
        Some(k) if !k.is_empty() => k,
        _ => return false,
    };

    // 尝试迁移到密钥环 / Try to migrate to keyring
    match store_api_key(provider, key) {
        Ok(()) => {
            tracing::info!(
                "API Key 已从配置文件迁移到密钥环: {} / \
                 API Key migrated from config to keyring: {}",
                provider,
                provider
            );
            true
        }
        Err(e) => {
            tracing::warn!(
                "API Key 迁移失败: {} - {} / API Key migration failed: {} - {}",
                provider,
                e,
                provider,
                e
            );
            false
        }
    }
}

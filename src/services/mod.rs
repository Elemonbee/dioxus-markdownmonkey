//! 服务模块 / Services Module
//!
//! 后端服务层 / Backend Services Layer

pub mod ai;
pub mod auto_save;
pub mod export;
pub mod file_watcher;
pub mod image;
pub mod keyring_service;
pub mod markdown;
pub mod recent_files;
pub mod settings;
pub mod spellcheck;
pub mod syntax_highlight;

#[cfg(test)]
mod __tests;

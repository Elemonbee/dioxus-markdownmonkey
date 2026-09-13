//! 服务模块 / Services Module
//!
//! 后端服务层 / Backend Services Layer

pub mod ai;
pub mod auto_save;
pub mod export;
pub mod file_watcher;
pub mod highlight;
pub mod image;
pub mod katex_css;
pub mod keyring_service;
pub mod markdown;
pub mod recent_files;
pub mod session;
pub mod settings;
pub mod theme_detector;

#[cfg(test)]
mod __tests;

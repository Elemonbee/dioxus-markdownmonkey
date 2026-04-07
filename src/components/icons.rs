#![allow(dead_code)]
//! SVG Icon Components
//!
//! 注意：部分图标为预留功能，暂未使用
//! Note: Some icons are reserved for future use, not yet used

use dioxus::prelude::*;

/// Icon Props
#[derive(Props, Clone, PartialEq)]
pub struct IconProps {
    #[props(default = 20)]
    pub size: u32,
    #[props(default = "currentColor".to_string())]
    pub color: String,
    #[props(default = "".to_string())]
    pub class: String,
}

/// New File Icon
pub fn NewFileIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" }
            polyline { points: "14 2 14 8 20 8" }
            line { x1: "12", y1: "18", x2: "12", y2: "12" }
            line { x1: "9", y1: "15", x2: "15", y2: "15" }
        }
    }
}

/// Open File Icon
pub fn OpenFileIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" }
        }
    }
}

/// Save Icon
pub fn SaveIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z" }
            polyline { points: "17 21 17 13 7 13 7 21" }
            polyline { points: "7 3 7 8 15 8" }
        }
    }
}

/// Undo Icon
pub fn UndoIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M3 7v6h6" }
            path { d: "M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13" }
        }
    }
}

/// Redo Icon
pub fn RedoIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M21 7v6h-6" }
            path { d: "M3 17a9 9 0 0 1 9-9 9 9 0 0 1 6 2.3l3 2.7" }
        }
    }
}

/// Copy Icon
pub fn CopyIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect { x: "9", y: "9", width: "13", height: "13", rx: "2", ry: "2" }
            path { d: "M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" }
        }
    }
}

/// Paste Icon
pub fn PasteIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" }
            rect { x: "8", y: "2", width: "8", height: "4", rx: "1", ry: "1" }
        }
    }
}

/// Bold Icon
pub fn BoldIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M6 4h8a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z" }
            path { d: "M6 12h9a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z" }
        }
    }
}

/// Italic Icon
pub fn ItalicIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line { x1: "19", y1: "4", x2: "10", y2: "4" }
            line { x1: "14", y1: "20", x2: "5", y2: "20" }
            line { x1: "15", y1: "4", x2: "9", y2: "20" }
        }
    }
}

/// Code Icon
pub fn CodeIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polyline { points: "16 18 22 12 16 6" }
            polyline { points: "8 6 2 12 8 18" }
        }
    }
}

/// Link Icon
pub fn LinkIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" }
            path { d: "M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" }
        }
    }
}

/// Image Icon
pub fn ImageIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect { x: "3", y: "3", width: "18", height: "18", rx: "2", ry: "2" }
            circle { cx: "8.5", cy: "8.5", r: "1.5" }
            polyline { points: "21 15 16 10 5 21" }
        }
    }
}

/// Table Icon
pub fn TableIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M9 3H5a2 2 0 0 0-2 2v4m6-6h10a2 2 0 0 1 2 2v4M9 3v18m0 0h10a2 2 0 0 0 2-2V9M9 21H5a2 2 0 0 1-2-2V9m0 0h18" }
        }
    }
}

/// Quote Icon
pub fn QuoteIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M3 21c3 0 7-1 7-8V5c0-1.25-.756-2.017-2-2H4c-1.25 0-2 .75-2 1.972V11c0 1.25.75 2 2 2 1 0 1 0 1 1v1c0 1-1 2-2 2s-1 .008-1 1.031V21z" }
            path { d: "M15 21c3 0 7-1 7-8V5c0-1.25-.757-2.017-2-2h-4c-1.25 0-2 .75-2 1.972V11c0 1.25.75 2 2 2h.75c0 2.25.25 4-2.75 4v3z" }
        }
    }
}

/// Divider Icon
pub fn DividerIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line { x1: "3", y1: "12", x2: "21", y2: "12" }
        }
    }
}

/// List Icon
pub fn ListIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line { x1: "8", y1: "6", x2: "21", y2: "6" }
            line { x1: "8", y1: "12", x2: "21", y2: "12" }
            line { x1: "8", y1: "18", x2: "21", y2: "18" }
            line { x1: "3", y1: "6", x2: "3.01", y2: "6" }
            line { x1: "3", y1: "12", x2: "3.01", y2: "12" }
            line { x1: "3", y1: "18", x2: "3.01", y2: "18" }
        }
    }
}

/// Ordered List Icon
pub fn OrderedListIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line { x1: "10", y1: "6", x2: "21", y2: "6" }
            line { x1: "10", y1: "12", x2: "21", y2: "12" }
            line { x1: "10", y1: "18", x2: "21", y2: "18" }
            path { d: "M4 6h1v4" }
            path { d: "M4 10h2" }
            path { d: "M6 18H4c0-1 2-2 2-3s-1-1.5-2-1" }
        }
    }
}

/// Sidebar Icon
pub fn SidebarIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect { x: "3", y: "3", width: "18", height: "18", rx: "2", ry: "2" }
            line { x1: "9", y1: "3", x2: "9", y2: "21" }
        }
    }
}

/// Preview Icon (Eye)
pub fn PreviewIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" }
            circle { cx: "12", cy: "12", r: "3" }
        }
    }
}

/// Settings Icon
pub fn SettingsIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "3" }
            path { d: "M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" }
        }
    }
}

/// AI Icon (Sparkles)
pub fn AIIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275L12 3Z" }
            path { d: "M5 3v4" }
            path { d: "M19 17v4" }
            path { d: "M3 5h4" }
            path { d: "M17 19h4" }
        }
    }
}

/// Theme Icon (Moon)
pub fn ThemeIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" }
        }
    }
}

/// Close Icon
pub fn CloseIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line { x1: "18", y1: "6", x2: "6", y2: "18" }
            line { x1: "6", y1: "6", x2: "18", y2: "18" }
        }
    }
}

/// Plus Icon
pub fn PlusIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line { x1: "12", y1: "5", x2: "12", y2: "19" }
            line { x1: "5", y1: "12", x2: "19", y2: "12" }
        }
    }
}

/// Search Icon
pub fn SearchIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "11", cy: "11", r: "8" }
            line { x1: "21", y1: "21", x2: "16.65", y2: "16.65" }
        }
    }
}

/// File Icon
pub fn FileIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" }
            polyline { points: "14 2 14 8 20 8" }
        }
    }
}

/// Folder Icon
pub fn FolderIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" }
        }
    }
}

/// Loading Icon
pub fn LoadingIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line { x1: "12", y1: "2", x2: "12", y2: "6" }
            line { x1: "12", y1: "18", x2: "12", y2: "22" }
            line { x1: "4.93", y1: "4.93", x2: "7.76", y2: "7.76" }
            line { x1: "16.24", y1: "16.24", x2: "19.07", y2: "19.07" }
            line { x1: "2", y1: "12", x2: "6", y2: "12" }
            line { x1: "18", y1: "12", x2: "22", y2: "12" }
            line { x1: "4.93", y1: "19.07", x2: "7.76", y2: "16.24" }
            line { x1: "16.24", y1: "7.76", x2: "19.07", y2: "4.93" }
        }
    }
}

/// Send Icon
pub fn SendIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line { x1: "22", y1: "2", x2: "11", y2: "13" }
            polygon { points: "22 2 15 22 11 13 2 9 22 2" }
        }
    }
}

/// Trash Icon
pub fn TrashIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polyline { points: "3 6 5 6 21 6" }
            path { d: "M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" }
        }
    }
}

/// Info Icon
pub fn InfoIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "10" }
            line { x1: "12", y1: "16", x2: "12", y2: "12" }
            line { x1: "12", y1: "8", x2: "12.01", y2: "8" }
        }
    }
}

/// Warning Icon
pub fn WarningIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z" }
            line { x1: "12", y1: "9", x2: "12", y2: "13" }
            line { x1: "12", y1: "17", x2: "12.01", y2: "17" }
        }
    }
}

/// Refresh Icon
pub fn RefreshIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M21 2v6h-6" }
            path { d: "M3 12a9 9 0 0 1 15-6.7L21 8" }
            path { d: "M3 22v-6h6" }
            path { d: "M21 12a9 9 0 0 1-15 6.7L3 16" }
        }
    }
}

/// AI Continue Icon
pub fn ContinueIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M5 12h14" }
            path { d: "m12 5 7 7-7 7" }
        }
    }
}

/// AI Improve Icon  
pub fn ImproveIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polygon { points: "12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" }
        }
    }
}

/// AI Outline Icon
pub fn OutlineIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line { x1: "8", y1: "6", x2: "21", y2: "6" }
            line { x1: "8", y1: "12", x2: "21", y2: "12" }
            line { x1: "8", y1: "18", x2: "21", y2: "18" }
            line { x1: "3", y1: "6", x2: "3.01", y2: "6" }
            line { x1: "3", y1: "12", x2: "3.01", y2: "12" }
            line { x1: "3", y1: "18", x2: "3.01", y2: "18" }
        }
    }
}

/// AI Translate Icon
pub fn TranslateIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "m5 8 6 6" }
            path { d: "m4 14 6-6 2-3" }
            path { d: "M2 5h12" }
            path { d: "M7 2h1" }
            path { d: "m22 22-5-10-5 10" }
            path { d: "M14 18h6" }
        }
    }
}

/// Language Icon (Globe)
pub fn LanguageIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "10" }
            line { x1: "2", y1: "12", x2: "22", y2: "12" }
            path { d: "M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" }
        }
    }
}

/// AI Grammar Icon
pub fn GrammarIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M12 20h9" }
            path { d: "M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" }
        }
    }
}

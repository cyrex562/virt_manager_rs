// libvirtmanager - Rust implementation of virt-manager components
//
// Copyright (C) 2025
// This work is licensed under the GNU GPLv2 or later.

pub mod about;
pub mod addhardware;
pub mod app;

// Re-export main types for easier access
pub use about::{AboutDialogManager, VmmAbout};
pub use addhardware::VmmAddHardware;
pub use app::run as run_main_app;

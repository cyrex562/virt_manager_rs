// Copyright (C) 2010, 2013 Red Hat, Inc.
// Copyright (C) 2010 Cole Robinson <crobinso@redhat.com>
// Rust port Copyright (C) 2025
//
// This work is licensed under the GNU GPLv2 or later.
// See the COPYING file in the top-level directory.

use iced::widget::{button, column, container, text, Column};
use iced::{window, Alignment, Element, Length, Task, Theme};
use log::debug;
use std::sync::{Arc, Mutex};

/// Message types for the About dialog
#[derive(Debug, Clone)]
pub enum Message {
    Close,
    CloseWindow(window::Id),
}

/// VmmAbout manages the application's About dialog using Iced
pub struct VmmAbout {
    app_version: String,
}

impl VmmAbout {
    /// Create a new VmmAbout instance
    pub fn new(app_version: String) -> Self {
        debug!("Creating VmmAbout with version: {}", app_version);
        Self {
            app_version,
        }
    }

    /// Show the About dialog as a new window
    ///
    /// # Arguments
    /// * `app_version` - The application version string to display
    ///
    /// # Returns
    /// Result indicating success or error message
    pub fn show_instance(app_version: &str) -> Result<(), String> {
        debug!("Showing about dialog");

        let version = app_version.to_string();
        match iced::application(
            "About Virtual Machine Manager",
            VmmAbout::update_static,
            VmmAbout::view_static,
        )
        .theme(|_| Theme::default())
        .window(window::Settings {
            size: iced::Size::new(400.0, 300.0),
            position: window::Position::Centered,
            resizable: false,
            decorations: true,
            ..Default::default()
        })
        .run_with(move || VmmAbout::new_static(version))
        {
            Ok(_) => {
                debug!("About dialog closed successfully");
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Error launching 'About' dialog: {}", e);
                Err(error_msg)
            }
        }
    }

    /// Build the UI view
    fn view(&self) -> Element<'_, Message> {
        let title = text("Virtual Machine Manager")
            .size(24)
            .width(Length::Fill);

        let version = text(format!("Version: {}", self.app_version))
            .size(16)
            .width(Length::Fill);

        let copyright = text("Copyright (C) 2006-2025 Red Hat, Inc.")
            .size(12)
            .width(Length::Fill);

        let description = text(
            "A desktop application for managing virtual machines through libvirt"
        )
        .size(14)
        .width(Length::Fill);

        let license = text("Licensed under the GNU GPLv2 or later")
            .size(12)
            .width(Length::Fill);

        let close_button = button("Close")
            .on_press(Message::Close)
            .padding(10);

        let content: Column<Message> = column![
            title,
            version,
            copyright,
            description,
            license,
            close_button,
        ]
        .spacing(20)
        .padding(30)
        .align_x(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    /// Handle updates from messages
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Close => {
                debug!("Closing about dialog");
                window::get_latest()
                    .and_then(window::close)
            }
            Message::CloseWindow(id) => {
                debug!("Window {:?} closed", id);
                Task::none()
            }
        }
    }

    /// Static initialization function for the application builder pattern
    fn new_static(app_version: String) -> (Self, Task<Message>) {
        (
            VmmAbout {
                app_version,
            },
            Task::none(),
        )
    }

    /// Static update function for the application builder pattern
    fn update_static(state: &mut Self, message: Message) -> Task<Message> {
        state.update(message)
    }

    /// Static view function for the application builder pattern
    fn view_static(state: &Self) -> Element<'_, Message> {
        state.view()
    }
}

/// Singleton wrapper for managing the About dialog instance
pub struct AboutDialogManager {
    instance: Arc<Mutex<Option<VmmAbout>>>,
}

impl AboutDialogManager {
    /// Get the singleton instance
    pub fn get_instance() -> &'static Self {
        static INSTANCE: std::sync::OnceLock<AboutDialogManager> = std::sync::OnceLock::new();
        INSTANCE.get_or_init(|| AboutDialogManager {
            instance: Arc::new(Mutex::new(None)),
        })
    }

    /// Show the About dialog
    ///
    /// # Arguments
    /// * `app_version` - The application version string to display
    /// * `error_callback` - Optional callback for error handling
    pub fn show<F>(&self, app_version: &str, error_callback: Option<F>)
    where
        F: FnOnce(String),
    {
        match VmmAbout::show_instance(app_version) {
            Ok(_) => {
                debug!("About dialog shown successfully");
            }
            Err(e) => {
                if let Some(callback) = error_callback {
                    callback(e);
                }
            }
        }
    }

    /// Close the About dialog
    pub fn close(&self) -> i32 {
        debug!("Closing about dialog from manager");
        let mut instance = self.instance.lock().unwrap();
        *instance = None;
        1
    }

    /// Cleanup resources
    pub fn cleanup(&self) {
        debug!("Cleaning up AboutDialogManager");
        let mut instance = self.instance.lock().unwrap();
        *instance = None;
    }
}

impl Drop for AboutDialogManager {
    fn drop(&mut self) {
        debug!("AboutDialogManager instance dropped");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_about_creation() {
        let about = VmmAbout::new("1.0.0".to_string());
        assert_eq!(about.app_version, "1.0.0");
    }

    #[test]
    fn test_manager_singleton() {
        let manager1 = AboutDialogManager::get_instance();
        let manager2 = AboutDialogManager::get_instance();

        // Both should point to the same instance
        assert!(std::ptr::eq(manager1, manager2));
    }

    #[test]
    fn test_close_returns_one() {
        let manager = AboutDialogManager::get_instance();
        assert_eq!(manager.close(), 1);
    }
}

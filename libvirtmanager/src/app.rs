// Main Iced parent application scaffold
// Mirrors high-level behavior of virtManager/virtmanager.py at a minimal level.

use iced::widget::{Column, button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length, Task, Theme};
use log::debug;

use crate::addhardware::{AddHardwareApp, Message as AddHwMsg};

#[derive(Debug, Clone)]
pub enum Message {
    ShowAddHardware,
    CloseAddHardware,
    AddHardware(AddHwMsg),
}

pub struct MainApp {
    show_add_hw: bool,
    add_hw: AddHardwareApp,
}

impl MainApp {
    pub fn new() -> (Self, Task<Message>) {
        let (add_hw, _t) = AddHardwareApp::new_static();
        (
            Self {
                show_add_hw: false,
                add_hw,
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::ShowAddHardware => {
                self.show_add_hw = true;
                Task::none()
            }
            Message::CloseAddHardware => {
                self.show_add_hw = false;
                Task::none()
            }
            Message::AddHardware(inner) => {
                AddHardwareApp::update_static(&mut self.add_hw, inner).map(Message::AddHardware)
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let header = row![
            text("Virtual Machine Manager (Rust)").size(20),
            iced::widget::Space::with_width(Length::Fill),
            button("Add Hardware").on_press(Message::ShowAddHardware),
        ]
        .align_y(Alignment::Center)
        .spacing(10);

        let mut body: Column<Message> =
            column![text("Connections and VMs list (placeholder)")].spacing(8);

        if self.show_add_hw {
            body = body.push(
                container(AddHardwareApp::view_static(&self.add_hw).map(Message::AddHardware))
                    .padding(10)
                    .width(Length::Fill),
            );
        }

        let content = column![header, scrollable(body).height(Length::Fill)]
            .padding(12)
            .spacing(12);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

pub fn run() -> Result<(), String> {
    use iced::{application, window};

    debug!("Starting parent Iced MainApp");
    application("Virtual Machine Manager", update, view)
        .theme(|_| Theme::default())
        .window(window::Settings {
            size: iced::Size::new(1200.0, 800.0),
            position: window::Position::Centered,
            resizable: true,
            decorations: true,
            ..Default::default()
        })
        .run_with(MainApp::new)
        .map_err(|e| format!("Error starting main app: {}", e))
}

fn update(state: &mut MainApp, msg: Message) -> Task<Message> {
    state.update(msg)
}

fn view(state: &MainApp) -> Element<'_, Message> {
    state.view()
}

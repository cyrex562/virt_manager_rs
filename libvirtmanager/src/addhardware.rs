// Add Hardware dialog (Iced port of virtManager/addhardware.py)
//
// Copyright (C) 2025
// This work is licensed under the GNU GPLv2 or later.

use iced::widget::{
    Column, button, checkbox, column, container, pick_list, row, scrollable, text, text_input,
};
use iced::{Alignment, Element, Length, Task, Theme, window};
use log::debug;
use quick_xml::de::from_str as from_xml_str;
use quick_xml::se::to_string as to_xml_string;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use std::process::Command;
use tokio::fs;
use tokio::task;

// =====================
// Serde XML structures
// =====================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "graphics")]
pub struct DeviceGraphicsXml {
    #[serde(rename = "type", default)]
    gtype: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    passwd: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    gl: Option<GlAttr>,

    #[serde(skip_serializing_if = "Option::is_none")]
    rendernode: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    listen: Option<ListenAttr>,

    #[serde(skip_serializing_if = "Option::is_none")]
    port: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GlAttr {
    #[serde(rename = "enable")]
    enable: String, // "yes"|"no"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ListenAttr {
    #[serde(rename = "type")]
    ltype: String, // "address"|"none"

    #[serde(skip_serializing_if = "Option::is_none")]
    address: Option<String>,
}

impl DeviceGraphicsXml {
    fn from_state(s: &AddHardwareApp) -> Self {
        let gl = if s.gfx_type == "spice" {
            Some(GlAttr {
                enable: if s.gfx_opengl {
                    "yes".into()
                } else {
                    "no".into()
                },
            })
        } else {
            None
        };

        let listen = if s.gfx_listen_kind == "none" {
            Some(ListenAttr {
                ltype: "none".into(),
                address: None,
            })
        } else {
            let addr = if s.gfx_address_selected == "Default" {
                None
            } else {
                Some(s.gfx_address_selected.clone())
            };
            Some(ListenAttr {
                ltype: "address".into(),
                address: addr,
            })
        };

        let port = if s.gfx_listen_kind == "none" {
            None
        } else if s.gfx_port_auto {
            Some(-1)
        } else {
            Some(s.gfx_port_value)
        };

        let rendernode = if s.gfx_opengl {
            if s.gfx_rendernode_selected == "Auto" {
                None
            } else {
                Some(s.gfx_rendernode_selected.clone())
            }
        } else {
            None
        };

        Self {
            gtype: s.gfx_type.clone(),
            passwd: if s.gfx_password_enabled {
                Some(s.gfx_password.clone())
            } else {
                None
            },
            gl,
            rendernode,
            listen,
            port,
        }
    }
}

/// Public facade to show the Add Hardware dialog
pub struct VmmAddHardware;

impl VmmAddHardware {
    /// Show the Add Hardware dialog as a new window. Backend is optional for now.
    pub fn show_instance() -> Result<(), String> {
        debug!("Launching Add Hardware dialog window");

        match iced::application(
            "Add Virtual Hardware",
            AddHardwareApp::update_static,
            AddHardwareApp::view_static,
        )
        .theme(|_| Theme::default())
        .window(window::Settings {
            size: iced::Size::new(900.0, 600.0),
            position: window::Position::Centered,
            resizable: true,
            decorations: true,
            ..Default::default()
        })
        .run_with(AddHardwareApp::new_static)
        {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Error launching 'Add Hardware' dialog: {}", e)),
        }
    }
}

/// Minimal trait surface for VM/backend interactions we will need.
/// Replace with a proper libvirt backend once available.
pub trait VmBackend: Send + Sync {
    fn name(&self) -> &str;
}

/// Hardware pages based on the original Python constants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Storage,
    Controller,
    Network,
    Input,
    Graphics,
    Sound,
    Hostdev,
    Char,
    Video,
    Watchdog,
    Filesystem,
    Smartcard,
    UsbRedir,
    Tpm,
    Rng,
    Panic,
    Vsock,
}

impl Page {
    fn title(&self) -> &'static str {
        match self {
            Page::Storage => "Storage",
            Page::Controller => "Controller",
            Page::Network => "Network",
            Page::Input => "Input",
            Page::Graphics => "Graphics",
            Page::Sound => "Sound",
            Page::Hostdev => "Host Device",
            Page::Char => "Char Device",
            Page::Video => "Video Device",
            Page::Watchdog => "Watchdog",
            Page::Filesystem => "Filesystem",
            Page::Smartcard => "Smartcard",
            Page::UsbRedir => "USB Redirection",
            Page::Tpm => "TPM",
            Page::Rng => "Random Number Generator",
            Page::Panic => "Panic Notifier",
            Page::Vsock => "VM Sockets",
        }
    }
}

/// Hardware list entry for the left navigation
#[derive(Debug, Clone, PartialEq, Eq)]
struct HwEntry {
    label: &'static str,
    page: Page,
    enabled: bool,
    tooltip: Option<&'static str>,
}

/// App-level messages
#[derive(Debug, Clone)]
pub enum Message {
    SelectPage(Page),
    Finish,
    Cancel,
    // Per-page messages (expand incrementally)
    StorageChanged(StorageMsg),
    NetworkChanged(NetworkMsg),
    GraphicsChanged(GraphicsMsg),
    GraphicsEdited(Result<DeviceGraphicsXml, String>),
}

/// Storage page messages (placeholder)
#[derive(Debug, Clone)]
pub enum StorageMsg {
    DeviceTypeChanged(String),
    BusChanged(String),
    PathChanged(String),
}

/// Network page messages (placeholder)
#[derive(Debug, Clone)]
pub enum NetworkMsg {
    ModelChanged(String),
    MacToggle(bool),
    MacChanged(String),
}

/// Graphics page messages
#[derive(Debug, Clone)]
pub enum GraphicsMsg {
    TypeChanged(String),
    ListenKindChanged(String), // "address" | "none"
    AddressChanged(String),
    PortAutoToggle(bool),
    PortChanged(i32),
    PasswordToggle(bool),
    PasswordChanged(String),
    OpenGlToggle(bool),
    RenderNodeChanged(String),
    EditXml,
}

/// Application state
pub struct AddHardwareApp {
    entries: Vec<HwEntry>,
    current: Page,

    // Storage state (minimal placeholders)
    storage_device_type: String,
    storage_bus: String,
    storage_path: String,

    // Network state (minimal placeholders)
    net_model_options: Vec<String>, // includes "Default"
    net_model_selected: String,
    net_mac_enabled: bool,
    net_mac: String,

    // Graphics state
    gfx_type: String,                 // spice | vnc
    gfx_listen_kind: String,          // address | none
    gfx_address_options: Vec<String>, // includes "Default"
    gfx_address_selected: String,
    gfx_port_auto: bool,
    gfx_port_value: i32,
    gfx_password_enabled: bool,
    gfx_password: String,
    gfx_opengl: bool,
    gfx_rendernode_options: Vec<String>, // includes "Auto"
    gfx_rendernode_selected: String,
    gfx_status: Option<String>, // status/info banner (e.g., editor not set)
    gfx_temp_xml_path: Option<PathBuf>,
}

impl AddHardwareApp {
    fn new() -> (Self, Task<Message>) {
        let entries = vec![
            HwEntry {
                label: "Storage",
                page: Page::Storage,
                enabled: true,
                tooltip: None,
            },
            HwEntry {
                label: "Controller",
                page: Page::Controller,
                enabled: true,
                tooltip: None,
            },
            HwEntry {
                label: "Network",
                page: Page::Network,
                enabled: true,
                tooltip: None,
            },
            HwEntry {
                label: "Input",
                page: Page::Input,
                enabled: true,
                tooltip: None,
            },
            HwEntry {
                label: "Graphics",
                page: Page::Graphics,
                enabled: true,
                tooltip: None,
            },
            HwEntry {
                label: "Sound",
                page: Page::Sound,
                enabled: true,
                tooltip: None,
            },
            HwEntry {
                label: "Serial/Console/Channel",
                page: Page::Char,
                enabled: true,
                tooltip: None,
            },
            HwEntry {
                label: "USB/PCI Hostdev",
                page: Page::Hostdev,
                enabled: true,
                tooltip: None,
            },
            HwEntry {
                label: "Video",
                page: Page::Video,
                enabled: true,
                tooltip: None,
            },
            HwEntry {
                label: "Watchdog",
                page: Page::Watchdog,
                enabled: true,
                tooltip: None,
            },
            HwEntry {
                label: "Filesystem",
                page: Page::Filesystem,
                enabled: true,
                tooltip: None,
            },
            HwEntry {
                label: "Smartcard",
                page: Page::Smartcard,
                enabled: true,
                tooltip: None,
            },
            HwEntry {
                label: "USB Redirection",
                page: Page::UsbRedir,
                enabled: true,
                tooltip: None,
            },
            HwEntry {
                label: "TPM",
                page: Page::Tpm,
                enabled: true,
                tooltip: None,
            },
            HwEntry {
                label: "RNG",
                page: Page::Rng,
                enabled: true,
                tooltip: None,
            },
            HwEntry {
                label: "Panic",
                page: Page::Panic,
                enabled: true,
                tooltip: None,
            },
            HwEntry {
                label: "Vsock",
                page: Page::Vsock,
                enabled: true,
                tooltip: None,
            },
        ];

        let state = AddHardwareApp {
            entries,
            current: Page::Storage,
            storage_device_type: "disk".to_string(),
            storage_bus: "virtio".to_string(),
            storage_path: String::new(),
            net_model_options: vec![
                "Default".into(),
                "virtio".into(),
                "e1000".into(),
                "rtl8139".into(),
            ],
            net_model_selected: "Default".into(),
            net_mac_enabled: true,
            net_mac: String::new(),
            gfx_type: "spice".into(),
            gfx_listen_kind: "address".into(),
            gfx_address_options: vec!["Default".into(), "127.0.0.1".into(), "0.0.0.0".into()],
            gfx_address_selected: "Default".into(),
            gfx_port_auto: true,
            gfx_port_value: 0,
            gfx_password_enabled: false,
            gfx_password: String::new(),
            gfx_opengl: false,
            gfx_rendernode_options: vec!["Auto".into()], // backend will append DRM render nodes
            gfx_rendernode_selected: "Auto".into(),
            gfx_status: None,
            gfx_temp_xml_path: None,
        };

        (state, Task::none())
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::SelectPage(p) => {
                self.current = p;
                Task::none()
            }
            Message::Finish => {
                debug!("Finish pressed on page: {:?}", self.current);
                window::get_latest().and_then(window::close)
            }
            Message::Cancel => window::get_latest().and_then(window::close),
            Message::StorageChanged(smsg) => {
                match smsg {
                    StorageMsg::DeviceTypeChanged(s) => self.storage_device_type = s,
                    StorageMsg::BusChanged(s) => self.storage_bus = s,
                    StorageMsg::PathChanged(s) => self.storage_path = s,
                }
                Task::none()
            }
            Message::NetworkChanged(nmsg) => {
                match nmsg {
                    NetworkMsg::ModelChanged(m) => self.net_model_selected = m,
                    NetworkMsg::MacToggle(v) => self.net_mac_enabled = v,
                    NetworkMsg::MacChanged(v) => self.net_mac = v,
                }
                Task::none()
            }
            Message::GraphicsChanged(gmsg) => {
                match gmsg {
                    GraphicsMsg::TypeChanged(t) => self.gfx_type = t,
                    GraphicsMsg::ListenKindChanged(k) => self.gfx_listen_kind = k,
                    GraphicsMsg::AddressChanged(a) => self.gfx_address_selected = a,
                    GraphicsMsg::PortAutoToggle(v) => self.gfx_port_auto = v,
                    GraphicsMsg::PortChanged(v) => self.gfx_port_value = v,
                    GraphicsMsg::PasswordToggle(v) => {
                        self.gfx_password_enabled = v;
                        if !v {
                            self.gfx_password.clear();
                        }
                    }
                    GraphicsMsg::PasswordChanged(p) => self.gfx_password = p,
                    GraphicsMsg::OpenGlToggle(v) => self.gfx_opengl = v,
                    GraphicsMsg::RenderNodeChanged(v) => self.gfx_rendernode_selected = v,
                    GraphicsMsg::EditXml => {
                        return self.launch_graphics_xml_editor();
                    }
                }
                Task::none()
            }
            Message::GraphicsEdited(result) => {
                match result {
                    Ok(devxml) => {
                        self.apply_graphics_from_xml(devxml);
                        self.gfx_status = Some("Applied changes from XML.".into());
                    }
                    Err(e) => {
                        self.gfx_status = Some(format!("Failed to apply XML: {}", e));
                    }
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let sidebar = self.view_sidebar();
        let page = self.view_page();

        let footer = row![
            button(text("Cancel")).on_press(Message::Cancel),
            iced::widget::Space::with_width(Length::Fill),
            button(text("Finish")).on_press(Message::Finish),
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .padding([10, 0]);

        let content = row![sidebar, page].spacing(16).height(Length::Fill);

        column![content, footer].padding(16).spacing(10).into()
    }

    fn view_sidebar(&self) -> Element<'_, Message> {
        let mut col: Column<Message> = column![text("Hardware").size(18)].spacing(8);

        for e in &self.entries {
            let mut b = button(text(e.label).width(Length::Fill)).padding(8);
            if e.enabled {
                b = b.on_press(Message::SelectPage(e.page));
            }
            let row = container(b).width(Length::Fill).padding(2);
            col = col.push(row);
        }

        container(scrollable(col))
            .width(Length::Fixed(220.0))
            .height(Length::Fill)
            .padding(8)
            .into()
    }

    fn view_page(&self) -> Element<'_, Message> {
        let title = text(self.current.title()).size(20);

        let body: Element<'_, Message> = match self.current {
            Page::Storage => self.view_storage_page(),
            Page::Network => self.view_network_page(),
            Page::Graphics => self.view_graphics_page(),
            _ => container(
                text("This page is not implemented yet.")
                    .size(14)
                    .width(Length::Fill),
            )
            .center_x(Length::Fill)
            .center_y(Length::Shrink)
            .into(),
        };

        container(column![title, body].spacing(12))
            .width(Length::Fill)
            .padding(8)
            .into()
    }

    fn view_storage_page(&self) -> Element<'_, Message> {
        // Simple placeholders for Device Type, Bus, and Path
        let device_types = vec![
            "disk".to_string(),
            "cdrom".to_string(),
            "floppy".to_string(),
            "lun".to_string(),
        ];
        let buses = vec![
            "virtio".to_string(),
            "sata".to_string(),
            "scsi".to_string(),
            "ide".to_string(),
            "usb".to_string(),
        ];

        let dev_type = pick_list(
            device_types.clone(),
            Some(self.storage_device_type.clone()),
            |v| Message::StorageChanged(StorageMsg::DeviceTypeChanged(v)),
        );

        let bus = pick_list(buses.clone(), Some(self.storage_bus.clone()), |v| {
            Message::StorageChanged(StorageMsg::BusChanged(v))
        });

        let path = text_input("/path/to/disk.img", &self.storage_path)
            .on_input(|s| Message::StorageChanged(StorageMsg::PathChanged(s)))
            .padding(8);

        let grid = column![
            row![text("Device type:"), dev_type]
                .spacing(8)
                .align_y(Alignment::Center),
            row![text("Bus:"), bus]
                .spacing(8)
                .align_y(Alignment::Center),
            row![text("Source:"), path]
                .spacing(8)
                .align_y(Alignment::Center),
        ]
        .spacing(10)
        .padding(8);

        container(grid).into()
    }

    fn view_network_page(&self) -> Element<'_, Message> {
        let model_pick = pick_list(
            self.net_model_options.clone(),
            Some(self.net_model_selected.clone()),
            |v| Message::NetworkChanged(NetworkMsg::ModelChanged(v)),
        );

        let mac_toggle_label = if self.net_mac_enabled {
            "Use this MAC"
        } else {
            "Random MAC"
        };
        let mac_toggle = button(mac_toggle_label)
            .on_press(Message::NetworkChanged(NetworkMsg::MacToggle(
                !self.net_mac_enabled,
            )))
            .padding(6);

        let mac_input = text_input("52:54:00:ab:cd:ef", &self.net_mac)
            .on_input(|s| Message::NetworkChanged(NetworkMsg::MacChanged(s)))
            .padding(8);

        let grid = column![
            row![text("Model:"), model_pick]
                .spacing(8)
                .align_y(Alignment::Center),
            row![text("MAC:"), mac_toggle, mac_input]
                .spacing(8)
                .align_y(Alignment::Center),
        ]
        .spacing(10)
        .padding(8);

        container(grid).into()
    }

    fn view_graphics_page(&self) -> Element<'_, Message> {
        // Graphics type
        let gfx_types = vec!["spice".to_string(), "vnc".to_string()];
        let gfx_type_pick = pick_list(gfx_types, Some(self.gfx_type.clone()), |v| {
            Message::GraphicsChanged(GraphicsMsg::TypeChanged(v))
        });

        // Listen kind and address
        let listen_kinds = vec!["address".to_string(), "none".to_string()];
        let listen_pick = pick_list(listen_kinds, Some(self.gfx_listen_kind.clone()), |v| {
            Message::GraphicsChanged(GraphicsMsg::ListenKindChanged(v))
        });

        let addr_pick = pick_list(
            self.gfx_address_options.clone(),
            Some(self.gfx_address_selected.clone()),
            |v| Message::GraphicsChanged(GraphicsMsg::AddressChanged(v)),
        );

        // Port auto + value
        let auto_btn = checkbox("Auto port", self.gfx_port_auto)
            .on_toggle(|v| Message::GraphicsChanged(GraphicsMsg::PortAutoToggle(v)));
        let port_input = text_input("5900", &self.gfx_port_value.to_string())
            .on_input(|s| {
                let val = s.parse::<i32>().unwrap_or(0);
                Message::GraphicsChanged(GraphicsMsg::PortChanged(val))
            })
            .padding(8);

        // Password
        let pass_toggle = checkbox("Use password", self.gfx_password_enabled)
            .on_toggle(|v| Message::GraphicsChanged(GraphicsMsg::PasswordToggle(v)));
        let pass_input = text_input("••••••", &self.gfx_password)
            .on_input(|s| Message::GraphicsChanged(GraphicsMsg::PasswordChanged(s)))
            .secure(true)
            .padding(8);

        // OpenGL + rendernode
        let gl_toggle = checkbox("Enable OpenGL (SPICE)", self.gfx_opengl)
            .on_toggle(|v| Message::GraphicsChanged(GraphicsMsg::OpenGlToggle(v)));
        let render_pick = pick_list(
            self.gfx_rendernode_options.clone(),
            Some(self.gfx_rendernode_selected.clone()),
            |v| Message::GraphicsChanged(GraphicsMsg::RenderNodeChanged(v)),
        );

        // XML edit button
        let edit_xml_btn = button("Edit XML…")
            .on_press(Message::GraphicsChanged(GraphicsMsg::EditXml))
            .padding(8);

        let mut grid: Column<Message> = column![
            row![text("Type:"), gfx_type_pick]
                .spacing(8)
                .align_y(Alignment::Center),
            row![text("Listen:"), listen_pick]
                .spacing(8)
                .align_y(Alignment::Center),
        ]
        .spacing(10)
        .padding(8);

        if self.gfx_listen_kind == "address" {
            grid = grid.push(
                row![text("Address:"), addr_pick]
                    .spacing(8)
                    .align_y(Alignment::Center),
            );
            grid = grid.push(
                row![text("Port:"), auto_btn, port_input]
                    .spacing(8)
                    .align_y(Alignment::Center),
            );
        }

        // Password row (applies to both VNC and SPICE)
        grid = grid.push(
            row![text("Password:"), pass_toggle, pass_input]
                .spacing(8)
                .align_y(Alignment::Center),
        );

        // OpenGL/SPICE-only
        if self.gfx_type == "spice" {
            grid = grid.push(
                row![text("OpenGL:"), gl_toggle]
                    .spacing(8)
                    .align_y(Alignment::Center),
            );
            if self.gfx_opengl {
                grid = grid.push(
                    row![text("Render node:"), render_pick]
                        .spacing(8)
                        .align_y(Alignment::Center),
                );
            }
        }

        // Status/info and XML edit
        if let Some(status) = &self.gfx_status {
            grid = grid.push(text(status.clone()).size(14));
        }
        grid = grid.push(edit_xml_btn);

        container(grid).into()
    }

    fn launch_graphics_xml_editor(&mut self) -> Task<Message> {
        // Build current graphics XML
        let xml = self.graphics_xml_string();
        return match tempfile::Builder::new()
            .prefix("vmm-graphics-")
            .suffix(".xml")
            .tempfile()
        {
            Ok(mut tf) => {
                use std::io::Write;
                if let Err(e) = write!(tf, "{}\n", xml) {
                    self.gfx_status = Some(format!("Failed writing temp XML: {}", e));
                    return Task::none();
                }
                // Persist the temp file so it isn't deleted when `tf` is dropped
                let (_file, pathbuf) = match tf.keep() {
                    Ok((f, p)) => (f, p),
                    Err(e) => {
                        self.gfx_status = Some(format!("Failed to persist temp XML: {}", e.error));
                        return Task::none();
                    }
                };
                self.gfx_temp_xml_path = Some(pathbuf.clone());

                // Choose editor
                let editor = env::var("VISUAL").or_else(|_| env::var("EDITOR")).ok();
                if let Some(ed) = editor {
                    match Command::new(ed).arg(&pathbuf).spawn() {
                        Ok(mut child) => {
                            let p = pathbuf.clone();
                            Task::perform(
                                async move {
                                    let wait_res = task::spawn_blocking(move || child.wait()).await;
                                    match wait_res {
                                        Ok(Ok(_status)) => match fs::read_to_string(&p).await {
                                            Ok(contents) => {
                                                match from_xml_str::<DeviceGraphicsXml>(
                                                    contents.trim(),
                                                ) {
                                                    Ok(devxml) => Ok(devxml),
                                                    Err(e) => {
                                                        Err(format!("XML parse error: {}", e))
                                                    }
                                                }
                                            }
                                            Err(e) => Err(format!("Read temp file failed: {}", e)),
                                        },
                                        Ok(Err(e)) => Err(format!("Editor wait failed: {}", e)),
                                        Err(e) => Err(format!("Join error waiting editor: {}", e)),
                                    }
                                },
                                Message::GraphicsEdited,
                            )
                        }
                        Err(e) => {
                            self.gfx_status = Some(format!("Failed to launch editor: {}", e));
                            Task::none()
                        }
                    }
                } else {
                    self.gfx_status = Some(
                        "No default editor set (VISUAL/EDITOR). Please set one to edit XML.".into(),
                    );
                    Task::none()
                }
            }
            Err(e) => {
                self.gfx_status = Some(format!("Failed to create temp file: {}", e));
                Task::none()
            }
        };
    }

    fn graphics_xml_string(&self) -> String {
        let xml = DeviceGraphicsXml::from_state(self);
        match to_xml_string(&xml) {
            Ok(mut s) => {
                // quick-xml won't add the root <graphics> tag name unless configured; we use serde rename
                // Wrap in a domain device element if needed later. For now, return the <graphics/> snippet.
                // Add XML header for editor usability.
                if !s.starts_with("<?xml") {
                    s = format!("{}{}", "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n", s);
                }
                s
            }
            Err(e) => format!("<!-- Serialization error: {} -->", e),
        }
    }

    fn apply_graphics_from_xml(&mut self, x: DeviceGraphicsXml) {
        if !x.gtype.is_empty() {
            self.gfx_type = x.gtype;
        }

        if let Some(lst) = x.listen {
            match lst.ltype.as_str() {
                "none" => {
                    self.gfx_listen_kind = "none".into();
                    self.gfx_address_selected = "Default".into();
                    self.gfx_port_auto = true;
                }
                _ => {
                    self.gfx_listen_kind = "address".into();
                    self.gfx_address_selected = lst.address.unwrap_or_else(|| "Default".into());
                }
            }
        }

        if let Some(p) = x.port {
            if p == -1 {
                self.gfx_port_auto = true;
                self.gfx_port_value = 0;
            } else {
                self.gfx_port_auto = false;
                self.gfx_port_value = p;
            }
        }

        match x.passwd {
            Some(p) => {
                self.gfx_password_enabled = true;
                self.gfx_password = p;
            }
            None => {
                self.gfx_password_enabled = false;
                self.gfx_password.clear();
            }
        }

        if let Some(gl) = x.gl {
            self.gfx_opengl = gl.enable == "yes";
        } else {
            self.gfx_opengl = false;
        }
        self.gfx_rendernode_selected = x.rendernode.unwrap_or_else(|| "Auto".into());
    }

    // Static adapter functions for iced::application (public for embedding)
    pub fn new_static() -> (Self, Task<Message>) {
        Self::new()
    }
    pub fn update_static(state: &mut Self, msg: Message) -> Task<Message> {
        state.update(msg)
    }
    pub fn view_static(state: &Self) -> Element<'_, Message> {
        state.view()
    }
}

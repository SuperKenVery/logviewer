use crate::netinfo::{get_network_interfaces, InterfaceInfo};
use std::net::IpAddr;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum ListenDisplayMode {
    #[default]
    AddrPort,
    NcCommand,
}

#[derive(Clone)]
pub struct ListenAddrEntry {
    pub ip: IpAddr,
    pub is_v6: bool,
    #[allow(dead_code)]
    pub is_self_assigned: bool,
    pub row: u16,
}

pub struct ListenState {
    pub port: Option<u16>,
    pub has_connection: bool,
    pub network_interfaces: Vec<InterfaceInfo>,
    pub display_mode: ListenDisplayMode,
    pub addr_list: Vec<ListenAddrEntry>,
    pub selected_idx: usize,
    pub popup_area: Option<(u16, u16, u16, u16)>,
}

impl ListenState {
    pub fn new(port: Option<u16>) -> Self {
        let network_interfaces = if port.is_some() {
            get_network_interfaces()
        } else {
            Vec::new()
        };
        Self {
            port,
            has_connection: false,
            network_interfaces,
            display_mode: ListenDisplayMode::default(),
            addr_list: Vec::new(),
            selected_idx: 0,
            popup_area: None,
        }
    }

    pub fn show_popup(&self) -> bool {
        self.port.is_some() && !self.has_connection
    }

    pub fn toggle_display_mode(&mut self) {
        self.display_mode = match self.display_mode {
            ListenDisplayMode::AddrPort => ListenDisplayMode::NcCommand,
            ListenDisplayMode::NcCommand => ListenDisplayMode::AddrPort,
        };
    }

    pub fn select_next(&mut self) {
        if !self.addr_list.is_empty() {
            self.selected_idx = (self.selected_idx + 1) % self.addr_list.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.addr_list.is_empty() {
            self.selected_idx = self
                .selected_idx
                .checked_sub(1)
                .unwrap_or(self.addr_list.len() - 1);
        }
    }

    pub fn get_selected_copy_text(&self) -> Option<String> {
        let port = self.port?;
        let entry = self.addr_list.get(self.selected_idx)?;
        Some(match self.display_mode {
            ListenDisplayMode::AddrPort => {
                if entry.is_v6 {
                    format!("[{}]:{}", entry.ip, port)
                } else {
                    format!("{}:{}", entry.ip, port)
                }
            }
            ListenDisplayMode::NcCommand => {
                if entry.is_v6 {
                    format!("nc -6 {} {}", entry.ip, port)
                } else {
                    format!("nc {} {}", entry.ip, port)
                }
            }
        })
    }

    pub fn handle_click(&mut self, x: u16, y: u16) -> Option<String> {
        let (px, py, pw, ph) = self.popup_area?;
        if x < px || x >= px + pw || y < py || y >= py + ph {
            return None;
        }

        for (idx, entry) in self.addr_list.iter().enumerate() {
            if y == py + entry.row {
                self.selected_idx = idx;
                return self.get_selected_copy_text();
            }
        }
        None
    }
}

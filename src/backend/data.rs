use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level structure matching `debug.json`.  
#[derive(Debug, Serialize, Deserialize)]
pub struct Root {
    pub proxies: HashMap<String, ProxyEntry>,
}

/// An entry under `proxies`, either a group (has `all`) or a proxy (has `id`).  
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProxyEntry {
    Group(ProxyGroup),
    Proxy(Proxy),
}

/// A proxy group (has `all` field).
#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyGroup {
    // in use
    pub all: Vec<String>,
    pub history: Vec<HistoryEntry>,
    pub name: String,
    #[serde(default)]
    pub now: Option<String>,
    #[serde(rename = "type")]
    pub typ: ProxyType,
    // not in use
    // pub alive: bool,
    // #[serde(rename = "dialer-proxy")]
    // pub dialer_proxy: String,
    // pub extra: HashMap<String, ExtraInfo>,
    // pub hidden: bool,
    // pub icon: String,
    // pub interface: String,
    // pub mptcp: bool,
    // #[serde(rename = "routing-mark")]
    // pub routing_mark: u64,
    // pub smux: bool,
    // pub tfo: bool,
    // pub udp: bool,
    // pub uot: bool,
    // pub xudp: bool,
    // pub fixed: Option<String>,
    // #[serde(rename = "expectedStatus")]
    // pub expected_status: Option<String>,
    // #[serde(rename = "testUrl")]
    // pub test_url: Option<String>,
}

/// A single proxy (has `id` field).
#[derive(Debug, Serialize, Deserialize)]
pub struct Proxy {
    // in use
    pub id: String,
    pub name: String,
    pub udp: bool,
    pub history: Vec<HistoryEntry>,
    #[serde(rename = "type")]
    pub typ: ProxyType,
    // not in use
    // pub alive: bool,
    // #[serde(rename = "dialer-proxy")]
    // pub dialer_proxy: String,
    // pub extra: HashMap<String, ExtraInfo>,
    // pub interface: String,
    // pub mptcp: bool,
    // #[serde(rename = "routing-mark")]
    // pub routing_mark: u64,
    // pub smux: bool,
    // pub tfo: bool,
    // pub uot: bool,
    // pub xudp: bool,
}

/// A single history record, under `history` or inside `extra`.
#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub time: String,
    pub delay: u64,
}

/// Additional info under the `extra` map.
#[derive(Debug, Serialize, Deserialize)]
pub struct ExtraInfo {
    pub alive: bool,
    pub history: Vec<HistoryEntry>,
}

/// Type of proxy or group, matching the `type` field.
#[derive(Debug, Serialize, Deserialize)]
// #[serde(rename_all = "PascalCase")]
pub enum ProxyType {
    Direct,
    Reject,
    RejectDrop,
    Compatible,
    Pass,

    Shadowsocks,
    ShadowsocksR,
    Snell,
    Socks5,
    Http,
    Vmess,
    Vless,
    Trojan,
    Hysteria,
    Hysteria2,
    Tuic,
    WireGuard,
    Dns,
    Ssh,
    Mieru,
    AnyTLS,

    // group specific
    Relay,
    Selector,
    Fallback,
    URLTest,
    LoadBalance,
}

mod test {
    #[test]
    fn test_proxy_group() {
        use super::*;
        let json = include_str!("../../test.json");
        serde_json::from_str::<Root>(json).unwrap();
    }
}

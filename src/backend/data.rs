use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum_macros::IntoStaticStr;

/// Top-level structure matching `debug.json`.  
#[derive(Debug, Serialize, Deserialize)]
pub struct Root {
    pub proxies: HashMap<String, ProxyEntryRaw>,
}

/// An entry under `proxies`, either a group (has `all`) or a proxy (has `id`).  
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProxyEntryRaw {
    Group(ProxyGroupRaw),
    Proxy(ProxyRaw),
}

/// A proxy group (has `all` field).
#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyGroupRaw {
    // in use
    pub all: Vec<String>,
    pub history: Vec<HistoryEntry>,
    pub name: String,
    #[serde(default)]
    pub now: Option<String>,
    #[serde(rename = "type")]
    pub typ: ProxyType,
    pub udp: bool,
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
pub struct ProxyRaw {
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
#[derive(Debug, Clone, Copy, Serialize, Deserialize, IntoStaticStr)]
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
impl ProxyType {
    pub fn is_group(&self) -> bool {
        matches!(
            self,
            ProxyType::Relay
                | ProxyType::Selector
                | ProxyType::Fallback
                | ProxyType::URLTest
                | ProxyType::LoadBalance
        )
    }
    pub fn str(&self) -> &'static str {
        self.into()
    }
}

#[derive(Debug, Deserialize)]
pub struct Provider {
    pub name: String,
    #[serde(rename = "vehicleType")]
    pub vehicle_type: String,
    #[serde(rename = "subscriptionInfo")]
    pub subscription_info: Option<SubscriptionInfo>,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub proxies: Vec<ProxyEntryRaw>,
    //   testUrl: string
}

#[derive(Debug, Deserialize)]
pub struct SubscriptionInfo {
    #[serde(rename = "Download")]
    pub download: Option<u64>,
    #[serde(rename = "Upload")]
    pub upload: Option<u64>,
    #[serde(rename = "Total")]
    pub total: Option<u64>,
    #[serde(rename = "Expire")]
    pub expire: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ProviderRoot {
    #[serde(deserialize_with = "deserialize_providers")]
    pub providers: HashMap<String, Provider>,
}

fn deserialize_providers<'de, D>(deserializer: D) -> Result<HashMap<String, Provider>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct ProviderVisitor;
    impl<'de> serde::de::Visitor<'de> for ProviderVisitor {
        type Value = HashMap<String, Provider>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str(
                "
                a map of provider names to provider objects",
            )
        }

        fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
        where
            V: serde::de::MapAccess<'de>,
        {
            let mut providers = HashMap::new();
            while let Some((key, value)) = map.next_entry::<String, Provider>()? {
                if value.name != "default" && value.vehicle_type != "Compatible" {
                    providers.insert(key, value);
                }
            }
            Ok(providers)
        }
    }

    deserializer.deserialize_map(ProviderVisitor)
}

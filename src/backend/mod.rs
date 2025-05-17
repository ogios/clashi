use std::{
    collections::HashMap,
    sync::{LazyLock, atomic::AtomicPtr},
};

use data::{ProxyEntryRaw, ProxyGroupRaw, Root};

mod data;
pub use data::Provider;
use reqwest::Url;

#[derive(Debug)]
pub struct ProxyGroup {
    pub name: String,
    pub now: Option<String>,
    pub proxy_type: data::ProxyType,
    pub udp: bool,
    pub latency: Option<u64>,

    pub proxies: Vec<SelectableProxy>,
}

#[derive(Debug, Clone)]
pub struct SelectableProxy {
    pub name: String,
    pub udp: bool,
    pub proxy_type: data::ProxyType,
    pub latency: Option<u64>,
}
impl SelectableProxy {
    pub fn from_group(group: &ProxyGroupRaw, root: &Root, cache: &HashMap<String, Self>) -> Self {
        let name = group.name.clone();
        if let Some(proxy) = cache.get(&name) {
            return proxy.clone();
        }

        let proxy_type = group.typ;
        let udp = group.udp;
        let latency = if let Some(sele) = cache.get(&name) {
            sele.latency
        } else {
            fn recursive_find_latency(
                name: &str,
                root: &Root,
                cache: &HashMap<String, SelectableProxy>,
            ) -> Option<u64> {
                if let Some(proxy) = cache.get(name) {
                    return proxy.latency;
                }

                root.proxies.get(name).and_then(|entry| match entry {
                    ProxyEntryRaw::Proxy(proxy) => proxy.history.last().map(|entry| entry.delay),
                    ProxyEntryRaw::Group(proxy_group_raw) => proxy_group_raw
                        .history
                        .last()
                        .map(|entry| entry.delay)
                        .or_else(|| {
                            if let Some(now) = proxy_group_raw.now.as_ref() {
                                recursive_find_latency(now, root, cache)
                            } else {
                                None
                            }
                        }),
                })
            }

            group.history.last().map(|entry| entry.delay).or_else(|| {
                group
                    .now
                    .as_ref()
                    .and_then(|now| recursive_find_latency(now, root, cache))
            })
        };

        Self {
            name,
            udp,
            proxy_type,
            latency,
        }
    }

    pub fn from_proxy(proxy: &data::ProxyRaw, cache: &HashMap<String, SelectableProxy>) -> Self {
        let name = proxy.name.clone();
        if let Some(proxy) = cache.get(&name) {
            return proxy.clone();
        }

        let proxy_type = proxy.typ;
        let udp = proxy.udp;
        let latency = if let Some(sele) = cache.get(&name) {
            sele.latency
        } else {
            proxy.history.last().map(|entry| entry.delay)
        };

        Self {
            name,
            udp,
            proxy_type,
            latency,
        }
    }

    pub fn from_proxy_without_cache(proxy: &data::ProxyRaw) -> Self {
        let name = proxy.name.clone();
        let proxy_type = proxy.typ;
        let udp = proxy.udp;
        let latency = proxy.history.last().map(|entry| entry.delay);

        Self {
            name,
            udp,
            proxy_type,
            latency,
        }
    }
}

static BASE_URL: LazyLock<Url> = LazyLock::new(|| Url::parse("http://localhost:9090/").unwrap());

fn get_proxy_groups() -> Vec<ProxyGroup> {
    let url = BASE_URL.join("proxies").unwrap();
    let response: data::Root = reqwest::blocking::get(url).unwrap().json().unwrap();

    let mut raw_proxy_groups = vec![];
    let mut cache: HashMap<String, SelectableProxy> = HashMap::new();

    for (name, v) in response.proxies.iter() {
        match v {
            data::ProxyEntryRaw::Group(proxy_group) => {
                raw_proxy_groups.push(proxy_group);
                cache.insert(
                    name.clone(),
                    SelectableProxy::from_group(proxy_group, &response, &cache),
                );
            }
            ProxyEntryRaw::Proxy(proxy_raw) => {
                cache.insert(name.clone(), SelectableProxy::from_proxy(proxy_raw, &cache));
            }
        }
    }

    let mut groups = raw_proxy_groups
        .into_iter()
        .map(|group| {
            let name = group.name.clone();
            let now = group.now.clone();
            let proxy_type = group.typ;
            let udp = group.udp;
            let latency = cache.get(&group.name).unwrap().latency;

            let proxies = group
                .all
                .iter()
                .map(|name| cache.get(name).unwrap())
                .cloned()
                .collect();

            ProxyGroup {
                name,
                now,
                proxy_type,
                udp,
                latency,
                proxies,
            }
        })
        .collect::<Vec<ProxyGroup>>();
    groups.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    groups
}

pub fn select_proxy(group: &str, proxy: &str) {
    let url = BASE_URL.join(format!("proxies/{group}").as_str()).unwrap();
    let client = reqwest::blocking::Client::new();
    let _ = client
        .put(url)
        .header("Content-Type", "application/json")
        .body(format!(r#"{{"name": "{}"}}"#, proxy))
        .send()
        .unwrap()
        .error_for_status()
        .unwrap();
}

const DEFAULT_LATENCY_TEXT_URL: &str = "https://www.gstatic.com/generate_204";
const TIMEOUT: u64 = 5000;
pub fn latency_test_group(group: &str) {
    let mut url = BASE_URL
        .join(format!("group/{group}/delay").as_str())
        .unwrap();

    url.query_pairs_mut()
        .append_pair("url", DEFAULT_LATENCY_TEXT_URL)
        .append_pair("timeout", &TIMEOUT.to_string());

    let _ = reqwest::blocking::get(url)
        .unwrap()
        .error_for_status()
        .unwrap();
}
pub fn latency_test_proxy(proxy: &str) {
    let mut url = BASE_URL
        .join(format!("proxies/{proxy}/delay").as_str())
        .unwrap();
    url.query_pairs_mut()
        .append_pair("url", DEFAULT_LATENCY_TEXT_URL)
        .append_pair("timeout", &TIMEOUT.to_string());

    let _ = reqwest::blocking::get(url)
        .unwrap()
        .error_for_status()
        .unwrap();
}

fn get_proxy_providers() -> Vec<data::Provider> {
    let url = BASE_URL.join("providers/proxies").unwrap();
    let response: data::ProviderRoot = reqwest::blocking::get(url).unwrap().json().unwrap();

    let mut providers = response.providers.into_values().collect::<Vec<_>>();
    providers.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    providers
}

pub fn update_proxy_provider(provider: &str) {
    let url = BASE_URL
        .join(format!("providers/proxies/{provider}").as_str())
        .unwrap();
    let client = reqwest::blocking::Client::new();
    let _ = client.put(url).send().unwrap().error_for_status().unwrap();
}

pub fn latency_test_provider(provider: &str) {
    let url = BASE_URL
        .join(format!("providers/proxies/{provider}/healthcheck").as_str())
        .unwrap();
    let client = reqwest::blocking::Client::new();
    let _ = client.get(url).send().unwrap().error_for_status().unwrap();
}

use std::sync::atomic::Ordering::*;
static GROUPS_DATA: AtomicPtr<Vec<ProxyGroup>> = AtomicPtr::new(std::ptr::null_mut());
static PROVIDER_DATA: AtomicPtr<Vec<Provider>> = AtomicPtr::new(std::ptr::null_mut());

pub fn refresh_data() {
    let groups = get_proxy_groups();
    let boxed = Box::new(groups);
    let ptr = Box::into_raw(boxed);
    GROUPS_DATA.store(ptr, Relaxed);

    let providers = get_proxy_providers();
    let boxed = Box::new(providers);
    let ptr = Box::into_raw(boxed);
    PROVIDER_DATA.store(ptr, Relaxed);
}

pub fn get_groups_data() -> &'static Vec<ProxyGroup> {
    let ptr = GROUPS_DATA.load(Relaxed);
    if ptr.is_null() {
        refresh_data();
    }

    unsafe { &*GROUPS_DATA.load(Relaxed) }
}

pub fn get_providers_data() -> &'static Vec<Provider> {
    let ptr = PROVIDER_DATA.load(Relaxed);
    if ptr.is_null() {
        refresh_data();
    }
    unsafe { &*PROVIDER_DATA.load(Relaxed) }
}

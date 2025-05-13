use std::collections::HashMap;

use data::{ProxyEntryRaw, ProxyGroupRaw, Root};

pub mod data;

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
    name: String,
    udp: bool,
    proxy_type: data::ProxyType,
    latency: Option<u64>,
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
}

pub fn get_proxy_groups() -> Vec<ProxyGroup> {
    let response: data::Root = reqwest::blocking::get("http://localhost:9090/proxies")
        .unwrap()
        .json()
        .unwrap();

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
        .iter()
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

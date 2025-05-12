pub mod data;

pub fn get_proxy_groups() -> Vec<data::ProxyGroup> {
    let response: data::Root = reqwest::blocking::get("http://localhost:9090/proxies")
        .unwrap()
        .json()
        .unwrap();
    response
        .proxies
        .into_iter()
        .filter_map(|(_, v)| match v {
            data::ProxyEntry::Group(proxy_group) => Some(proxy_group),
            _ => None,
        })
        .collect()
}

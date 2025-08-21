use anyhow::Result;

use once_cell::sync::Lazy;

use super::{Device, DeviceList};

use super::DeviceType;
use crate::parsers::utils::{static_user_agent_match, SafeRegex as Regex};

static DEVICE_LIST: Lazy<DeviceList> = Lazy::new(|| {
    let contents = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/regexes/device/televisions.yml"
    ));
    DeviceList::from_file(contents).expect("loading televisions.yml")
});
static HBTV: Lazy<Regex> = static_user_agent_match!(r#"HbbTV/([1-9]{1}(?:\.[0-9]{1}){1,2})"#);
static CE_HTML: Lazy<Regex> = static_user_agent_match!(r#"CE-HTML"#);

pub fn is_hbbtv(ua: &str) -> Result<bool> {
    let res = HBTV.is_match(ua)?;
    Ok(res)
}

pub fn lookup(ua: &str) -> Result<Option<Device>> {
    // Check for HbbTV or CE-HTML (both indicate TV-like devices)
    if !is_hbbtv(ua)? && !CE_HTML.is_match(ua)? {
        return Ok(None);
    }

    let res = DEVICE_LIST.lookup(ua, "tv")?.map(|mut res| {
        // Only set device type to Television if not already set (e.g., could be Peripheral)
        if res.device_type.is_none() {
            res.device_type = Some(DeviceType::Television);
        }
        res
    });

    // always set device type to tv for hbtvs
    let res = res.or_else(|| {
        Some(Device {
            device_type: Some(DeviceType::Television),
            ..Default::default()
        })
    });

    Ok(res)
}

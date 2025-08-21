use anyhow::Result;
use serde::Deserialize;
use fancy_regex::Regex;

use crate::parsers::utils::{lazy_user_agent_match, LazyRegex};
use once_cell::sync::Lazy;

static ENGINE_LIST: Lazy<BrowserEngineList> = Lazy::new(|| {
    let contents = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/regexes/client/browser_engine.yml"
    ));
    BrowserEngineList::from_file(contents).expect("loading browser_engine.yml")
});

static AVAILABLE_ENGINES: Lazy<Vec<String>> = Lazy::new(|| {
    // hard coded list taken from matamoto device detector
    let engines = [
        "WebKit",
        "Blink",
        "Trident",
        "Text-based",
        "Dillo",
        "iCab",
        "Elektra",
        "Presto",
        "Clecko",
        "Gecko",
        "KHTML",
        "NetFront",
        "Edge",
        "NetSurf",
        "Servo",
        "Goanna",
        "EkiohFlow",
        "Arachne",
        "LibWeb",
    ];
    engines.into_iter().map(|x| x.to_owned()).collect()
});

pub fn lookup(name: &str) -> Result<Option<String>> {
    // println!("browser engine lookup {}", name);
    let res = match ENGINE_LIST.lookup(name)? {
        None => AVAILABLE_ENGINES
            .iter()
            .find(|engine| engine.to_lowercase() == name.to_lowercase())
            .map(|x| x.to_owned()),
        res => res,
    };
    Ok(res)
}

struct BrowserEngineList {
    list: Vec<BrowserEngine>,
}

#[derive(Debug)]
struct BrowserEngine {
    name: String,
    regex: LazyRegex,
}

impl BrowserEngineList {
    fn lookup(&self, ua: &str) -> Result<Option<String>> {
        for engine in &self.list {
            // println!("engine {:?}", engine);
            if engine.regex.is_match(ua)? {
                // println!("engine match {:?}", engine);
                return Ok(Some(engine.name.clone()));
            }
        }

        Ok(None)
    }
    fn from_file(contents: &str) -> Result<Self> {
        #[derive(Debug, Deserialize)]
        #[serde(transparent)]
        struct YamlBrowserEngineList {
            list: Vec<YamlBrowserEngine>,
        }

        #[allow(clippy::from_over_into)]
        impl Into<BrowserEngineList> for YamlBrowserEngineList {
            fn into(self) -> BrowserEngineList {
                let list = self.list.into_iter().map(|e| e.into()).collect();
                BrowserEngineList { list }
            }
        }

        #[derive(Debug, Deserialize)]
        struct YamlBrowserEngine {
            name: String,
            regex: String,
        }

        #[allow(clippy::from_over_into)]
        impl Into<BrowserEngine> for YamlBrowserEngine {
            fn into(self) -> BrowserEngine {
                let regex = lazy_user_agent_match(&self.regex);

                BrowserEngine {
                    name: self.name,
                    regex,
                }
            }
        }

        let res: YamlBrowserEngineList = serde_yaml::from_str(contents)?;
        Ok(res.into())
    }
}

/// Detect engine version from user agent - equivalent to PHP's Engine\Version class
pub fn detect_engine_version(ua: &str, engine: &str) -> Result<Option<String>> {
    if engine.is_empty() {
        return Ok(None);
    }

    // Gecko and Clecko engines use special pattern
    if engine == "Gecko" || engine == "Clecko" {
        let pattern = r"[ ](?:rv[: ]([0-9.]+)).*(?:g|cl)ecko/[0-9]{8,10}";
        let regex = Regex::new(pattern)?;
        if let Some(captures) = regex.captures(ua)? {
            if let Some(version_match) = captures.get(1) {
                return Ok(Some(version_match.as_str().to_owned()));
            }
        }
        return Ok(None);
    }

    // Map engine names to regex tokens - from PHP Engine\Version.php
    let engine_token = match engine {
        "Blink" => "Chr[o0]me|Chromium|Cronet",
        "Arachne" => "Arachne\\/5\\.",
        "LibWeb" => "LibWeb\\+LibJs",
        _ => engine,
    };

    // Build the regex pattern - equivalent to PHP line 74
    // PHP: "~(?:{$engineToken})\s*[/_]?\s*((?(?=\d+\.\d)\d+[.\d]*|\d{1,7}(?=(?:\D|$))))~i"
    // The conditional regex (?(?=\d+\.\d)\d+[.\d]*|\d{1,7}(?=(?:\D|$))) is complex, let's simplify
    let pattern = format!(r"(?i)(?:{})\s*[/_]?\s*(\d+(?:\.\d+)*)", engine_token);
    let regex = Regex::new(&pattern)?;
    
    if let Some(captures) = regex.captures(ua)? {
        if let Some(version_match) = captures.get(1) {
            return Ok(Some(version_match.as_str().to_owned()));
        }
    }

    Ok(None)
}

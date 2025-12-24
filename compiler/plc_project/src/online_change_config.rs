use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct OnlineChangeConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "got-layout-file")]
    pub got_layout_file: Option<String>,
}

impl OnlineChangeConfig {
    pub fn get_enabled(&self) -> bool {
        self.enabled
    }
    pub fn get_got_layout_file(&self) -> Option<&String> {
        self.got_layout_file.as_ref()
    }
}

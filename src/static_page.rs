use crate::page::Page;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Default, Debug)]
pub struct StaticPage {
    #[serde(default = "String::new")]
    pub id: String,
    #[serde(default)]
    pub order: u8,
    pub name: String,
    #[serde(default = "String::new")]
    pub image: String,
    #[serde(default = "String::new")]
    pub image_alt: String,
    #[serde(default = "String::new")]
    pub markdown: String,
}

impl Page for StaticPage {
    fn from_path(path: String) -> Self {
        let meta_string = fs::read_to_string(path.clone() + "/meta.ron").unwrap();
        let mut page: Self = ron::from_str(meta_string.as_str()).unwrap();
        page.markdown = fs::read_to_string(path + "/content.md").unwrap();
        page
    }

    fn to_html(&self, source_dir: &String, pages: &Vec<StaticPage>, url_base: &String, site_name: &String) -> String {
        String::new()
    }
}
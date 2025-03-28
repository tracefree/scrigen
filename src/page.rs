use crate::StaticPage;

pub trait Page {
    fn from_path(path: String) -> Self;
    fn to_html(&self, source_dir: &String, pages: &Vec<StaticPage>, url_base: &String, site_name: &String) -> String;
}
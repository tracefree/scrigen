use crate::page::Page;
use convert_case::Casing;
use inkjet::{formatter, Highlighter};
use regex::{Captures, Regex};
use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Default, Debug)]
pub struct StaticPage {
    #[serde(default = "String::new")]
    pub id: String,
    #[serde(default)]
    pub order: u8,
    pub name: String,
    pub title: String,
    pub summary: String,
    pub author_fediverse: String,
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

    fn to_html(
        &self,
        source_dir: &String,
        pages: &Vec<StaticPage>,
        url_base: &String,
        site_name: &String,
    ) -> String {
        let mut html = format!(
            "<!DOCTYPE html>
        <html>
        <head>
	<meta http-equiv=\"Content-Type\" content=\"text/html; charset=utf-8\">
	<meta property=\"og:site_name\" content=\"{site_name}\">
	<meta property=\"og:type\" content=\"website\" />
	<meta property=\"og:url\" content=\"{url_base}/blog/{}\">
	<meta property=\"og:title\" content=\"{}\" />
	<meta property=\"og:description\" content=\"{}\">
	<meta property=\"og:image\" content=\"{url_base}/blog/{}/{}\"/>
    <meta name='fediverse:creator' content='{}'/>
	<title>{}</title>",
            self.id,
            self.title,
            self.summary,
            self.id,
            self.image,
            self.author_fediverse,
            self.title
        );
        let mut highlighter = Highlighter::new();

        let header =
            fs::read_to_string(format!("{source_dir}/fragments/page_header.html")).unwrap();

        let mut page_links = String::from("<a href=\"../index.html\">Blog</a>");
        for page in pages {
            page_links +=
                format!("<a href=\"../{}/index.html\">{}</a>", page.id, page.name).as_str();
        }

        let header = header.replace("___STATIC_PAGES___", &page_links);
        html += header.as_str();

        if !self.image.is_empty() {
            html += format!(
                "<div class='post-header-image'>
                <img alt='{}' src='{}' class='post-image'><br />
            </div>
            ___SIDEBAR___
            ",
                self.image_alt, self.image
            )
            .as_str();
        }

        html += "<div class='post-text'>";
        html += format!("<h1>{}</h1>", self.title).as_str();

        let mut in_code_block = false;
        let mut current_code_block = String::new();
        let mut format_gdscript = false;
        let mut in_image = false;

        let mut sections: Vec<(String, String)> = vec![];

        for line in self.markdown.lines() {
            if line.starts_with("```") {
                in_code_block = !in_code_block;
                if in_code_block {
                    format_gdscript = line.starts_with("```GDScript");
                } else {
                    if format_gdscript {
                        let result = highlighter
                            .highlight_to_string(
                                inkjet::Language::Gdscript,
                                &formatter::Html,
                                current_code_block.as_str(),
                            )
                            .unwrap();
                        // Doing crimes against Regex
                        let mut inside_tag = false;
                        let mut new_result = String::new();
                        for character in result.chars() {
                            match character {
                                '<' => inside_tag = true,
                                '>' => inside_tag = false,
                                '=' => {
                                    if !inside_tag {
                                        new_result += "&equals;";
                                        continue;
                                    }
                                }
                                _ => {}
                            };
                            new_result.push(character);
                        }
                        let regex = Regex::new(
                            "(\\(|\\)|\\[|\\]|\\:|\\+|\\-)|\\*|\\{|\\}|&gt;|&#x2f;|&equals;",
                        )
                        .unwrap();
                        let result =
                            regex.replace_all(new_result.as_str(), |captures: &Captures| {
                                format!("<span class='symbol'>{}</span>", &captures[0])
                            });
                        html += format!("<pre>{}</pre>", result).as_str();
                    } else {
                        html += format!("<pre>{}</pre>", current_code_block.as_str()).as_str();
                    }
                    current_code_block.clear();
                }
                continue;
            }
            if in_code_block {
                current_code_block += format!("{line}\n").as_str();
                continue;
            }
            if line.starts_with("!insert ") {
                let markdown_part;
                if line.starts_with("!insert bg ") {
                    html += "</div><div class='post-insert with-background'><div class='insert-content'><div class='insert-content-inner'>";

                    markdown_part = line.replace("!insert bg ", "");
                } else {
                    html += "</div><div class='post-insert'><div class='insert-content'><div class='insert-content-inner'>";
                    markdown_part = line.replace("!insert ", "");
                }

                html += markdown::to_html(markdown_part.as_str())
                    .replace("<p>", "")
                    .replace("</p>", "")
                    .as_str();
                in_image = true;
                continue;
            }
            if in_image {
                in_image = false;
                if line.starts_with("!image_subtitle ") {
                    html += format!(
                        "<br><div class='insert-description'><em>{}</em>",
                        line.replace("!image_subtitle ", "")
                    )
                    .as_str();
                    html += "</div></div></div></div><div class='post-text'>\n";
                    continue;
                } else {
                    html += "</div></div></div><div class='post-text'>\n";
                }
            }
            if line.starts_with("!html ") {
                let line = line.replace("!html ", "");
                html += line.as_str();
                continue;
            }

            let parsed_line =
                markdown::to_html_with_options(line, &markdown::Options::gfm()).unwrap();
            if parsed_line.starts_with("<h2>") {
                let section_title = parsed_line[4..parsed_line.len() - 5].to_string();
                let section_id: String = section_title
                    .chars()
                    .filter(|&c| c.is_alphanumeric() || c == ' ')
                    .collect();
                let section_id = section_id
                    .replace(":", "")
                    .to_case(convert_case::Case::Snake);
                html += format!(
                    "
                <h2 id='{}'>{}<a href='#{}'><div class='section-link' alt='Section link'>
                </div></a></h2>",
                    section_id, section_title, section_id
                )
                .as_str();
                sections.push((section_title, section_id));
            } else {
                html += parsed_line.as_str();
            }
        }
        html += "<div class='post-end'>
	<a href='../index.html'><div id='home-link'></div>Home</a>
	<a href='#page-top'><div id='top-link'></div>Back to the top</a>
</div></div>";
        let footer =
            fs::read_to_string(format!("{source_dir}/fragments/post_footer.html")).unwrap();
        html += footer.as_str();

        let mut sidebar = format!(
            "
        <div id='sidebar'>
        <ol>"
        );
        for section in sections {
            sidebar += format!("<li><a href='#{}'>{}</a></li>", section.1, section.0).as_str();
        }
        sidebar += "</ol></div>";
        let html = html.replace("___SIDEBAR___", sidebar.as_str());
        html
    }
}

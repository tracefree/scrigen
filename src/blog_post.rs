use atom_syndication::{Content, FixedDateTime, Link, Person};
use chrono::{Local, NaiveDate, NaiveDateTime, NaiveTime};
use convert_case::Casing;
use inkjet::{formatter, Highlighter};
use regex::{Captures, Regex};
use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Default)]
pub struct BlogPost {
    #[serde(default = "String::new")]
    pub id: String,
    pub title: String,
    pub summary: String,
    author_name: String,
    author_email: String,
    author_uri: String,
    pub image: String,
    pub image_alt: String,
    pub published: String,
    pub updated: String,
    #[serde(default = "String::new")]
    pub markdown: String,
}

impl BlogPost {
    pub fn from_path(path: String) -> Self {
        let meta_string = fs::read_to_string(path.clone() + "/meta.ron").unwrap();
        let mut post: BlogPost = ron::from_str(meta_string.as_str()).unwrap();
        post.markdown = fs::read_to_string(path + "/content.md").unwrap();
        post
    }

    pub fn published(&self) -> FixedDateTime {
        FixedDateTime::from_naive_utc_and_offset(
            NaiveDateTime::new(
                NaiveDate::parse_from_str(&self.published, "%Y-%m-%d").unwrap(),
                NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            ),
            Local::now().offset().clone(),
        )
    }

    pub fn updated(&self) -> FixedDateTime {
        FixedDateTime::from_naive_utc_and_offset(
            NaiveDateTime::new(
                NaiveDate::parse_from_str(&self.updated, "%Y-%m-%d").unwrap(),
                NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            ),
            Local::now().offset().clone(),
        )
    }

    pub fn to_html(&self, source_dir: &String, url_base: &String, site_name: &String) -> String {
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
	<title>{}</title>",
            self.id, self.title, self.summary, self.id, self.image, self.title
        );
        let mut highlighter = Highlighter::new();

        let header =
            fs::read_to_string(format!("{source_dir}/fragments/post_header.html")).unwrap();
        html += header.as_str();

        html += format!(
            "<div class='post-header-image'>
                <img alt='{}' src='{}' class='post-image'><br />
            </div>
            ___SIDEBAR___
            ",
            self.image_alt, self.image
        )
        .as_str();

        html += "<div class='post-text'>";
        html += format!("<h1>{}</h1>", self.title).as_str();

        let mut in_code_block = false;
        let mut current_code_block = String::new();

        let mut in_image = false;

        let mut sections: Vec<(String, String)> = vec![];

        for line in self.markdown.lines() {
            if line.starts_with("```") {
                in_code_block = !in_code_block;
                if !in_code_block {

                    if line.starts_with("```GDScript") {
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
                        let result = regex.replace_all(new_result.as_str(), |captures: &Captures| {
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
                current_code_block += line;
                current_code_block += "\n";
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
                    html +=
                        format!("<br><div class='insert-description'><em>{}</em>", line.replace("!image_subtitle ", "")).as_str();
                    html += "</div></div></div></div><div class='post-text'>\n";
                    continue;
                } else {
                    html += "</div></div></div><div class='post-text'>\n";
                }
            }
            let parsed_line = markdown::to_html_with_options(line, &markdown::Options::gfm()).unwrap();
            if parsed_line.starts_with("<h2>") {
                let section_title = parsed_line[4..parsed_line.len()-5].to_string();
                let section_id: String = section_title.chars().filter(|&c| c.is_alphanumeric() || c == ' ').collect();
                let section_id = section_id.replace(":", "").to_case(convert_case::Case::Snake);
                html += format!("
                <h2 id='{}'>{}<a href='#{}'><div class='section-link' alt='Section link'>
                </div></a></h2>", section_id, section_title, section_id).as_str();
                sections.push((section_title, section_id));
            } else {
                html += parsed_line.as_str();
            }
        }
        html += "<div class='post-end'>
	<a href='../../index.html'><div id='home-link'></div>Home</a>
	<a href='#page-top'><div id='top-link'></div>Back to the top</a>
</div></div>";
        let footer =
            fs::read_to_string(format!("{source_dir}/fragments/post_footer.html")).unwrap();
        html += footer.as_str();

        let mut sidebar = format!("
        <div id='sidebar'>
        <div class='sidebar-info'>
        Published <span class='sidebar-date'>{}</span><br>
        Updated <span class='sidebar-date'>{}</span><br>
        </div>
        <hr>
        <ol>", self.published().format("%B %e, %Y"), self.updated().format("%B %e, %Y"));
        for section in sections {
            sidebar += format!("<li><a href='#{}'>{}</a></li>", section.1, section.0).as_str();
        }
        sidebar += "</ol></div>";
        let html = html.replace("___SIDEBAR___", sidebar.as_str());
        html
    }

    pub fn to_entry_fragment(&self) -> String {
        format!(
            "<div class='post-entry'>
                <img class='entry-image' src='blog/{}/{}' alt='{}'/>
                <div class='entry-text'>
                    <a href='blog/{}/index.html' class='entry-link'></a>
         			<h2 class='entry-title'>{}</h2>
         			<span class='entry-date'>{}</span>
         			<p class='entry-summary'>{}</p>
                </div>
			</div>",
            self.id,
            self.image,
            self.image_alt,
            self.id,
            self.title,
            self.published().format("%B %e, %Y"),
            self.summary
        )
    }

    pub fn get_atom_entry(
        &self,
        source_dir: &String,
        url_base: &String,
        site_name: &String,
    ) -> atom_syndication::Entry {
        let mut entry = atom_syndication::Entry::default();
        entry.set_title(self.title.clone());
        entry.set_authors(vec![Person {
            name: self.author_name.clone(),
            email: Some(self.author_email.clone()),
            uri: Some(self.author_uri.clone()),
        }]);
        let post_url = format!("{url_base}/blog/{}", self.id);
        entry.set_id(&post_url);
        entry.set_links(vec![Link {
            href: post_url.clone(),
            rel: "alternate".into(),
            mime_type: Some("text/html".into()),
            ..Default::default()
        }]);
        entry.set_summary(Some(atom_syndication::Text::plain(self.summary.clone())));
        entry.set_published(self.published());
        entry.set_updated(self.updated());
        let content = Content {
            base: Some(post_url.clone()),
            lang: Some("en".into()),
            value: Some(self.to_html(source_dir, url_base, site_name)),
            src: Some(post_url.clone()),
            content_type: Some("html".into()),
        };
        entry.set_content(content);
        entry
    }
}

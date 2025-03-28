use atom_syndication::{Entry, Feed, Generator, Link, Person, Text};
use serde::Deserialize;
use static_page::StaticPage;
use std::{
    cmp::Ordering,
    env,
    fs::{self, File},
    io::Write,
};

mod page;
mod blog_post;
mod static_page;

use page::Page;
use blog_post::*;


// TODO:
// - Publish and update dates in post
// - Categories
// - Proper error handling
// - Proper OS directory handling

#[derive(Deserialize)]
struct FeedInfo {
    title: String,
    id: String,
    author_name: String,
    author_email: String,
    author_uri: String,
    link_site: String,
    link_feed: String,
}

#[derive(PartialEq)]
enum Operation {
    WriteAll,
    WriteLanding,
    WritePosts,
    WritePages,
    WriteFeed,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let source_directory;
    let target_directory;
    let operation;
    match args[1].as_str() {
        "--landing" => {
            operation = Operation::WriteLanding;
            source_directory = &args[2];
            target_directory = &args[3];
        }
        "--post" => {
            operation = Operation::WritePosts;
            source_directory = &args[2];
            target_directory = &args[3];
        }
        "--pages" => {
            operation = Operation::WritePages;
            source_directory = &args[2];
            target_directory = &args[3];
        }
        "--feed" => {
            operation = Operation::WriteFeed;
            source_directory = &args[2];
            target_directory = &args[3];
        }
        _ => {
            operation = Operation::WriteAll;
            source_directory = &args[1];
            target_directory = &args[2];
        }
    };

    let static_pages: Vec<StaticPage> = parse_pages(&source_directory);
    let blog_posts = parse_posts(source_directory);
    if operation == Operation::WritePosts || operation == Operation::WriteAll {
        write_posts(&blog_posts, &static_pages, &source_directory, &target_directory)
    }
    if operation == Operation::WriteLanding || operation == Operation::WriteAll {
        write_landing_page(&blog_posts, &static_pages, &source_directory, &target_directory);
    }
    if operation == Operation::WriteFeed || operation == Operation::WriteAll {
        write_feed(&blog_posts, &static_pages, &source_directory, &target_directory);
    }
    if operation == Operation::WritePages || operation == Operation::WriteAll {
        write_static_pages(&static_pages, source_directory, target_directory);
    }
}

fn parse_pages(path: &str) -> Vec<StaticPage> {
    let mut pages: Vec<StaticPage> = Vec::new();
    let paths = fs::read_dir(format!("{path}/pages/")).unwrap();
    for path in paths {
        let path = path.unwrap();
        let mut page = StaticPage::from_path(path.path().to_str().unwrap().to_string());
        let path_name = path.file_name().to_str().unwrap().to_string();
        page.id = path_name[2..].to_string();
        page.order = u8::from_str_radix(&path_name[0..1], 10).unwrap();
        if page.id.starts_with(".") {
            continue;
        }
        pages.push(page);
    }
    pages
        .sort_by(|entry1, entry2| -> Ordering { entry1.order.cmp(&entry2.order) });
    pages
}

fn parse_posts(path: &str) -> Vec<BlogPost> {
    let mut blog_posts: Vec<BlogPost> = Vec::new();
    let paths = fs::read_dir(format!("{path}/entries/")).unwrap();
    for path in paths {
        let path = path.unwrap();
        let mut post = BlogPost::from_path(path.path().to_str().unwrap().to_string());
        post.id = path.file_name().to_str().unwrap().to_string();
        if post.id.starts_with(".") {
            continue;
        }
        blog_posts.push(post);
    }
    blog_posts
        .sort_by(|entry1, entry2| -> Ordering { entry2.published().cmp(&entry1.published()) });
    blog_posts
}

fn write_posts(blog_posts: &Vec<BlogPost>, pages: &Vec<StaticPage>, source_directory: &String, target_directory: &String) {
    // TODO: Avoid reading this file twice
    let feed_string = fs::read_to_string(format!("{source_directory}/feed.ron")).unwrap();
    let feed_info: FeedInfo = ron::from_str(feed_string.as_str()).unwrap();
    for post in blog_posts {
        let html = post.to_html(source_directory, pages, &feed_info.link_site, &feed_info.title);
        let directory = format!("{target_directory}/blog/{}", post.id);
        let _result = fs::create_dir(directory.clone());
        let _result = fs::write(directory + "/index.html", html);

        for file in fs::read_dir(format!("{source_directory}/entries/{}", post.id)).unwrap() {
            if let Ok(file) = file {
                match file.file_name().to_str() {
                    Some("content.md") => continue,
                    Some("meta.ron") => continue,
                    Some(file_name) => {
                        let source_path =
                            format!("{source_directory}/entries/{}/{}", post.id, file_name);
                        let target_path =
                            format!("{target_directory}/blog/{}/{}", post.id, file_name);
                        let _result = fs::copy(source_path, target_path);
                    }
                    None => continue,
                };
            }
        }
    }
}

fn write_static_pages(
    pages: &Vec<StaticPage>,
    source_directory: &String,
    target_directory: &String
) {
    let feed_string = fs::read_to_string(format!("{source_directory}/feed.ron")).unwrap();
    let feed_info: FeedInfo = ron::from_str(feed_string.as_str()).unwrap();
    for page in pages {
        let html = page.to_html(source_directory, pages, &feed_info.link_site, &feed_info.title);
        let directory = format!("{target_directory}/{}", page.id);
        let _result = fs::create_dir(directory.clone());
        let _result = fs::write(directory + "/index.html", html);
        for file in fs::read_dir(format!("{source_directory}/pages/{}_{}", page.order, page.id)).unwrap() {
            if let Ok(file) = file {
                match file.file_name().to_str() {
                    Some("content.md") => continue,
                    Some("meta.ron") => continue,
                    Some(file_name) => {
                        let source_path =
                            format!("{source_directory}/pages/{}_{}/{}", page.order, page.id, file_name);
                        let target_path =
                            format!("{target_directory}/{}/{}", page.id, file_name);
                        println!("Copying {source_path} to {target_path}");
                        let _result = fs::copy(source_path, target_path);
                    }
                    None => continue,
                };
            }
        }
    }
}

fn write_landing_page(
    blog_posts: &Vec<BlogPost>,
    pages: &Vec<StaticPage>,
    source_directory: &String,
    target_directory: &String,
) {
    let landing_header =
        fs::read_to_string(format!("{source_directory}/fragments/landing_header.html")).unwrap();
    let mut page_links = String::new();
    for page in pages {
        page_links += format!("<a href=\"../../{}/index.html\">{}</a>", page.id, page.name).as_str();
    }
    let landing_header = landing_header.replace("___STATIC_PAGES___", &page_links);
    
    let landing_footer =
        fs::read_to_string(format!("{source_directory}/fragments/landing_footer.html")).unwrap();

    let mut html = String::from(landing_header);
    for post in blog_posts {
        html += post.to_entry_fragment().as_str();
    }
    html += landing_footer.as_str();
    let _result = fs::write(format!("{target_directory}/index.html"), html);
}

fn write_feed(blog_posts: &Vec<BlogPost>, pages: &Vec<StaticPage>, source_directory: &String, target_directory: &String) {
    let feed_string = fs::read_to_string(format!("{source_directory}/feed.ron")).unwrap();
    let feed_info: FeedInfo = ron::from_str(feed_string.as_str()).unwrap();

    let mut feed = Feed {
        title: Text::from(feed_info.title.clone()),
        id: feed_info.id,
        authors: vec![Person {
            name: feed_info.author_name,
            email: Some(feed_info.author_email),
            uri: Some(feed_info.author_uri),
        }],
        generator: Some(Generator {
            value: "atom_syndication".into(),
            uri: Some("https://github.com/rust-syndication/atom".into()),
            version: Some("0.12.4".into()),
        }),
        links: vec![
            Link {
                href: feed_info.link_site.clone(),
                rel: "alternate".into(),
                mime_type: Some("text/html".into()),
                ..Default::default()
            },
            Link {
                href: feed_info.link_feed.clone(),
                rel: "self".into(),
                mime_type: Some("application/atom+xml".into()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let mut entries: Vec<Entry> = blog_posts
        .iter()
        .map(|post| -> Entry {
            post.get_atom_entry(
                source_directory,
                &pages,
                &feed_info.link_site.clone(),
                &feed_info.title.clone(),
            )
        })
        .collect();

    feed.set_entries(entries.clone());
    entries.sort_by(|entry1, entry2| -> Ordering { entry2.updated().cmp(&entry1.updated()) });
    feed.set_updated(*entries[0].updated());

    let mut feed_file = File::create(format!("{target_directory}/blog/atom.xml")).unwrap();
    let _result = feed_file.write_all(feed.to_string().as_bytes());
}

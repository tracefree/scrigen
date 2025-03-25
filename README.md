# scrigen

This is a small, bespoke static site generator for my gamedev blog, [Invariance](https://rie.codeberg.page), written in Rust. It takes blog posts in the form of markdown files as input and generates the corresponding HTML file (including syntax highlighting support for code blocks), the corresponding entry on the landing page, and an Atom feed. Currently it's tightly coupled to my HTML and CSS file structure, doesn't handle errors gracefully, and is undocumented, so there's no reason to use this over any other static site generator. Just releasing the source in case anyone is curious about how my site works.

![Screenshot of my blog showing a sidebar next to the article content with links to each section and a code block with proper syntax highlighting. The article talks about the script template feature of Godot.](https://github.com/user-attachments/assets/daa78aed-68be-4657-89ea-8365b8d93834)

## Crates used
- [Serde](https://crates.io/crates/serde) and [ron](https://crates.io/crates/ron), for reading metadata
- [markdown](https://crates.io/crates/markdown), for converting Markdown to HTML
- [inkjet](https://crates.io/crates/inkjet), for syntax highlighting
- [regex](https://crates.io/crates/regex), for working around a limitation of the above, where GDScript symbols `+ - < > / * : [ ] { } ( )` aren't differentiated
- [atom_syndication](https://crates.io/crates/atom_syndication), for generating the Atom feed
- [chrono](https://crates.io/crates/chrono), for sorting posts by their publishing date
- [convert_case](https://docs.rs/convert_case/latest/convert_case/), for, uh, converting case

## License
Your choice of either Apache 2.0 or the MIT license.

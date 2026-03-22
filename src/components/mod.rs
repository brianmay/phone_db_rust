pub mod buttons;
pub mod contacts;
pub mod defaults;
pub mod navbar;
pub mod phone_calls;
pub mod users;

mod times;

use dioxus::prelude::*;

#[component]
pub fn Markdown(content: String) -> Element {
    use pulldown_cmark::{Options, Parser, html};

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(&content, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    rsx! {
        div { class: "prose", dangerous_inner_html: "{html_output}" }
    }
}

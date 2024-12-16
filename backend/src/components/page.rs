#![allow(non_snake_case)]

use crate::version;
use dioxus::prelude::*;

pub fn NavBar() -> Element {
    rsx! {
            nav {
                class: "navbar navbar-expand-sm navbar-dark bg-dark navbar-fixed-top",
                role: "navigation",
                div {
                    class: "container-fluid",
                    a { class: "navbar-brand", href: "/", "Phone DB" }
                    button {
                        class: "navbar-toggler",
                        type: "button",
                        "data-bs-toggle": "collapse",
                        "data-bs-target": "#navbarNav",
                        "aria-controls": "navbarNav",
                        "aria-expanded": "false",
                        "aria-label": "Toggle navigation",
                        span { class: "navbar-toggler-icon"}
                    }
                    div { class: "collapse navbar-collapse", id: "navbarNav",
                       div { class: "navbar-nav",
                            li { class: "nav-item",
                                a { class: "nav-link", href: "/phone_calls", { "Phone Calls" }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn Footer() -> Element {
    let build_date: &str = version::BUILD_DATE.unwrap_or("unknown");
    let build_version: &str = version::VCS_REF.unwrap_or("unknown");

    rsx! {
        footer {
            div {
                div { { "Build Date: " } { build_date } }
                div { { "Version: " }  { build_version } }
            }
            div {
                "Phone DB"
            }
        }
    }
}

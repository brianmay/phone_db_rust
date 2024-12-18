#![allow(non_snake_case)]

use crate::version;
use dioxus::prelude::*;
use dioxus_router::prelude::navigator;

use super::app::Route;

#[component]
pub fn NavBar() -> Element {
    rsx! {
        nav {
            class: "navbar navbar-expand-sm navbar-dark bg-dark navbar-fixed-top",
            role: "navigation",
            div {
                class: "container-fluid",
                a {
                    class: "navbar-brand",
                    // href: "/",
                    onclick: |_| {
                        let navigator = navigator();
                        navigator.push(Route::Root {});
                    },
                    "Phone DB"
                }
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
                div {
                    class: "collapse navbar-collapse", id: "navbarNav",
                    div {
                        class: "navbar-nav",
                        li {
                            class: "nav-item",
                            a {
                                class: "nav-link",
                                // href: "/phone_calls",
                                onclick: |_| {
                                    let navigator = navigator();
                                    navigator.push(Route::PhoneCallListView {});
                                },
                                { "Phone Calls" }
                            }
                        }
                    }
                    div {
                        class: "navbar-nav",
                        li {
                            class: "nav-item",
                            a {
                                class: "nav-link",
                                // href: "/contacts",
                                onclick: |_| {
                                    let navigator = navigator();
                                    navigator.push(Route::ContactListView {});
                                },
                                { "Contacts" }
                            }
                        }
                    }
                    div {
                        class: "navbar-nav",
                        li {
                            class: "nav-item",
                            a {
                                class: "nav-link",
                                // href: "/defaults",
                                onclick: |_| {
                                    let navigator = navigator();
                                    navigator.push(Route::DefaultListView {});
                                },
                                { "Defaults" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
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

#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
    let segments = segments.join("/");
    rsx! {
        div {
            NavBar {}

            main {
                role: "main",
                class: "container",
                h1 { "404 Not Found" }
                p { "The page you are looking for does not exist." }
                p { "Segments: {segments}" }
                p { "Please ask a friendly penguin for help." }
            }

            Footer {}
        }
    }
}

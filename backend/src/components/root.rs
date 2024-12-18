#![allow(non_snake_case)]

use super::page::{Footer, NavBar};
use crate::Props;
use dioxus::prelude::*;

pub fn App() -> Element {
    let mut num = use_signal(|| 0);

    let props = use_context::<Props>();
    let user = format!("{:?}", props.user);

    rsx! {
        NavBar {}
        div {
            "hello {user}! {num}"
            button { onclick: move |_| num += 1, "Increment" }
        }
        Footer {}
    }
}

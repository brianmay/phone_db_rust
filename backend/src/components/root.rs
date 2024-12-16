#![allow(non_snake_case)]

use super::page::{Footer, NavBar};
use crate::Props;
use dioxus::prelude::*;

pub fn App(props: Props) -> Element {
    let mut num = use_signal(|| 0);

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

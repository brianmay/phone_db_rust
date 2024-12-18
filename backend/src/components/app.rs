#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_router::prelude::*;

use super::contacts::ContactDetailView;
use super::contacts::ContactListView;
use super::defaults::DefaultDetailView;
use super::defaults::DefaultListView;
use super::page::NotFound;
use super::phone_calls::PhoneCallListView;
use super::root::App as Root;
use crate::Props;

#[rustfmt::skip]
#[derive(Clone, Debug, PartialEq, Routable)]
pub enum Route {
    #[nest("/phone_calls")]
        #[route("/")]
        PhoneCallListView {},
    #[end_nest]
    #[nest("/contacts")]
        #[route("/")]
        ContactListView {},
        #[route("/:contact_id")]
        ContactDetailView { contact_id: i64 },
    #[end_nest]
    #[nest("/defaults")]
        #[route("/")]
        DefaultListView {},
        #[route("/:default_id")]
        DefaultDetailView { default_id: i64 },
    #[end_nest]
    #[route("/")]
    Root {},
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

pub fn App(props: Props) -> Element {
    use_context_provider(|| props.state.db.clone());
    use_context_provider(|| props.state.incoming_call.clone());
    use_context_provider(|| props);

    rsx! {
        Router::<Route> { }
    }
}

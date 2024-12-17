#![allow(non_snake_case)]

use super::page::{Footer, NavBar};
use crate::{
    components::contacts::EditContactDialog, database, datetime::datetime_to_string, Props,
};
use common::{Page, PhoneCallDetails, PhoneCallKey};
use dioxus::prelude::*;
use sqlx::PgPool;

#[component]
fn PhoneCallComponent(
    show_actions: bool,
    phone_call: ReadOnlySignal<PhoneCallDetails>,
    on_edit_contact: Callback<i64>,
) -> Element {
    let phone_call = phone_call.read();
    let contact_id = phone_call.contact_id;

    rsx! {
        tr {
            td { { datetime_to_string(phone_call.inserted_at) } }
            td { { phone_call.phone_number.clone() } }
            td { { phone_call.contact_name.clone() } }
            td { { phone_call.destination_number.clone() } }
            td { { phone_call.action.to_string()} {"->"} { phone_call.contact_action.to_string() } }
            td { { phone_call.number_calls.unwrap_or(-1).to_string() } }
            if show_actions {
                td { a { href: format!("/contacts/{contact_id}"), "Contact" }
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| {
                            on_edit_contact.call(contact_id);
                        },
                        "Edit"
                    }
                }
            }
        }
    }
}

#[component]
pub fn PhoneCallList(
    contact_id: Option<i64>,
    phone_calls: Signal<Option<Result<Page<PhoneCallDetails, PhoneCallKey>, sqlx::Error>>>,
    request: Signal<common::PageRequest<PhoneCallKey>>,
    on_edit_contact: Callback<i64>,
) -> Element {
    // let mut request = use_signal(|| common::PageRequest::<PhoneCallKey> {
    //     after_key: None,
    //     search: None,
    // });

    // let mut phone_calls =
    //     use_signal(|| Option::<Result<Page<PhoneCallDetails, PhoneCallKey>, sqlx::Error>>::None);

    // let phone_calls = use_resource(move || {
    //     let db = props.state.db.clone();
    //     let request = request.read_unchecked();
    //     async move { database::phone_calls::get_phone_calls(&db, &request).await }
    // });

    let _resource = use_resource(move || async move {
        let db = use_context::<PgPool>();
        let request = request.read_unchecked();
        let result = database::phone_calls::get_phone_calls(&db, &request, None).await;
        let mut writable = phone_calls.write();

        match (writable.as_mut(), result) {
            (Some(Ok(writable)), Ok(result)) => {
                writable.items.extend(result.items);
                writable.next_key = result.next_key;
            }
            (Some(writable), result) => {
                *writable = result;
            }
            (None, result) => {
                *writable = Some(result);
            }
        }
    });

    let mut next_key = Option::<PhoneCallKey>::None;

    rsx! {

        main {
            role: "main",
            class: "container",

            h1 {
                "Listing Phone Calls"
            }

            form {
                input { type: "text", placeholder: "Search...", oninput: move |e| {
                    let value = e.value();
                    let value = if value.is_empty() { None } else { Some(value) };
                    phone_calls.set(None);
                    request.set(common::PageRequest {
                        after_key: None,
                        search: value,
                    });
                } }
            }

            match &*phone_calls.read() {
                Some(Ok(page)) => {
                    let list = &page.items;
                    next_key = page.next_key.clone();

                    rsx! {
                        table {
                            class: "table table-hover",
                            thead {
                                class: "thead-dark",
                                tr {
                                    th { "Time" }
                                    th { "Phone Number" }
                                    th { "Name" }
                                    th { "Destination" }
                                    th { "Action" }
                                    th { "Calls" }
                                    if contact_id.is_none() {
                                        th {}
                                    }
                                }
                            }
                            tbody {
                                for phone_call in list {
                                    PhoneCallComponent {
                                        key: phone_call.id, show_actions: contact_id.is_none(),
                                        phone_call: phone_call.clone(),
                                        on_edit_contact: move |contact_id| {
                                            on_edit_contact.call(contact_id);
                                        }
                                     }
                                }
                            }
                        }


                    }

                }
                Some(Err(err)) => {
                    rsx! { "An error occurred while fetching phone calls: {err}" }
                }
                None => {
                    rsx! { "Loading items" }
                }
            }

            if let Some(next_key) = next_key {
                button {
                    class: "btn btn-primary",
                    onclick: move |_| {
                        let next_key = next_key.clone();
                        async move {
                            let mut writable = request.write();
                            writable.after_key = Some(next_key.clone());
                        }
                    },
                    "Load more"
                }
            }
        }
    }
}

pub fn PhoneCallListView(props: Props) -> Element {
    let mut phone_calls = use_signal(|| None);
    let mut request = use_signal(|| common::PageRequest::<PhoneCallKey> {
        after_key: None,
        search: None,
    });

    let mut edit_contact = use_signal(|| None);

    use_context_provider(|| props.state.db.clone());

    rsx! {
        NavBar {}
        PhoneCallList {
            contact_id: None,
            phone_calls: phone_calls,
            request: request,
            on_edit_contact: move |contact_id| {
                edit_contact.set(Some(contact_id));
            }
        }

        if let Some(contact_id) = *edit_contact.read() {
            EditContactDialog{
                contact_id: contact_id,
                on_cancel: move || {
                    edit_contact.set(None);
                },
                on_save: move || {
                    edit_contact.set(None);
                    phone_calls.set(None);
                    let mut writable = request.write();
                    writable.after_key = None;
                }
            }
        }

        Footer {}
    }
}

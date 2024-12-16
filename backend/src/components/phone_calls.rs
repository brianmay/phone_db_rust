#![allow(non_snake_case)]

use super::page::{Footer, NavBar};
use crate::{database, datetime::datetime_to_string, Props};
use common::{Page, PhoneCallDetails, PhoneCallKey};
use dioxus::prelude::*;

#[component]
fn PhoneCall(phone_call: ReadOnlySignal<PhoneCallDetails>) -> Element {
    let phone_call = phone_call.read();

    rsx! {
        tr {
            td { { datetime_to_string(phone_call.inserted_at) } }
            td { { phone_call.contact_phone_number.clone() } }
            td { { phone_call.contact_name.clone() } }
            td { { phone_call.destination_number.clone() } }
            td { { phone_call.action.to_string()} {"->"} { phone_call.contact_action.to_string() } }
            td { { phone_call.number_calls.unwrap_or(99).to_string() } }
            td { "" }
        }
    }
}

pub fn PhoneCalls(props: Props) -> Element {
    let mut request = use_signal(|| common::PageRequest::<PhoneCallKey> {
        after_key: None,
        search: None,
    });

    let mut phone_calls =
        use_signal(|| Option::<Result<Page<PhoneCallDetails, PhoneCallKey>, sqlx::Error>>::None);

    // let phone_calls = use_resource(move || {
    //     let db = props.state.db.clone();
    //     let request = request.read_unchecked();
    //     async move { database::phone_calls::get_phone_calls(&db, &request).await }
    // });

    let db = props.state.db.clone();
    let _resource = use_resource(move || {
        let db = db.clone();
        async move {
            let request = request.read_unchecked();
            let result = database::phone_calls::get_phone_calls(&db, &request).await;
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
        }
    });

    let mut next_key = Option::<PhoneCallKey>::None;

    rsx! {

        NavBar {}


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
                                    th {}
                                }
                            }
                            tbody {
                                for phone_call in list {
                                    PhoneCall { key: phone_call.id, phone_call: phone_call.clone() }
                                }
                            }
                        }


                    }

                }
                Some(Err(err)) => {
                    rsx! { "An error occurred while fetching phone calls {err}" }
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

        Footer {}

    }
}
#![allow(non_snake_case)]

use super::page::{Footer, NavBar};
use crate::{components::phone_calls::PhoneCallList, database, Props};
use common::{ContactDetails, ContactKey, Page};
use dioxus::prelude::*;

#[component]
fn Contact(contact: ReadOnlySignal<ContactDetails>) -> Element {
    let contact = contact.read();

    rsx! {
        tr {
            td { { contact.phone_number.clone() } }
            td { { contact.name.clone() } }
            td { { contact.action.to_string()} }
            td { { contact.number_calls.unwrap_or(-1).to_string() } }
            td { a { href: format!("/contacts/{}", contact.id), "View" } }
        }
    }
}

pub fn ContactListView(props: Props) -> Element {
    use_context_provider(|| props.state.db.clone());

    let mut request = use_signal(|| common::PageRequest::<ContactKey> {
        after_key: None,
        search: None,
    });

    let mut contacts =
        use_signal(|| Option::<Result<Page<ContactDetails, ContactKey>, sqlx::Error>>::None);

    let db = props.state.db.clone();
    let _resource = use_resource(move || {
        let db = db.clone();
        async move {
            let request = request.read_unchecked();
            let result = database::contacts::get_contacts(&db, &request).await;
            let mut writable = contacts.write();

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

    let mut next_key = Option::<ContactKey>::None;

    rsx! {

        NavBar {}


        main {
            role: "main",
            class: "container",

            h1 {
                "Listing Contacts"
            }

            form {
                input { type: "text", placeholder: "Search...", oninput: move |e| {
                    let value = e.value();
                    let value = if value.is_empty() { None } else { Some(value) };
                    contacts.set(None);
                    request.set(common::PageRequest {
                        after_key: None,
                        search: value,
                    });
                } }
            }

            match &*contacts.read() {
                Some(Ok(page)) => {
                    let list = &page.items;
                    next_key = page.next_key.clone();

                    rsx! {
                        table {
                            class: "table table-hover",
                            thead {
                                class: "thead-dark",
                                tr {
                                    th { "Phone Number" }
                                    th { "Name" }
                                    th { "Action" }
                                    th { "Calls" }
                                    th {}
                                }
                            }
                            tbody {
                                for contact in list {
                                    Contact { key: contact.id, contact: contact.clone() }
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

pub fn ContactDetailView(props: Props<i64>) -> Element {
    use_context_provider(|| props.state.db.clone());

    let id = props.path;

    let contact = use_resource(move || {
        let db = props.state.db.clone();
        async move { database::contacts::get_contact(&db, id).await }
    });

    rsx! {
        NavBar {}

        main {
            role: "main",
            class: "container",


            {
                match &*contact.read() {
                    Some(Ok(contact)) => {
                        let contact_name = contact.name.clone().unwrap_or("Unknown".to_string());
                        rsx! {
                            h1 {
                                "Contact {contact_name}"
                            }
                            table {
                                class: "table table-hover",

                                tbody {
                                    tr {
                                        th { "Name" }
                                        td { { contact.name.clone() } }
                                    }
                                    tr {
                                        th { "Phone Number" }
                                        td { { contact.phone_number.clone() } }
                                    }
                                    tr {
                                        th { "Action" }
                                        td { { contact.action.to_string() } }
                                    }
                                    tr {
                                        th { "Comments" }
                                        td { { contact.comments.clone() } }
                                    }
                                }
                            }

                            PhoneCallList { contact_id: Some(id) }
                        }

                    }
                    Some(Err(err)) => {
                        rsx! { "An error occurred while fetching contact {err}" }
                    }
                    None => {
                        rsx! { "Loading contact" }
                    }
                }
            }
        }
        Footer {}
    }
}

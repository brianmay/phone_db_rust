use std::ops::Deref;

use chrono::Local;
use dioxus::prelude::*;
use dioxus_fullstack::ServerFnError;

use crate::{
    components::contacts::ContactSummary,
    functions::phone_calls::search_phone_calls,
    models::{
        contacts::Contact,
        phone_calls::{PhoneCall, PhoneCallId},
    },
    use_user,
};

#[component]
fn EntryRow(
    phone_call: PhoneCall,
    contact: Contact,
    selected: Signal<Option<PhoneCallId>>,
) -> Element {
    let id = phone_call.id;

    rsx! {
        tr {
            class: "hover:bg-gray-500 border-blue-300 mt-2 mb-2 p-2 border-2 w-full sm:w-auto sm:border-none inline-block sm:table-row",
            onclick: move |_| { selected.set(Some(id)) },
            td { class: "block sm:table-cell border-blue-300 sm:border-t-2",
                {
                    phone_call
                        .inserted_at
                        .with_timezone(&Local)
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string()
                }
            }
            td { class: "block sm:table-cell border-blue-300 sm:border-t-2",
                ContactSummary { contact: contact.clone() }
            }
            td { class: "block sm:table-cell border-blue-300 sm:border-t-2",
                span { class: "sm:hidden", "Action: " }
                {phone_call.action.clone()}
            }
            td { class: "block sm:table-cell border-blue-300 sm:border-t-2",
                if let Some(dest) = &phone_call.destination_number {
                    {dest.clone()}
                }
            }
        }
    }
}

#[component]
pub fn PhoneCallList() -> Element {
    let user = use_user().ok().flatten();

    let Some(_user) = user.as_ref() else {
        return rsx! {
            p { class: "alert alert-error", "You are not logged in." }
        };
    };

    let selected: Signal<Option<PhoneCallId>> = use_signal(|| None);

    let mut query = use_signal(|| "".to_string());

    let list: Resource<Result<Vec<(PhoneCall, Contact)>, ServerFnError>> =
        use_resource(move || async move { search_phone_calls(query()).await });

    rsx! {
        div { class: "ml-2 mr-2",
            div { class: "mb-2",
                input {
                    class: "form-control",
                    r#type: "text",
                    value: query(),
                    oninput: move |e| query.set(e.value()),
                    placeholder: "Search...",
                }
            }
        }

        match list.read().deref() {
            Some(Err(err)) => rsx! {
                div { class: "alert alert-error",
                    "Error loading phone calls: "
                    {err.to_string()}
                }
            },
            Some(Ok(list)) if list.is_empty() => rsx! {
                p { class: "alert alert-info", "No entries found." }
            },
            Some(Ok(list)) => rsx! {
                div { class: "ml-2 mr-2 sm:ml-0 sm:mr-0",
                    table { class: "block sm:table",
                        thead { class: "hidden sm:table-header-group",
                            tr {
                                th { "Time" }
                                th { "Contact ID" }
                                th { "Action" }
                                th { "Destination" }
                            }
                        }
                        tbody { class: "block sm:table-row-group",
                            for phone_call in list.iter() {
                                EntryRow {
                                    phone_call: phone_call.0.clone(),
                                    contact: phone_call.1.clone(),
                                    selected,
                                }
                            }
                        }
                    }
                }
            },
            None => {
                rsx! {
                    p { class: "alert alert-info", "Loading..." }
                }
            }
        }
    }
}

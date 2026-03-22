use std::ops::Deref;

use chrono::Local;
use dioxus::prelude::*;
use dioxus_fullstack::ServerFnError;
use tap::Pipe;

use crate::{
    Route,
    components::{
        buttons::ChangeButton,
        contacts::{ActiveDialog, ContactDialog, ContactSummary, ListDialogReference, Operation},
    },
    functions::{contacts::get_contact_by_id, phone_calls::search_phone_calls},
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

        if selected() == Some(id) {
            tr {
                td { colspan: "6", class: "block sm:table-cell",
                    div { class: "flex gap-2",
                        ChangeButton {
                            on_click: move |_| {
                                navigator()
                                    .push(Route::PhoneCallList {
                                        dialog: ListDialogReference::Update {
                                            contact_id: contact.id,
                                        },
                                    });
                            },
                            "Edit"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn PhoneCallList(dialog: ReadSignal<Option<ListDialogReference>>) -> Element {
    let user = use_user().ok().flatten();

    let Some(_user) = user.as_ref() else {
        return rsx! {
            p { class: "alert alert-error", "You are not logged in." }
        };
    };

    let selected: Signal<Option<PhoneCallId>> = use_signal(|| None);

    let mut query = use_signal(|| "".to_string());

    let dialog: Resource<Result<ActiveDialog, ServerFnError>> = use_resource(move || async move {
        let Some(dialog) = dialog() else {
            return Ok(ActiveDialog::Idle);
        };
        match dialog {
            ListDialogReference::Create => ActiveDialog::Change(Operation::Create).pipe(Ok),
            ListDialogReference::Update { contact_id } => {
                let contact = get_contact_by_id(contact_id)
                    .await?
                    .ok_or(ServerFnError::new("Cannot find contact"))?;
                ActiveDialog::Change(Operation::Update { contact }).pipe(Ok)
            }
            ListDialogReference::Delete { contact_id } => {
                let contact = get_contact_by_id(contact_id)
                    .await?
                    .ok_or(ServerFnError::new("Cannot find contact"))?;
                ActiveDialog::Delete(contact).pipe(Ok)
            }
            ListDialogReference::Idle => Ok(ActiveDialog::Idle),
        }
    });

    let mut list: Resource<Result<Vec<(PhoneCall, Contact)>, ServerFnError>> =
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

        match dialog.read().deref() {
            Some(Err(err)) => rsx! {
                div { class: "alert alert-error",
                    "Error loading dialog: "
                    {err.to_string()}
                }
            },
            Some(Ok(dialog)) => rsx! {
                ContactDialog {
                    dialog: dialog.clone(),
                    on_change: move |_contact: Contact| { list.restart() },
                    on_delete: move |_contact| list.restart(),
                    on_close: move |()| {
                        navigator()
                            .push(Route::PhoneCallList {
                                dialog: ListDialogReference::Idle,
                            });
                    },
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

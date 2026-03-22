#![allow(non_snake_case)]

use std::ops::Deref;

use chrono::{DateTime, Local, Utc};
use dioxus::prelude::*;
use dioxus_fullstack::ServerFnError;
use tap::Pipe;

use crate::{
    Route,
    components::{
        buttons::{ChangeButton, NavButton},
        contacts::{ActiveDialog, ContactDialog, ContactSummary, ListDialogReference, Operation},
    },
    functions::{contacts::get_contact_by_id, phone_calls::search_phone_calls_paginated},
    models::{
        contacts::Contact,
        phone_calls::{PhoneCall, PhoneCallId},
    },
    use_user,
};

const PAGE_SIZE: i64 = 50;

#[component]
fn EntryRow(
    phone_call: PhoneCall,
    contact: Contact,
    selected: Signal<Option<PhoneCallId>>,
    q: String,
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
                span { class: "sm:hidden", "Contact: " }
                div { class: "md:pl-0 pl-4",
                    ContactSummary { contact: contact.clone() }
                }
            }
            td { class: "block sm:table-cell border-blue-300 sm:border-t-2",
                span { class: "sm:hidden", "Action: " }
                {phone_call.action.clone()}
            }
            td { class: "block sm:table-cell border-blue-300 sm:border-t-2",
                span { class: "sm:hidden", "Source: " }
                {phone_call.source_number.clone()}
            }
            td { class: "block sm:table-cell border-blue-300 sm:border-t-2",
                span { class: "sm:hidden", "Destination: " }
                if let Some(dest) = &phone_call.destination_number {
                    {dest.clone()}
                }
            }
        }

        if selected() == Some(id) {
            tr {
                td { colspan: "6", class: "block sm:table-cell",
                    div { class: "flex gap-2",
                        NavButton {
                            on_click: move |_| {
                                navigator()
                                    .push(Route::ContactDetail {
                                        contact_id: contact.id,
                                        dialog: crate::components::contacts::ListDialogReference::Idle,
                                        before_ts: None,
                                        before_id: None,
                                    });
                            },
                            "View"
                        }
                        ChangeButton {
                            on_click: {
                                let q = q.clone();
                                move |_| {
                                    navigator()
                                        .push(Route::PhoneCallList {
                                            dialog: ListDialogReference::Update {
                                                contact_id: contact.id,
                                            },
                                            q: q.clone(),
                                            before_ts: None,
                                            before_id: None,
                                        });
                                }
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
pub fn PhoneCallList(
    dialog: ReadSignal<Option<ListDialogReference>>,
    q: ReadSignal<String>,
    before_ts: ReadSignal<Option<DateTime<Utc>>>,
    before_id: ReadSignal<Option<PhoneCallId>>,
) -> Element {
    let user = use_user().ok().flatten();

    let Some(_user) = user.as_ref() else {
        return rsx! {
            p { class: "alert alert-error", "You are not logged in." }
        };
    };

    let selected: Signal<Option<PhoneCallId>> = use_signal(|| None);

    let dialog_resource: Resource<Result<ActiveDialog, ServerFnError>> =
        use_resource(move || async move {
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

    // Fetch PAGE_SIZE + 1 rows so we can detect whether a next page exists.
    let mut list: Resource<Result<Vec<(PhoneCall, Contact)>, ServerFnError>> =
        use_resource(move || async move {
            search_phone_calls_paginated(q(), before_ts(), before_id(), PAGE_SIZE).await
        });

    rsx! {
        div { class: "ml-2 mr-2",
            div { class: "mb-2",
                input {
                    class: "form-control",
                    r#type: "text",
                    value: q(),
                    oninput: move |e| {
                        // Push a new URL when the query changes, resetting the cursor.
                        // Using replace() so typing doesn't pile up in the back stack.
                        navigator().replace(Route::PhoneCallList {
                            dialog: ListDialogReference::Idle,
                            q: e.value(),
                            before_ts: None,
                            before_id: None,
                        });
                    },
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
            Some(Ok(rows)) => {
                let has_next = rows.len() > PAGE_SIZE as usize;
                let visible = &rows[..rows.len().min(PAGE_SIZE as usize)];

                if visible.is_empty() {
                    rsx! {
                        p { class: "alert alert-info", "No entries found." }
                    }
                } else {
                    rsx! {
                        div { class: "ml-2 mr-2 sm:ml-0 sm:mr-0",
                            table { class: "block sm:table",
                                thead { class: "hidden sm:table-header-group",
                                    tr {
                                        th { "Time" }
                                        th { "Contact" }
                                        th { "Action" }
                                        th { "Source" }
                                        th { "Destination" }
                                    }
                                }
                                tbody { class: "block sm:table-row-group",
                                    for (phone_call, contact) in visible.iter() {
                                        EntryRow {
                                            phone_call: phone_call.clone(),
                                            contact: contact.clone(),
                                            selected,
                                            q: q(),
                                        }
                                    }
                                }
                            }

                            // ── Pagination ───────────────────────────────────
                            div { class: "flex gap-4 mt-4 ml-2",
                                if before_ts().is_some() {
                                    NavButton {
                                        on_click: move |_| { navigator().go_back(); },
                                        "Previous"
                                    }
                                }
                                if has_next {
                                    if let Some((last_call, _)) = visible.last() {
                                        NavButton {
                                            on_click: {
                                                let ts = last_call.inserted_at;
                                                let id = last_call.id;
                                                let q_val = q();
                                                move |_| {
                                                    navigator().push(Route::PhoneCallList {
                                                        dialog: ListDialogReference::Idle,
                                                        q: q_val.clone(),
                                                        before_ts: Some(ts),
                                                        before_id: Some(id),
                                                    });
                                                }
                                            },
                                            "Next"
                                        }
                                    }
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

        match dialog_resource.read().deref() {
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
                                q: q(),
                                before_ts: before_ts(),
                                before_id: before_id(),
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

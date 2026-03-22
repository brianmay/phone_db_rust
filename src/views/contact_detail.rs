#![allow(non_snake_case)]

use std::ops::Deref;

use chrono::{DateTime, Local, Utc};
use dioxus::prelude::*;
use dioxus_fullstack::ServerFnError;
use dioxus_router::navigator;

use crate::{
    Route,
    components::{
        Markdown,
        buttons::{ChangeButton, DeleteButton, NavButton},
        contacts::ListDialogReference,
    },
    functions::{contacts::get_contact_by_id, phone_calls::get_phone_calls_for_contact},
    models::{
        contacts::ContactId,
        phone_calls::{PhoneCall, PhoneCallId},
    },
    use_user,
};

const PAGE_SIZE: i64 = 50;

#[component]
fn CallRow(call: PhoneCall) -> Element {
    rsx! {
        tr { class: "border-blue-300 mt-2 mb-2 p-2 border-2 w-full sm:w-auto sm:border-none inline-block sm:table-row",
            td { class: "block sm:table-cell border-blue-300 sm:border-t-2",
                {
                    call.inserted_at
                        .with_timezone(&Local)
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string()
                }
            }
            td { class: "block sm:table-cell border-blue-300 sm:border-t-2",
                span { class: "sm:hidden", "Source: " }
                {call.source_number.clone()}
            }
            td { class: "block sm:table-cell border-blue-300 sm:border-t-2",
                span { class: "sm:hidden", "Destination: " }
                if let Some(dest) = &call.destination_number {
                    {dest.clone()}
                }
            }
            td { class: "block sm:table-cell border-blue-300 sm:border-t-2",
                span { class: "sm:hidden", "Action: " }
                {call.action.clone()}
            }
        }
    }
}

#[component]
pub fn ContactDetail(
    contact_id: ContactId,
    before_ts: ReadSignal<Option<DateTime<Utc>>>,
    before_id: ReadSignal<Option<PhoneCallId>>,
) -> Element {
    let user = use_user().ok().flatten();
    let Some(_user) = user.as_ref() else {
        return rsx! {
            p { class: "alert alert-error", "You are not logged in." }
        };
    };

    let contact_resource = use_resource(move || async move { get_contact_by_id(contact_id).await });

    // Fetch one extra row to detect whether a next page exists.
    let calls_resource: Resource<Result<Vec<PhoneCall>, ServerFnError>> =
        use_resource(move || async move {
            get_phone_calls_for_contact(contact_id, before_ts(), before_id(), PAGE_SIZE).await
        });

    let navigator = navigator();

    rsx! {
        // ── Contact detail card ──────────────────────────────────────────────
        match contact_resource() {
            Some(Ok(Some(contact))) => rsx! {
                div { class: "ml-2 mr-2 mb-4",
                    table { class: "table table-striped mb-4",
                        tbody {
                            tr {
                                th { class: "pr-4 text-left", "Phone Number" }
                                td { {contact.phone_number.clone()} }
                            }
                            tr {
                                th { class: "pr-4 text-left", "Name" }
                                td {
                                    if let Some(name) = &contact.name {
                                        {name.clone()}
                                    }
                                }
                            }
                            tr {
                                th { class: "pr-4 text-left", "Action" }
                                td { {contact.action.clone()} }
                            }
                            tr {
                                th { class: "pr-4 text-left", "Comments" }
                                td {
                                    if let Some(comments) = &contact.comments {
                                        Markdown { content: comments.clone() }
                                    }
                                }
                            }
                            tr {
                                th { class: "pr-4 text-left", "Total Calls" }
                                td { {contact.phone_call_count.to_string()} }
                            }
                        }
                    }

                    div { class: "flex flex-wrap gap-2",
                        ChangeButton {
                            on_click: move |_| {
                                navigator.push(Route::ContactList {
                                    dialog: ListDialogReference::Update {
                                        contact_id,
                                    },
                                });
                            },
                            "Edit"
                        }
                        DeleteButton {
                            on_click: move |_| {
                                navigator.push(Route::ContactList {
                                    dialog: ListDialogReference::Delete {
                                        contact_id,
                                    },
                                });
                            },
                            "Delete"
                        }
                        NavButton {
                            on_click: move |_| {
                                navigator.push(Route::ContactList {
                                    dialog: ListDialogReference::Idle,
                                });
                            },
                            "Back to Contacts"
                        }
                    }
                }
            },
            Some(Ok(None)) => rsx! {
                div { class: "alert alert-error", "Contact not found." }
            },
            Some(Err(err)) => rsx! {
                div { class: "alert alert-error", "Error: " {err.to_string()} }
            },
            None => rsx! {
                div { class: "alert alert-info", "Loading..." }
            },
        }

        // ── Call history table ───────────────────────────────────────────────
        match calls_resource.read().deref() {
            None => rsx! {
                p { class: "alert alert-info", "Loading calls..." }
            },
            Some(Err(err)) => rsx! {
                div { class: "alert alert-error", "Error loading calls: " {err.to_string()} }
            },
            Some(Ok(calls)) => {
                // We requested PAGE_SIZE + 1 rows; any extra means there is a next page.
                let has_next = calls.len() > PAGE_SIZE as usize;
                let visible = &calls[..calls.len().min(PAGE_SIZE as usize)];

                rsx! {
                    div { class: "ml-2 mr-2 sm:ml-0 sm:mr-0",
                        if visible.is_empty() {
                            p { class: "alert alert-info", "No calls recorded." }
                        } else {
                            table { class: "block sm:table w-full",
                                thead { class: "hidden sm:table-header-group",
                                    tr {
                                        th { "Time" }
                                        th { "Source" }
                                        th { "Destination" }
                                        th { "Action" }
                                    }
                                }
                                tbody { class: "block sm:table-row-group",
                                    for call in visible.iter() {
                                        CallRow { call: call.clone() }
                                    }
                                }
                            }

                            // ── Pagination ───────────────────────────────────
                            div { class: "flex gap-4 mt-4 ml-2",
                                // "Previous" = browser back — no explicit button needed.
                                if before_ts().is_some() {
                                    NavButton {
                                        on_click: move |_| { navigator.go_back(); },
                                        "Previous"
                                    }
                                }
                                if has_next {
                                    // Cursor = last visible row.
                                    if let Some(last) = visible.last() {
                                        NavButton {
                                            on_click: {
                                                let ts = last.inserted_at;
                                                let id = last.id;
                                                move |_| {
                                                    navigator.push(Route::ContactDetail {
                                                        contact_id,
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
        }

        // Dialogs (edit/delete) are handled by navigating to ContactList;
        // nothing to render here.
    }
}

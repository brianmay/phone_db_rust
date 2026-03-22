#![allow(non_snake_case)]

use std::ops::Deref;

use dioxus::prelude::*;
use dioxus_fullstack::ServerFnError;
use dioxus_router::navigator;
use tap::Pipe;

use crate::{
    Route,
    components::{
        Markdown,
        buttons::{ChangeButton, CreateButton, NavButton},
        contacts::{ActiveDialog, ContactDialog, ListDialogReference, Operation},
    },
    functions::contacts::{get_contact_by_id, search_contacts_paginated},
    models::contacts::{Contact, ContactId},
    use_user,
};

const PAGE_SIZE: i64 = 50;

#[component]
fn EntryRow(
    contact: Contact,
    selected: Signal<Option<ContactId>>,
    q: String,
    before_id: Option<ContactId>,
    before_name: Option<String>,
    before_name_null: bool,
) -> Element {
    let id = contact.id;
    let navigator = navigator();

    rsx! {
        tr {
            class: "hover:bg-gray-500 border-blue-300 mt-2 mb-2 p-2 border-2 w-full sm:w-auto sm:border-none inline-block sm:table-row",
            onclick: move |_| { selected.set(Some(id)) },
            td { class: "block sm:table-cell border-blue-300 sm:border-t-2",
                {contact.phone_number.clone()}
            }
            td { class: "block sm:table-cell border-blue-300 sm:border-t-2",
                if let Some(name) = &contact.name {
                    {name.clone()}
                }
            }
            td { class: "block sm:table-cell border-blue-300 sm:border-t-2",
                span { class: "sm:hidden", "Action: " }
                {contact.action.clone()}
            }
            td { class: "block sm:table-cell border-blue-300 sm:border-t-2",
                if let Some(comments) = &contact.comments {
                    Markdown { content: comments.to_string() }
                }
            }
            td { class: "block sm:table-cell border-blue-300 sm:border-t-2",
                span { class: "sm:hidden", "Calls: " }
                {contact.phone_call_count.to_string()}
            }
        }

        if selected() == Some(id) {
            tr {
                td { colspan: "6", class: "block sm:table-cell",
                    div { class: "flex gap-2",
                        NavButton {
                            on_click: move |_| {
                                navigator
                                    .push(Route::ContactDetail {
                                        contact_id: id,
                                        before_ts: None,
                                        before_id: None,
                                    });
                            },
                            "View"
                        }
                        ChangeButton {
                            on_click: {
                                let q = q.clone();
                                let before_name = before_name.clone();
                                move |_| {
                                    navigator
                                        .push(Route::ContactList {
                                            dialog: ListDialogReference::Update {
                                                contact_id: id,
                                            },
                                            q: q.clone(),
                                            before_id,
                                            before_name: before_name.clone(),
                                            before_name_null,
                                        });
                                }
                            },
                            "Edit"
                        }
                        ChangeButton {
                            on_click: {
                                let q = q.clone();
                                let before_name = before_name.clone();
                                move |_| {
                                    navigator
                                        .push(Route::ContactList {
                                            dialog: ListDialogReference::Delete {
                                                contact_id: id,
                                            },
                                            q: q.clone(),
                                            before_id,
                                            before_name: before_name.clone(),
                                            before_name_null,
                                        });
                                }
                            },
                            "Delete"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ContactList(
    dialog: ReadSignal<Option<ListDialogReference>>,
    q: ReadSignal<String>,
    before_id: ReadSignal<Option<ContactId>>,
    before_name: ReadSignal<Option<String>>,
    before_name_null: ReadSignal<bool>,
) -> Element {
    let user = use_user().ok().flatten();

    let Some(_user) = user.as_ref() else {
        return rsx! {
            p { class: "alert alert-error", "You are not logged in." }
        };
    };

    let selected: Signal<Option<ContactId>> = use_signal(|| None);

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
    let mut list: Resource<Result<Vec<Contact>, ServerFnError>> =
        use_resource(move || async move {
            search_contacts_paginated(
                q(),
                before_id(),
                before_name(),
                before_name_null(),
                PAGE_SIZE,
            )
            .await
        });

    rsx! {
        div { class: "ml-2 mr-2",
            div { class: "mb-2",
                CreateButton {
                    on_click: move |_| {
                        navigator().push(Route::ContactList {
                            dialog: ListDialogReference::Create,
                            q: q(),
                            before_id: before_id(),
                            before_name: before_name(),
                            before_name_null: before_name_null(),
                        });
                    },
                    "Create"
                }
            }

            div { class: "mb-2",
                input {
                    class: "form-control",
                    r#type: "text",
                    value: q(),
                    oninput: move |e| {
                        // Reset cursor when query changes; replace so typing
                        // doesn't pile up in the back stack.
                        navigator().replace(Route::ContactList {
                            dialog: ListDialogReference::Idle,
                            q: e.value(),
                            before_id: None,
                            before_name: None,
                            before_name_null: false,
                        });
                    },
                    placeholder: "Search...",
                }
            }
        }

        match list.read().deref() {
            Some(Err(err)) => rsx! {
                div { class: "alert alert-error",
                    "Error loading contacts: "
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
                                        th { "Phone Number" }
                                        th { "Name" }
                                        th { "Actions" }
                                        th { "Comments" }
                                        th { "Calls" }
                                    }
                                }
                                tbody { class: "block sm:table-row-group",
                                    for contact in visible.iter() {
                                        EntryRow {
                                            // Broken, See https://github.com/dioxuslabs/dioxus/issues/4066
                                            // key: "{contact.id}",
                                            contact: contact.clone(),
                                            selected,
                                            q: q(),
                                            before_id: before_id(),
                                            before_name: before_name(),
                                            before_name_null: before_name_null(),
                                        }
                                    }
                                }
                            }

                            // ── Pagination ───────────────────────────────────
                            div { class: "flex gap-4 mt-4 ml-2",
                                if before_id().is_some() {
                                    NavButton {
                                        on_click: move |_| { navigator().go_back(); },
                                        "Previous"
                                    }
                                }
                                if has_next {
                                    if let Some(last) = visible.last() {
                                        NavButton {
                                            on_click: {
                                                let last_id = last.id;
                                                let last_name = last.name.clone();
                                                let last_name_null = last.name.is_none();
                                                let q_val = q();
                                                move |_| {
                                                    navigator().push(Route::ContactList {
                                                        dialog: ListDialogReference::Idle,
                                                        q: q_val.clone(),
                                                        before_id: Some(last_id),
                                                        before_name: last_name.clone(),
                                                        before_name_null: last_name_null,
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
                            .push(Route::ContactList {
                                dialog: ListDialogReference::Idle,
                                q: q(),
                                before_id: before_id(),
                                before_name: before_name(),
                                before_name_null: before_name_null(),
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

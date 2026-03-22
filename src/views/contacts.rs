use std::ops::Deref;

use dioxus::prelude::*;
use dioxus_fullstack::ServerFnError;
use dioxus_router::navigator;
use tap::Pipe;

use crate::{
    Route,
    components::{
        Markdown,
        buttons::{ChangeButton, CreateButton},
        contacts::{ActiveDialog, ContactDialog, ListDialogReference, Operation},
    },
    functions::contacts::{get_contact_by_id, search_contacts},
    models::contacts::{Contact, ContactId},
    use_user,
};

#[component]
fn EntryRow(contact: Contact, selected: Signal<Option<ContactId>>) -> Element {
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
                        ChangeButton {
                            on_click: move |_| {
                                navigator
                                    .push(Route::ContactList {
                                        dialog: ListDialogReference::Update {
                                            contact_id: id,
                                        },
                                    });
                            },
                            "Edit"
                        }
                        ChangeButton {
                            on_click: move |_| {
                                navigator
                                    .push(Route::ContactList {
                                        dialog: ListDialogReference::Delete {
                                            contact_id: id,
                                        },
                                    });
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
pub fn ContactList(dialog: ReadSignal<Option<ListDialogReference>>) -> Element {
    let user = use_user().ok().flatten();

    let Some(_user) = user.as_ref() else {
        return rsx! {
            p { class: "alert alert-error", "You are not logged in." }
        };
    };

    let selected: Signal<Option<ContactId>> = use_signal(|| None);

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

    let navigator = navigator();
    let mut list: Resource<Result<Vec<Contact>, ServerFnError>> =
        use_resource(move || async move { search_contacts(query()).await });

    rsx! {
        div { class: "ml-2 mr-2",
            div { class: "mb-2",
                CreateButton {
                    on_click: move |_| {
                        navigator
                            .push(Route::ContactList {
                                dialog: ListDialogReference::Create,
                            });
                    },
                    "Create"
                }
            }

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
                    "Error loading contacts: "
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
                                th { "Phone Number" }
                                th { "Name" }
                                th { "Actions" }
                                th { "Comments" }
                                th { "Calls" }
                            }
                        }
                        tbody { class: "block sm:table-row-group",
                            for contact in list.iter() {
                                EntryRow {
                                    // Borken, See https://github.com/dioxuslabs/dioxus/issues/4066
                                    // key: "{contact.contact.id.as_inner().to_string()}",
                                    contact: contact.clone(),
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
                        navigator
                            .push(Route::ContactList {
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

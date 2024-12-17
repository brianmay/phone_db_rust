#![allow(non_snake_case)]

use super::page::{Footer, NavBar};
use crate::{
    components::phone_calls::PhoneCallList,
    database::{self, contacts::update_contact},
    Props,
};
use common::{Action, ContactDetails, ContactKey, Page, PhoneCallKey};
use dioxus::prelude::*;
use sqlx::PgPool;

#[component]
fn Contact(contact: ReadOnlySignal<ContactDetails>, on_edit_contact: Callback<i64>) -> Element {
    let contact = contact.read();
    let contact_id = contact.id;

    rsx! {
        tr {
            td { { contact.phone_number.clone() } }
            td { { contact.name.clone() } }
            td { { contact.action.to_string()} }
            td { { contact.number_calls.unwrap_or(-1).to_string() } }
            td {
                a { href: format!("/contacts/{}", contact.id), "View" }
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

    let mut edit_contact = use_signal(|| None);

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
                                    Contact {
                                        key: contact.id,
                                        contact: contact.clone(),
                                        on_edit_contact: move |contact_id| {
                                            edit_contact.set(Some(contact_id));
                                        }
                                    }
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

            if let Some(contact_id) = *edit_contact.read() {
                EditContactDialog{
                    contact_id: contact_id,
                    on_cancel: move || {
                        edit_contact.set(None);
                    },
                    on_save: move || {
                        edit_contact.set(None);
                        contacts.set(None);
                        let mut writable = request.write();
                        writable.after_key = None;
                    }
                }
            }
        }

        Footer {}

    }
}

pub fn ContactDetailView(props: Props<i64>) -> Element {
    let mut phone_calls = use_signal(|| None);
    let mut request = use_signal(|| common::PageRequest::<PhoneCallKey> {
        after_key: None,
        search: None,
    });

    use_context_provider(|| props.state.db.clone());

    let id = props.path;

    let mut edit = use_signal(|| false);

    let mut contact_resource = use_resource(move || {
        let db = props.state.db.clone();
        async move { database::contacts::get_contact(&db, id).await }
    });

    rsx! {
        NavBar {}

        main {
            role: "main",
            class: "container",


            {
                match &*contact_resource.read() {
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

                            if *edit.read() {
                                EditContactDialog {
                                    contact_id: id,
                                    on_save: move || {
                                        edit.set(false);
                                        contact_resource.restart();

                                        //reset phone call list
                                        phone_calls.set(None);
                                        let mut writable = request.write();
                                        writable.after_key = None;
                                    },
                                    on_cancel: move || {
                                        edit.set(false);
                                    }
                                }
                            } else {
                                button {
                                    class: "btn btn-primary",
                                    onclick: move |_| {
                                        edit.set(true);
                                    },
                                    "Edit"
                                }
                            }

                            PhoneCallList {
                                contact_id: Some(id),
                                phone_calls: phone_calls,
                                request: request,
                                // This is required but not used, as no edit buttons should be shown.
                                on_edit_contact: |_| {}
                            }
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

enum Saving {
    No,
    Yes,
    Finished(Result<(), sqlx::Error>),
}

#[component]
pub fn EditContactDialog(
    contact_id: i64,
    on_save: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut name = use_signal(String::default);
    let mut action = use_signal(Action::default);
    let mut comments = use_signal(String::default);

    let contact_resource = use_resource(move || async move {
        let db = use_context::<PgPool>();
        let contact = database::contacts::get_contact(&db, contact_id).await;
        if let Ok(contact) = contact.as_ref() {
            name.set(contact.name.clone().unwrap_or("".to_string()));
            action.set(contact.action);
            comments.set(contact.comments.clone().unwrap_or("".to_string()));
        } else {
            name.set(String::default());
            action.set(Action::default());
            comments.set(String::default());
        }
        contact
    });

    let mut saving_resource = use_signal(|| Saving::No);

    let contact = contact_resource.read();
    let saving = saving_resource.read();
    let disabled = !matches!(*contact, Some(Ok(_))) || matches!(*saving, Saving::Yes);
    let status = match (&*contact, &*saving) {
        (_, Saving::Yes) => rsx! {
            div {
                class: "alert alert-primary",
                "Saving..."
            }
        },
        (_, Saving::Finished(Ok(_))) => rsx! {
            div {
                class: "alert alert-success",
                "Contact saved"
            }
        },
        (_, Saving::Finished(Err(err))) => rsx! {
            div {
                class: "alert alert-danger",
                "Error saving contact: {err}"
            }
        },
        (Some(Ok(_)), _) => rsx! {},
        (Some(Err(err)), _) => rsx! {
            div {
                class: "alert alert-danger",
                "Error loading contact: {err}"
            }
        },
        (None, _) => rsx! {
            div {
                class: "alert alert-info",
                "Loading contact..."
            }
        },
    };

    rsx! {
        div {
            class: "modal fade show d-block",
            id: "editContactDialog",
            tabindex: "-1",
            role: "dialog",
            aria_labelledby: "editContactDialogLabel",

            "data-bs-backdrop": "static",
            // aria_hidden: "true",
            div {
                class: "modal-dialog",
                role: "document",
                div {
                    class: "modal-content",
                    div {
                        class: "modal-header",
                        h5 {
                            class: "modal-title",
                            id: "editContactDialogLabel",
                            "Edit Contact"
                        }
                        button {
                            type: "button",
                            class: "btn-close",
                            // "data_dismiss": "modal",
                            aria_label: "Close",
                            onclick: move |_event| {
                                on_cancel.call(());
                            }
                        }
                    }
                    div {
                        class: "modal-body",
                        { status }

                        form {
                            div {
                                class: "form-group",
                                label {
                                    for: "name",
                                    "Name"
                                }
                                input {
                                    type: "text",
                                    class: "form-control",
                                    id: "name",
                                    placeholder: "Enter name",
                                    value: "{name}",
                                    disabled: disabled,
                                    oninput: move |e| {
                                        name.set(e.value());
                                    }
                                }
                            }
                            div {
                                class: "form-group",
                                label {
                                    for: "action",
                                    "Action"
                                }
                                select {
                                    class: "form-control",
                                    id: "action",
                                    disabled: disabled,
                                    value: "{action}",
                                    oninput: move |e| {
                                        action.set(e.value().into());
                                    },
                                    for op in Action::get_all_options_as_str() {
                                        option {
                                            value: "{op.1}",
                                             "{op.0}"
                                        }
                                    }
                                }
                            }
                            div {
                                class: "form-group",
                                label {
                                    for: "comments",
                                    "Comments"
                                }
                                textarea {
                                    class: "form-control",
                                    id: "comments",
                                    rows: "3",
                                    placeholder: "Enter comments",
                                    value: "{comments}",
                                    disabled: disabled,
                                    oninput: move |e| {
                                        comments.set(e.value());
                                    },
                                }
                            }
                        }
                    }
                    div {
                        class: "modal-footer",
                        button {
                            type: "button",
                            class: "btn btn-secondary",
                            onclick: move |_event| {
                                on_cancel.call(());
                            },
                            "data_dismiss": "modal",
                            "Close"

                        }
                        button {
                            type: "button",
                            class: "btn btn-primary",
                            disabled: disabled,
                            onclick: move |_event| {
                                saving_resource.set(Saving::Yes);

                                spawn(async move {
                                    let name = name.read_unchecked().clone();
                                    let name = if name.is_empty() { None } else { Some(name) };
                                    let comments = comments.read().clone();
                                    let comments = if comments.is_empty() { None } else { Some(comments) };

                                    let request = common::ContactUpdateRequest {
                                        id: contact_id,
                                        name,
                                        action: *action.read(),
                                        comments,
                                    };

                                    let db = use_context::<PgPool>();
                                    let result = update_contact(&db, &request).await;
                                    let ok = result.is_ok();
                                    saving_resource.set(Saving::Finished(result));
                                    if ok {
                                        on_save.call(());
                                    }
                                });
                            },
                            "Save changes"
                        }
                    }
                }
            }
        }
        div {
            class: "modal-backdrop fade show"
        }
    }
}

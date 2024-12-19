#![allow(non_snake_case)]

use super::{
    common::{EditError, ValidationError},
    page::{Footer, NavBar},
    validation::validate_comments,
};
use crate::{
    components::{
        app::Route,
        common::{ActiveDialog, InputSelect, InputString, InputTextArea, Operation, Saving},
        phone_calls::PhoneCallList,
        validation::{validate_action, validate_name, validate_phone_number},
    },
    database::{
        self,
        contacts::{add_contact, delete_contact, update_contact},
    },
};
use common::{Action, ContactDetails, ContactKey, Page, PhoneCallKey};
use dioxus::prelude::*;
use dioxus_router::prelude::navigator;
use sqlx::PgPool;

#[component]
fn ContactComponent(
    contact: ReadOnlySignal<ContactDetails>,
    on_edit_contact: Callback<i64>,
    on_delete_contact: Callback<i64>,
) -> Element {
    let contact = contact.read();
    let contact_id = contact.id;

    let handler = move |_| {
        let navigator = navigator();
        navigator.push(Route::ContactDetailView { contact_id });
    };

    rsx! {
        tr {
            td { onclick: handler, { contact.phone_number.clone() } }
            td { onclick: handler, { contact.name.clone() } }
            td { onclick: handler, { contact.action.to_string()} }
            td { onclick: handler, { contact.number_calls.unwrap_or(-1).to_string() } }
            td {
                button {
                    class: "btn btn-primary",
                    onclick: move |_| {
                        on_edit_contact.call(contact_id);
                    },
                    "Edit"
                }
                button {
                    class: "btn btn-danger",
                    disabled: contact.number_calls.unwrap_or(0) > 0,
                    onclick: move |_| {
                        on_delete_contact.call(contact_id);
                    },
                    "Delete"
                }
            }

        }
    }
}

#[component]
pub fn ContactListView() -> Element {
    let mut request = use_signal(|| common::PageRequest::<ContactKey> {
        after_key: None,
        search: None,
    });

    let mut contacts =
        use_signal(|| Option::<Result<Page<ContactDetails, ContactKey>, sqlx::Error>>::None);

    let db = use_context::<PgPool>();
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

    let mut edit_contact = use_signal(|| ActiveDialog::Idle);

    rsx! {

        NavBar {}


        main {
            role: "main",
            class: "container",

            h1 {
                "Listing Contacts"
            }

            button {
                class: "btn btn-primary",
                onclick: move |_| {
                    edit_contact.set(ActiveDialog::Editing(Operation::Add));
                },
                "Add Contact"
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
                                    ContactComponent {
                                        key: contact.id,
                                        contact: contact.clone(),
                                        on_edit_contact: move |contact_id| {
                                            edit_contact.set(ActiveDialog::Editing(Operation::Edit(contact_id)));
                                        },
                                        on_delete_contact: move |contact_id| {
                                            edit_contact.set(ActiveDialog::Deleting(contact_id));
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

            if let ActiveDialog::Editing(contact_id) = *edit_contact.read() {
                EditContactDialog{
                    contact_id: contact_id,
                    on_cancel: move || {
                        edit_contact.set(ActiveDialog::Idle);
                    },
                    on_save: move || {
                    edit_contact.set(ActiveDialog::Idle);
                        contacts.set(None);
                        let mut writable = request.write();
                        writable.after_key = None;
                    }
                }
            }

            if let ActiveDialog::Deleting(contact_id) = *edit_contact.read() {
                ConfirmDeleteDialog {
                    contact_id: contact_id,
                    on_cancel: move || {
                        edit_contact.set(ActiveDialog::Idle);
                    },
                    on_delete: move || {
                        edit_contact.set(ActiveDialog::Idle);
                        contacts.set(None);
                        let mut writable = request.write();
                        writable.after_key = None;
                        let navigator = navigator();
                        navigator.push(Route::ContactListView{});
                    }
                }
            }
        }

        Footer {}

    }
}

#[component]
pub fn ContactDetailView(contact_id: i64) -> Element {
    let phone_calls = use_signal(|| None);
    let request = use_signal(|| common::PageRequest::<PhoneCallKey> {
        after_key: None,
        search: None,
    });

    let id = contact_id;

    let mut edit = use_signal(|| ActiveDialog::Idle);

    let db = use_context::<PgPool>();
    let mut contact_resource = use_resource(move || {
        let db = db.clone();
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

                            match &*edit.read() {
                                ActiveDialog::Deleting(id) => {
                                    rsx!{
                                        ConfirmDeleteDialog {
                                            contact_id: *id,
                                            on_delete: move || {
                                                edit.set(ActiveDialog::Idle);
                                                contact_resource.restart();
                                            },
                                            on_cancel: move || {
                                                edit.set(ActiveDialog::Idle);
                                            }
                                        }
                                    }
                                }
                                ActiveDialog::Editing(id) => {
                                    rsx! {
                                        EditContactDialog {
                                            contact_id: *id,
                                            on_save: move || {
                                                edit.set(ActiveDialog::Idle);
                                                contact_resource.restart();
                                            },
                                            on_cancel: move || {
                                                edit.set(ActiveDialog::Idle);
                                            }
                                        }
                                    }
                                }
                                ActiveDialog::Idle => {
                                    rsx! {
                                        div {
                                            button {
                                                class: "btn btn-primary",
                                                onclick: move |_| {
                                                    edit.set(ActiveDialog::Editing(Operation::Edit(id)));
                                                },
                                                "Edit"
                                            }
                                            button {
                                                class: "btn btn-danger",
                                                disabled: contact.number_calls.unwrap_or(0) > 0,
                                                onclick: move |_| {
                                                    edit.set(ActiveDialog::Deleting(id));
                                                },
                                                "Delete"
                                            }
                                        }
                                    }
                                }
                            }

                            h2 {
                                "Phone Calls"
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
                        rsx! { "An error occurred while fetching contact: {err}" }
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

async fn save(
    contact_id: Operation,
    phone_number: Result<String, ValidationError>,
    name: Result<String, ValidationError>,
    action: Result<Action, ValidationError>,
    comments: Result<Option<String>, ValidationError>,
) -> Result<(), EditError> {
    let phone_number = phone_number?;
    let name = name?;
    let action = action?;
    let comments = comments?;

    let db = use_context::<PgPool>();

    if let Operation::Edit(id) = contact_id {
        let request = common::ContactUpdateRequest {
            id,
            phone_number,
            name: Some(name),
            action,
            comments,
        };

        update_contact(&db, &request).await?;
    } else {
        let request = common::ContactAddRequest {
            phone_number,
            name: Some(name),
            action,
            comments,
        };

        add_contact(&db, &request).await?;
    }

    Ok(())
}

#[component]
pub fn EditContactDialog(
    contact_id: Operation,
    on_save: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut phone_number = use_signal(String::default);
    let mut name = use_signal(String::default);
    let mut action = use_signal(String::default);
    let mut comments = use_signal(String::default);

    let validate_phone_number =
        use_memo(move || validate_phone_number(&phone_number.read_unchecked()));
    let validate_name = use_memo(move || validate_name(&name.read_unchecked()));
    let validate_action = use_memo(move || validate_action(&action.read_unchecked()));
    let validate_comments = use_memo(move || validate_comments(&comments.read_unchecked()));

    let changed_phone_number = use_signal(|| false);
    let changed_name = use_signal(|| false);
    let changed_action = use_signal(|| false);
    let changed_comments = use_signal(|| false);

    let contact_resource = use_resource(move || async move {
        if let Operation::Edit(contact_id) = contact_id {
            let db = use_context::<PgPool>();
            let contact = database::contacts::get_contact(&db, contact_id).await;
            if let Ok(contact) = contact.as_ref() {
                phone_number.set(contact.phone_number.clone());
                name.set(contact.name.clone().unwrap_or_default());
                action.set(contact.action.as_str().to_string());
                comments.set(contact.comments.clone().unwrap_or_default());
            } else {
                phone_number.set(String::default());
                name.set(String::default());
                action.set(String::default());
                comments.set(String::default());
            }
            contact.map(|_| ())
        } else {
            phone_number.set(String::default());
            name.set(String::default());
            action.set(String::default());
            comments.set(String::default());
            Ok(())
        }
    });

    let mut saving_resource = use_signal(|| Saving::No);

    let is_valid = [
        validate_phone_number().is_ok(),
        validate_name().is_ok(),
        validate_action().is_ok(),
        validate_comments().is_ok(),
    ]
    .into_iter()
    .all(|x| x);

    let contact = contact_resource.read();
    let saving = saving_resource.read();
    let disabled = !matches!(*contact, Some(Ok(_))) || matches!(*saving, Saving::Yes);
    let disabled_save = disabled || !is_valid;
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
                            InputString {
                                id: "phone_number",
                                label: "Phone Number",
                                validate: validate_phone_number,
                                changed: changed_phone_number,
                                value: phone_number,
                                disabled: disabled,
                            }

                            InputString {
                                id: "name",
                                label: "Name",
                                validate: validate_name,
                                changed: changed_name,
                                value: name,
                                disabled: disabled,
                            }

                            InputSelect {
                                id: "action",
                                label: "Action",
                                validate: validate_action,
                                changed: changed_action,
                                value: action,
                                disabled: disabled,
                                options: Action::get_all_options_as_str(),
                            }

                            InputTextArea {
                                id: "comments",
                                label: "Comments",
                                validate: validate_comments,
                                changed: changed_comments,
                                value: comments,
                                disabled: disabled,
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
                            disabled: disabled_save,
                            onclick: move |_event| {
                                saving_resource.set(Saving::Yes);

                                spawn(async move {
                                    let result = save(contact_id, validate_phone_number(), validate_name(), validate_action(), validate_comments()).await;
                                    let is_ok = result.is_ok();
                                    saving_resource.set(Saving::Finished(result));
                                    if is_ok {
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

#[component]
fn ConfirmDeleteDialog(
    contact_id: i64,
    on_delete: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut saving_resource = use_signal(|| Saving::No);

    let saving = saving_resource.read();
    let disabled = matches!(*saving, Saving::Yes);

    let status = match &*saving {
        Saving::Yes => rsx! {
            div {
                class: "alert alert-primary",
                "Deleting..."
            }
        },
        Saving::No => rsx! {},
        Saving::Finished(Ok(_)) => rsx! {
            div {
                class: "alert alert-success",
                "Default deleted"
            }
        },
        Saving::Finished(Err(err)) => rsx! {
            div {
                class: "alert alert-danger",
                "Error deleting default: {err}"
            }
        },
    };

    rsx! {
        div {
            class: "modal fade show d-block",
            id: "confirmDeleteDialog",
            tabindex: "-1",
            role: "dialog",
            aria_labelledby: "confirmDeleteDialogLabel",
            "data-bs-backdrop": "static",
            div {
                class: "modal-dialog",
                role: "document",
                div {
                    class: "modal-content",
                    div {
                        class: "modal-header",
                        h5 {
                            class: "modal-title",
                            id: "confirmDeleteDialogLabel",
                            "Confirm Delete"
                        }
                        button {
                            type: "button",
                            class: "btn-close",
                            aria_label: "Close",
                            onclick: move |_event| {
                                on_cancel.call(());
                            }
                        }
                    }
                    div {
                        class: "modal-body",
                        { status }
                        "Are you sure you want to delete this contact?"
                    }
                    div {
                        class: "modal-footer",
                        button {
                            type: "button",
                            class: "btn btn-secondary",
                            onclick: move |_event| {
                                on_cancel.call(());
                            },
                            "Cancel"
                        }
                        button {
                            type: "button",
                            class: "btn btn-danger",
                            disabled: disabled,
                            onclick: move |_event| {
                                saving_resource.set(Saving::Yes);

                                spawn(async move {
                                    let db = use_context::<PgPool>();
                                    let result = delete_contact(&db, contact_id).await.map_err(|err| err.into());
                                    let is_ok = result.is_ok();
                                    saving_resource.set(Saving::Finished(result));
                                    if is_ok {
                                        on_delete.call(());
                                    }
                                });
                            },
                            "Delete"
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

#![allow(non_snake_case)]

use super::common::{EditError, InputString, Operation, ValidationError};
use super::page::{Footer, NavBar};
use crate::components::common::{ActiveDialog, InputSelect, Saving};
use crate::components::validation::{
    validate_action, validate_name, validate_order, validate_regexp,
};
use crate::{
    components::app::Route,
    database::{
        self,
        defaults::{add_default, delete_default, get_defaults, update_default},
    },
};
use common::{Action, Default};
use dioxus::prelude::*;
use dioxus_router::prelude::navigator;
use sqlx::PgPool;

#[component]
fn DefaultComponent(
    default: ReadOnlySignal<Default>,
    on_edit_default: Callback<i64>,
    on_delete_default: Callback<i64>,
) -> Element {
    let default = default.read();
    let default_id = default.id;

    let handler = move |_| {
        let navigator = navigator();
        navigator.push(Route::DefaultDetailView { default_id });
    };

    rsx! {
        tr {
            td { onclick: handler, { default.order.to_string() } }
            td { onclick: handler, { default.regexp.clone() } }
            td { onclick: handler, { default.name.clone() } }
            td { onclick: handler, { default.action.to_string()} }
            td {
                button {
                    class: "btn btn-primary",
                    onclick: move |_| {
                        on_edit_default.call(default_id);
                    },
                    "Edit"
                }
                button {
                    class: "btn btn-danger",
                    onclick: move |_| {
                        on_delete_default.call(default_id);
                    },
                    "Delete"
                }
            }
        }
    }
}

#[component]
pub fn DefaultListView() -> Element {
    let mut defaults = use_resource(|| async {
        let db = use_context::<PgPool>();
        get_defaults(&db).await
    });

    let mut test_phone_number: Signal<Option<String>> = use_signal(|| None);

    let mut edit_default = use_signal(|| ActiveDialog::Idle);

    rsx! {

        NavBar {}


        main {
            role: "main",
            class: "container",

            h1 {
                "Listing Defaults"
            }

            button {
                class: "btn btn-primary",
                onclick: move |_| {
                    edit_default.set(ActiveDialog::Editing(Operation::Add));
                },
                "Add Default"
            }

            {
            match &*defaults.read() {
                Some(Ok(defaults)) => {
                    let list = defaults.iter();

                    rsx! {
                        table {
                            class: "table table-hover",
                            thead {
                                class: "thead-dark",
                                tr {
                                    th { "Order" }
                                    th { "RegExp" }
                                    th { "Name" }
                                    th { "Action" }
                                    th {}
                                }
                            }
                            tbody {
                                for default in list {
                                    DefaultComponent {
                                        key: default.id,
                                        default: default.clone(),
                                        on_edit_default: move |default_id| {
                                            edit_default.set(ActiveDialog::Editing(Operation::Edit(default_id)));
                                        },
                                        on_delete_default: move |default_id| {
                                            edit_default.set(ActiveDialog::Deleting(default_id));
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
            }

            if let ActiveDialog::Editing(default_id) = *edit_default.read() {
                EditDefaultDialog{
                    default_id: default_id,
                    on_cancel: move || {
                        edit_default.set(ActiveDialog::Idle);
                    },
                    on_save: move || {
                        defaults.restart();
                        edit_default.set(ActiveDialog::Idle);
                    }
                }
            }

            if let ActiveDialog::Deleting(default_id) = *edit_default.read() {
                ConfirmDeleteDialog {
                    default_id: default_id,
                    on_cancel: move || {
                        edit_default.set(ActiveDialog::Idle);
                    },
                    on_delete: move || {
                        defaults.restart();
                        edit_default.set(ActiveDialog::Idle);
                    }
                }
            }

            form {
                input { type: "text", placeholder: "Phone number", oninput: move |e| {
                    let value = e.value();
                    let value = if value.is_empty() { None } else { Some(value) };
                    test_phone_number.set(value);
                } }
            }

            if let Some(phone_number) = &*test_phone_number.read() {
                if let Some(Ok(defaults)) = &*defaults.read() {

                    div {
                        class: "card",
                        div {
                            class: "card-header",
                            "Testing phone number: {phone_number}"
                        }

                        div {
                            class: "card-body",
                            {
                                let result = defaults.search_phone_number(phone_number);
                                match result {
                                    Some(result) => {
                                        rsx! {
                                            "default: {result:?}"
                                        }
                                    }
                                    None => {
                                        rsx! {
                                            "No default found"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

        }

        Footer {}

    }
}

#[component]
pub fn DefaultDetailView(default_id: i64) -> Element {
    let id = default_id;

    let mut edit = use_signal(|| ActiveDialog::Idle);

    let db = use_context::<PgPool>();
    let mut default_resource = use_resource(move || {
        let db = db.clone();
        async move { database::defaults::get_default(&db, id).await }
    });

    rsx! {
        NavBar {}

        main {
            role: "main",
            class: "container",


            {
                match &*default_resource.read() {
                    Some(Ok(default)) => {
                        let default_name = default.name.clone();
                        rsx! {
                            h1 {
                                "Default {default_name}"
                            }
                            table {
                                class: "table table-hover",

                                tbody {
                                    tr {
                                        th { "Order" }
                                        td { { default.order.to_string() } }
                                    }
                                    tr {
                                        th { "RegExp" }
                                        td { { default.regexp.clone() } }
                                    }
                                    tr {
                                        th { "Name" }
                                        td { { default.name.clone() } }
                                    }
                                    tr {
                                        th { "Action" }
                                        td { { default.action.to_string() } }
                                    }
                                }
                            }

                            match &*edit.read() {
                                ActiveDialog::Deleting(id) => {
                                    rsx!{
                                        ConfirmDeleteDialog {
                                            default_id: *id,
                                            on_delete: move || {
                                                edit.set(ActiveDialog::Idle);
                                                default_resource.restart();
                                                let navigator = navigator();
                                                navigator.push(Route::DefaultListView {  });
                                            },
                                            on_cancel: move || {
                                                edit.set(ActiveDialog::Idle);
                                            }
                                        }
                                    }
                                }
                                ActiveDialog::Editing(id) => {
                                    rsx! {
                                        EditDefaultDialog {
                                            default_id: *id,
                                            on_save: move || {
                                                edit.set(ActiveDialog::Idle);
                                                default_resource.restart();
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
                                                onclick: move |_| {
                                                    edit.set(ActiveDialog::Deleting(id));
                                                },
                                                "Delete"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Some(Err(err)) => {
                        rsx! { "An error occurred while fetching default {err}" }
                    }
                    None => {
                        rsx! { "Loading default" }
                    }
                }
            }
        }
        Footer {}
    }
}

async fn save(
    id: Operation,
    order: Result<i32, ValidationError>,
    regexp: Result<String, ValidationError>,
    name: Result<String, ValidationError>,
    action: Result<Action, ValidationError>,
) -> Result<(), EditError> {
    let order = order?;
    let regexp = regexp?;
    let name = name?;
    let action = action?;

    let db = use_context::<PgPool>();

    if let Operation::Edit(id) = id {
        let request = common::DefaultUpdateRequest {
            id,
            order,
            regexp,
            name,
            action,
        };
        update_default(&db, &request).await?;
    } else {
        let request = common::DefaultAddRequest {
            order,
            regexp,
            name,
            action,
        };
        add_default(&db, &request).await?;
    }

    Ok(())
}

#[component]
fn EditDefaultDialog(
    default_id: Operation,
    on_save: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut order = use_signal(String::default);
    let mut regexp = use_signal(String::default);
    let mut name = use_signal(String::default);
    let mut action = use_signal(String::default);

    let validate_order = use_memo(move || validate_order(&order.read_unchecked()));
    let validate_regexp = use_memo(move || validate_regexp(&regexp.read_unchecked()));
    let validate_name = use_memo(move || validate_name(&name.read_unchecked()));
    let validate_action = use_memo(move || validate_action(&action.read_unchecked()));

    let changed_order = use_signal(|| false);
    let changed_regexp = use_signal(|| false);
    let changed_name = use_signal(|| false);
    let changed_action = use_signal(|| false);

    let default_resource = use_resource(move || async move {
        let db = use_context::<PgPool>();
        if let Operation::Edit(default_id) = default_id {
            let default = database::defaults::get_default(&db, default_id).await;
            if let Ok(default) = &default {
                order.set(default.order.to_string());
                regexp.set(default.regexp.clone());
                name.set(default.name.clone());
                action.set(default.action.as_str().to_string());
            } else {
                order.set(String::default());
                regexp.set(String::default());
                name.set(String::default());
                action.set(String::default());
            };
            default.map(|_| ())
        } else {
            order.set(String::default());
            regexp.set(String::default());
            name.set(String::default());
            action.set(String::default());
            Ok(())
        }
    });

    let mut saving_resource = use_signal(|| Saving::No);

    let is_valid = [
        validate_order().is_ok(),
        validate_regexp().is_ok(),
        validate_name().is_ok(),
        validate_action().is_ok(),
    ]
    .into_iter()
    .all(|x| x);

    let default = default_resource.read();
    let saving = saving_resource.read();
    let disabled = !matches!(*default, Some(Ok(_))) || matches!(*saving, Saving::Yes);
    let disabled_save = disabled || !is_valid;
    let status = match (&*default, &*saving) {
        (_, Saving::Yes) => rsx! {
            div {
                class: "alert alert-primary",
                "Saving..."
            }
        },
        (_, Saving::Finished(Ok(_))) => rsx! {
            div {
                class: "alert alert-success",
                "Default saved"
            }
        },
        (_, Saving::Finished(Err(err))) => rsx! {
            div {
                class: "alert alert-danger",
                "Error saving default: {err}"
            }
        },
        (Some(Ok(_)), _) => rsx! {},
        (Some(Err(err)), _) => rsx! {
            div {
                class: "alert alert-danger",
                "Error loading default: {err}"
            }
        },
        (None, _) => rsx! {
            div {
                class: "alert alert-info",
                "Loading default..."
            }
        },
    };

    rsx! {
        div {
            class: "modal fade show d-block",
            id: "editDefaultDialog",
            tabindex: "-1",
            role: "dialog",
            aria_labelledby: "editDefaultDialogLabel",

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
                            id: "editDefaultDialogLabel",
                            "Edit Default"
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
                            novalidate: true,
                            class: "needs-validation",


                            InputString {
                                id: "order",
                                label: "Order",
                                value: order,
                                validate: validate_order,
                                changed: changed_order,
                                disabled: disabled,
                            }

                            InputString {
                                id: "regexp",
                                label: "RegExp",
                                value: regexp,
                                validate: validate_regexp,
                                changed: changed_regexp,
                                disabled: disabled,
                            }

                            InputString {
                                id: "name",
                                label: "Name",
                                value: name,
                                validate: validate_name,
                                changed: changed_name,
                                disabled: disabled,
                            }

                            InputSelect {
                                id: "action",
                                label: "Action",
                                value: action,
                                validate: validate_action,
                                changed: changed_action,
                                disabled: disabled,
                                options: Action::get_all_options_as_str(),
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
                                    let result = save(default_id, validate_order(), validate_regexp(), validate_name(), validate_action()).await;
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
    default_id: i64,
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
                        "Are you sure you want to delete this default?"
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
                                    let result = delete_default(&db, default_id).await.map_err(|err| err.into());
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

#![allow(non_snake_case)]

use super::page::{Footer, NavBar};
use crate::{
    database::{
        self,
        defaults::{add_default, delete_default, get_defaults, update_default},
    },
    Props,
};
use common::{Action, Default};
use dioxus::prelude::*;
use sqlx::PgPool;
use thiserror::Error;

#[component]
fn DefaultComponent(
    default: ReadOnlySignal<Default>,
    on_edit_default: Callback<i64>,
    on_delete_default: Callback<i64>,
) -> Element {
    let default = default.read();
    let default_id = default.id;

    rsx! {
        tr {
            td { { default.order.to_string() } }
            td { { default.regexp.clone() } }
            td { { default.name.clone() } }
            td { { default.action.to_string()} }
            td {
                a { href: format!("/defaults/{}", default.id), "View" }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Operation {
    Add,
    Edit(i64),
}

#[derive(Debug, Clone, Copy)]
enum ActiveDialog {
    Editing(Operation),
    Deleting(i64),
    Idle,
}

pub fn DefaultListView(props: Props) -> Element {
    use_context_provider(|| props.state.db.clone());

    let mut defaults = use_resource(|| async {
        let db = use_context::<PgPool>();
        get_defaults(&db).await
    });

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
        }

        Footer {}

    }
}

pub fn DefaultDetailView(props: Props<i64>) -> Element {
    use_context_provider(|| props.state.db.clone());

    let id = props.path;

    let mut edit = use_signal(|| ActiveDialog::Idle);

    let mut default_resource = use_resource(move || {
        let db = props.state.db.clone();
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

enum Saving {
    No,
    Yes,
    Finished(Result<(), EditError>),
}

#[derive(Error, Debug, PartialEq, Eq, Clone)]
#[error("{0}")]
struct ValidationError(String);

#[derive(Error, Debug)]
enum EditError {
    #[error("{0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("{0}")]
    Validation(#[from] ValidationError),
}

fn validate_order(str: &str) -> Result<i32, ValidationError> {
    if str.is_empty() {
        return Err(ValidationError("Order cannot be empty".to_string()));
    }

    str.parse()
        .map_err(|err| ValidationError(format!("Invalid integer: {err}")))
}

fn validate_regexp(str: &str) -> Result<String, ValidationError> {
    if str.is_empty() {
        return Err(ValidationError("Regexp cannot be empty".to_string()));
    }
    regex::Regex::new(str).map_err(|err| ValidationError(format!("Invalid regexp: {err}")))?;
    Ok(str.to_string())
}

fn validate_name(str: &str) -> Result<String, ValidationError> {
    if str.is_empty() {
        return Err(ValidationError("Name cannot be empty".to_string()));
    }
    Ok(str.to_string())
}

fn validate_action(str: &str) -> Result<Action, ValidationError> {
    Action::try_from(str).map_err(|err| ValidationError(format!("Invalid action: {err}")))
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

fn get_input_classes(is_valid: bool, changed: bool) -> &'static str {
    if is_valid {
        return "form-control is-valid";
    }

    if !changed {
        return "form-control";
    }

    "form-control is-invalid"
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

    let mut changed_order = use_signal(|| false);
    let mut changed_regexp = use_signal(|| false);
    let mut changed_name = use_signal(|| false);
    let mut changed_action = use_signal(|| false);

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
    let disabled_save =
        !matches!(*default, Some(Ok(_))) || matches!(*saving, Saving::Yes) || !is_valid;
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

                            div {
                                class: "form-group",
                                label {
                                    for: "order",
                                    "Order"
                                }
                                input {
                                    type: "text",
                                    class: get_input_classes(validate_order().is_ok(), changed_order()),
                                    id: "order",
                                    placeholder: "Enter order",
                                    value: "{order}",
                                    disabled: disabled,
                                    oninput: move |e| {
                                        changed_order.set(true);
                                        order.set(e.value());
                                    }
                                }
                                if let Err(err) = validate_order() {
                                    div {
                                        class: "invalid-feedback",
                                        "{err}"
                                    }
                                } else {
                                    div {
                                        class: "valid-feedback",
                                        "Looks good!"
                                    }
                                }
                            }
                            div {
                                class: "form-group",
                                label {
                                    for: "regexp",
                                    "RegExp"
                                }
                                input {
                                    type: "text",
                                    class: get_input_classes(validate_regexp().is_ok(), changed_regexp()),
                                    id: "regexp",
                                    placeholder: "Enter RegExp",
                                    value: "{regexp}",
                                    disabled: disabled,
                                    oninput: move |e| {
                                        changed_regexp.set(true);
                                        regexp.set(e.value());
                                    }
                                }
                                if let Err(err) = validate_regexp() {
                                    div {
                                        class: "invalid-feedback",
                                        "{err}"
                                    }
                                } else {
                                    div {
                                        class: "valid-feedback",
                                        "Looks good!"
                                    }
                                }
                            }
                            div {
                                class: "form-group",
                                label {
                                    for: "name",
                                    "Name"
                                }
                                input {
                                    type: "text",
                                    class: get_input_classes(validate_name().is_ok(), changed_name()),
                                    id: "name",
                                    placeholder: "Enter name",
                                    value: "{name}",
                                    disabled: disabled,
                                    oninput: move |e| {
                                        changed_name.set(true);
                                        name.set(e.value());
                                    }
                                }
                                if let Err(err) = validate_name() {
                                    div {
                                        class: "invalid-feedback",
                                        "{err}"
                                    }
                                } else {
                                    div {
                                        class: "valid-feedback",
                                        "Looks good!"
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
                                    class: get_input_classes(validate_action().is_ok(), changed_action()),
                                    id: "action",
                                    disabled: disabled,
                                    value: "{action}",
                                    oninput: move |e| {
                                        changed_action.set(true);
                                        action.set(e.value());
                                    },
                                    for op in Action::get_all_options_as_str() {
                                        option {
                                            value: "{op.1}",
                                             "{op.0}"
                                        }
                                    }
                                }
                                if let Err(err) = validate_action() {
                                    div {
                                        class: "invalid-feedback",
                                        "{err}"
                                    }
                                } else {
                                    div {
                                        class: "valid-feedback",
                                        "Looks good!"
                                    }
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

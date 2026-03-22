use std::{num::ParseIntError, str::FromStr};

use dioxus::prelude::*;
use dioxus_router::ToQueryArgument;
use tap::Pipe;
use thiserror::Error;

use crate::{
    forms::{
        Dialog, EditError, FieldValue, FormSaveCancelButton, InputString, Saving, ValidationError,
        validate_default_name, validate_regex,
    },
    functions::defaults::{create_default, delete_default, update_default},
    models::{
        common::MaybeSet,
        defaults::{ChangeDefault, Default, DefaultId, NewDefault},
    },
};

fn validate_optional_string(str: &str) -> Result<Option<String>, ValidationError> {
    <Option<String> as FieldValue>::from_raw(str).map_err(|err| ValidationError(err.to_string()))
}

fn validate_optional_order(str: &str) -> Result<Option<i32>, ValidationError> {
    <Option<i32> as FieldValue>::from_raw(str).map_err(|err| ValidationError(err.to_string()))
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Operation {
    Create,
    Update { default: Default },
}

#[derive(Debug, Clone)]
struct Validate {
    order: Memo<Result<Option<i32>, ValidationError>>,
    regexp: Memo<Result<Option<String>, ValidationError>>,
    name: Memo<Result<Option<String>, ValidationError>>,
    action: Memo<Result<Option<String>, ValidationError>>,
}

async fn do_save(op: &Operation, validate: &Validate) -> Result<Default, EditError> {
    let order = validate.order.read().clone()?;
    let regexp = validate.regexp.read().clone()?;
    let name = validate.name.read().clone()?;
    let action = validate.action.read().clone()?;

    match op {
        Operation::Create => {
            let new_default = NewDefault {
                order,
                regexp,
                name,
                action,
            };
            create_default(new_default).await.map_err(EditError::Server)
        }
        Operation::Update { default } => {
            let changes = ChangeDefault {
                id: default.id,
                order: MaybeSet::Set(order),
                regexp: MaybeSet::Set(regexp),
                name: MaybeSet::Set(name),
                action: MaybeSet::Set(action),
            };
            update_default(default.clone(), changes)
                .await
                .map_err(EditError::Server)
        }
    }
}

#[component]
pub fn DefaultUpdate(op: Operation, on_cancel: Callback, on_save: Callback<Default>) -> Element {
    let order = use_signal(|| match &op {
        Operation::Create => String::new(),
        Operation::Update { default } => default.order.as_raw(),
    });

    let regexp = use_signal(|| match &op {
        Operation::Create => String::new(),
        Operation::Update { default } => default.regexp.as_raw(),
    });

    let name = use_signal(|| match &op {
        Operation::Create => String::new(),
        Operation::Update { default } => default.name.as_raw(),
    });

    let action = use_signal(|| match &op {
        Operation::Create => String::new(),
        Operation::Update { default } => default.action.as_raw(),
    });

    let validate = Validate {
        order: use_memo(move || validate_optional_order(&order())),
        regexp: use_memo(move || validate_regex(&regexp())),
        name: use_memo(move || validate_default_name(&name())),
        action: use_memo(move || validate_optional_string(&action())),
    };

    let mut saving = use_signal(|| Saving::No);

    let disabled = use_memo(move || saving.read().is_saving());
    let disabled_save = use_memo(move || {
        validate.order.read().is_err()
            || validate.regexp.read().is_err()
            || validate.name.read().is_err()
            || validate.action.read().is_err()
            || disabled()
    });

    let op_clone = op.clone();
    let validate_clone = validate.clone();
    let on_save = use_callback(move |()| {
        let op = op_clone.clone();
        let validate = validate_clone.clone();
        spawn(async move {
            saving.set(Saving::Yes);

            let result = do_save(&op, &validate).await;

            match result {
                Ok(default) => {
                    saving.set(Saving::Finished(Ok(())));
                    on_save(default);
                }
                Err(err) => saving.set(Saving::Finished(Err(err))),
            }
        });
    });

    rsx! {
        h3 { class: "text-lg font-bold",
            match &op {
                Operation::Create => "Create Default".to_string(),
                Operation::Update { default } => format!("Edit Default {}", default.as_title()),
            }
        }
        p { class: "py-4", "Press ESC key or click the button below to close" }
        form {
            novalidate: true,
            action: "javascript:void(0)",
            method: "dialog",
            onkeyup: move |event| {
                if event.key() == Key::Escape {
                    on_cancel(());
                }
            },
            InputString {
                id: "order",
                label: "Order",
                value: order,
                validate: validate.order,
                disabled,
            }
            InputString {
                id: "regexp",
                label: "Regexp",
                value: regexp,
                validate: validate.regexp,
                disabled,
            }
            InputString {
                id: "name",
                label: "Name",
                value: name,
                validate: validate.name,
                disabled,
            }
            InputString {
                id: "action",
                label: "Action",
                value: action,
                validate: validate.action,
                disabled,
            }

            FormSaveCancelButton {
                disabled: disabled_save,
                on_save: move |()| on_save(()),
                on_cancel: move |()| on_cancel(()),
                title: match &op {
                    Operation::Create => "Create",
                    Operation::Update { .. } => "Save",
                },
                saving,
            }
        }
    }
}

#[component]
pub fn DefaultDelete(
    default: Default,
    on_cancel: Callback,
    on_delete: Callback<Default>,
) -> Element {
    let mut saving = use_signal(|| Saving::No);

    let disabled = use_memo(move || saving.read().is_saving());

    let default_clone = default.clone();
    let on_save = use_callback(move |()| {
        let default_clone = default_clone.clone();
        spawn(async move {
            saving.set(Saving::Yes);

            match delete_default(default_clone.clone()).await {
                Ok(_) => {
                    saving.set(Saving::Finished(Ok(())));
                    on_delete(default_clone.clone());
                }
                Err(err) => saving.set(Saving::Finished(Err(EditError::Server(err)))),
            }
        });
    });

    rsx! {
        h3 { class: "text-lg font-bold",
            "Delete default "
            {default.as_title()}
        }
        p { class: "py-4", "Press ESC key or click the button below to close" }
        DefaultSummary { default: default.clone() }
        form {
            novalidate: true,
            action: "javascript:void(0)",
            method: "dialog",
            onkeyup: move |event| {
                if event.key() == Key::Escape {
                    on_cancel(());
                }
            },
            FormSaveCancelButton {
                disabled,
                on_save: move |()| on_save(()),
                on_cancel: move |_| on_cancel(()),
                title: "Delete",
                saving,
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ActiveDialog {
    Change(Operation),
    Delete(Default),
    Idle,
}

#[derive(Error, Debug)]
pub enum ListDialogReferenceError {
    #[error("Invalid integer")]
    ParseIntError(#[from] ParseIntError),

    #[error("Invalid reference")]
    ReferenceError,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ListDialogReference {
    Create,
    Update {
        default_id: DefaultId,
    },
    Delete {
        default_id: DefaultId,
    },
    #[default]
    Idle,
}

impl ToQueryArgument for ListDialogReference {
    fn display_query_argument(
        &self,
        query_name: &str,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{}={}", query_name, self.to_string())
    }
}

impl FromStr for ListDialogReference {
    type Err = ListDialogReferenceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split = s.split("-").collect::<Vec<_>>();
        match split[..] {
            ["create"] => Self::Create,
            ["update", id] => {
                let default_id = DefaultId::new(id.parse()?);
                Self::Update { default_id }
            }
            ["delete", id] => {
                let default_id = DefaultId::new(id.parse()?);
                Self::Delete { default_id }
            }
            [""] | [] => Self::Idle,
            _ => return Err(ListDialogReferenceError::ReferenceError),
        }
        .pipe(Ok)
    }
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for ListDialogReference {
    fn to_string(&self) -> String {
        match self {
            ListDialogReference::Create => "create".to_string(),
            ListDialogReference::Update { default_id } => format!("update-{default_id}"),
            ListDialogReference::Delete { default_id } => format!("delete-{default_id}"),
            ListDialogReference::Idle => String::new(),
        }
    }
}

#[component]
pub fn DefaultDialog(
    dialog: ReadSignal<ActiveDialog>,
    on_change: Callback<Default>,
    on_delete: Callback<Default>,
    on_close: Callback<()>,
) -> Element {
    match dialog() {
        ActiveDialog::Idle => rsx! {},
        ActiveDialog::Change(op) => {
            rsx! {
                Dialog {
                    DefaultUpdate {
                        op,
                        on_cancel: on_close,
                        on_save: move |default: Default| {
                            on_change(default.clone());
                            on_close(());
                        },
                    }
                }
            }
        }
        ActiveDialog::Delete(default) => {
            rsx! {
                Dialog {
                    DefaultDelete {
                        default,
                        on_cancel: on_close,
                        on_delete: move |default| {
                            on_delete(default);
                            on_close(());
                        },
                    }
                }
            }
        }
    }
}

#[component]
pub fn DefaultSummary(default: Default) -> Element {
    rsx! {
        div {
            if let Some(order) = default.order {
                {order.to_string()}
            }
        }
        div {
            if let Some(regexp) = &default.regexp {
                {regexp.clone()}
            }
        }
        div {
            if let Some(name) = &default.name {
                {name.clone()}
            }
        }
        div {
            if let Some(action) = &default.action {
                {action.clone()}
            }
        }
    }
}

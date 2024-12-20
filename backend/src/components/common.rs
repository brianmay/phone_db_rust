#![allow(non_snake_case)]
use dioxus::{prelude::*, signals::Signal};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
#[error("{0}")]
pub struct ValidationError(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Add,
    Edit(i64),
}

#[derive(Debug, Clone, Copy)]
pub enum ActiveDialog {
    Editing(Operation),
    Deleting(i64),
    Idle,
}

pub enum Saving {
    No,
    Yes,
    Finished(Result<(), EditError>),
}

#[derive(Error, Debug)]
pub enum EditError {
    #[error("{0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("{0}")]
    Ldap(#[from] crate::ldap::Error),

    #[error("{0}")]
    Validation(#[from] ValidationError),
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
pub fn InputString<D: 'static + Clone + Eq + PartialEq>(
    id: &'static str,
    label: &'static str,
    validate: Memo<Result<D, ValidationError>>,
    changed: Signal<bool>,
    value: Signal<String>,
    disabled: bool,
) -> Element {
    rsx! {
        div {
            class: "form-group",
            label {
                for: id,
                "{label}"
            }
            input {
                type: "text",
                class: get_input_classes(validate().is_ok(), changed()),
                id: id,
                placeholder: "Enter input",
                value: value(),
                disabled: disabled,
                oninput: move |e| {
                    changed.set(true);
                    value.set(e.value());
                }
            }
            if let Err(err) = validate() {
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

#[component]
pub fn InputTextArea<D: 'static + Clone + Eq + PartialEq>(
    id: &'static str,
    label: &'static str,
    validate: Memo<Result<D, ValidationError>>,
    changed: Signal<bool>,
    value: Signal<String>,
    disabled: bool,
) -> Element {
    rsx! {
        div {
            class: "form-group",
            label {
                for: id,
                "{label}"
            }
            textarea {
                class: get_input_classes(validate().is_ok(), changed()),
                id: id,
                placeholder: "Enter input",
                value: value(),
                disabled: disabled,
                oninput: move |e| {
                    changed.set(true);
                    value.set(e.value());
                }
            }
            if let Err(err) = validate() {
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

#[component]
pub fn InputSelect<D: 'static + Clone + Eq + PartialEq>(
    id: &'static str,
    label: &'static str,
    validate: Memo<Result<D, ValidationError>>,
    changed: Signal<bool>,
    value: Signal<String>,
    disabled: bool,
    options: Vec<(&'static str, &'static str)>,
) -> Element {
    rsx! {
        div {
            class: "form-group",
            label {
                for: id,
                "{label}"
            }
            select {
                class: get_input_classes(validate().is_ok(), changed()),
                id: "input",
                disabled: disabled,
                oninput: move |e| {
                    changed.set(true);
                    value.set(e.value());
                },
                value: value(),
                option {
                    value: "",
                    label: "Select..."
                }
                for (label, value) in options {
                    option {
                        value: value,
                        label
                    }
                }
            }
            if let Err(err) = validate() {
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

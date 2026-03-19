#![allow(non_snake_case)]
use chrono::{DateTime, FixedOffset, Local, TimeDelta, Utc};
use classes::classes;
use dioxus::{prelude::*, signals::Signal};

use crate::components::buttons::ActionButton;

use super::FieldValue;
use super::errors::ValidationError;

fn get_label_classes() -> String {
    classes![
        "block",
        "mb-2",
        "text-sm",
        "font-medium",
        "text-gray-900",
        "dark:text-white"
    ]
}

fn get_checkbox_classes(is_valid: bool, is_disabled: bool) -> String {
    let classes = classes![
        "bg-gray-100",
        "checkbox",
        "dark:bg-gray-700",
        "dark:focus:ring-blue-500",
        "dark:ring-offset-gray-800",
        "dark:text-white",
        "focus:ring-2",
        "focus:ring-blue-500",
        "h-4",
        "ring-offset-2",
        "ring-offset-gray-100",
        "rounded",
        "text-gray-900",
        "w-4",
        "focus:outline-none"
    ];

    if is_disabled {
        return classes + " " + &classes!["border-gray-300", "dark:border-gray-600"];
    }

    if is_valid {
        return classes + " " + &classes!["border-green-500", "dark:border-green-500"];
    }

    classes + &classes!["border-red-500", "dark:border-red-500"]
}

fn get_input_classes(is_valid: bool, is_disabled: bool) -> String {
    let classes = classes![
        "bg-gray-50",
        "block",
        "border",
        "dark:bg-gray-700",
        "dark:focus:ring-blue-500",
        "dark:ring-offset-gray-800",
        "dark:placeholder-gray-400",
        "dark:text-white",
        "focus:ring-2",
        "focus:ring-blue-500",
        "p-2.5",
        "ring-offset-2",
        "ring-offset-gray-100",
        "rounded-lg",
        "text-gray-900",
        "w-full",
        "focus:outline-none"
    ];

    if is_disabled {
        return classes + " " + &classes!["border-gray-300", "dark:border-gray-600"];
    }

    if is_valid {
        return classes + " " + &classes!["border-green-500", "dark:border-green-500"];
    }

    classes + " " + &classes!["border-red-500", "dark:border-red-500"]
}

#[component]
pub fn FieldMessage<D: 'static + Clone + PartialEq>(
    validate: Memo<Result<D, ValidationError>>,
    disabled: Memo<bool>,
) -> Element {
    rsx! {
        if disabled() {
            div { class: "text-gray-300", "Inactive" }
        } else if let Err(err) = validate() {
            div { class: "text-red-500", "{err}" }
        } else {
            div { class: "text-green-500", "Looks good!" }
        }
    }
}

#[component]
pub fn InputString<D: 'static + Clone + PartialEq>(
    id: &'static str,
    label: &'static str,
    value: Signal<String>,
    validate: Memo<Result<D, ValidationError>>,
    disabled: Memo<bool>,
) -> Element {
    rsx! {
        div { class: "mb-5",
            label { r#for: id, class: get_label_classes(), "{label}" }
            input {
                r#type: "text",
                class: get_input_classes(validate().is_ok(), disabled()),
                id,
                placeholder: "Enter input",
                value: "{value()}",
                disabled,
                oninput: move |e| {
                    value.set(e.value());
                },
            }
            FieldMessage { validate, disabled }
        }
    }
}

#[component]
pub fn InputNumber<D: 'static + Clone + PartialEq>(
    id: &'static str,
    label: String,
    value: Signal<String>,
    validate: Memo<Result<D, ValidationError>>,
    disabled: Memo<bool>,
) -> Element {
    rsx! {
        div { class: "mb-5",
            label { r#for: id, class: get_label_classes(), "{label}" }
            input {
                r#type: "text",
                class: get_input_classes(validate().is_ok(), disabled()),
                id,
                r#type: "number",
                pattern: "[0-9]*",
                inputmode: "numeric",
                placeholder: "Enter input",
                value: "{value()}",
                disabled,
                oninput: move |e| {
                    value.set(e.value());
                },
            }
            FieldMessage { validate, disabled }
        }
    }
}

#[component]
pub fn InputSymptomIntensity(
    id: &'static str,
    label: &'static str,
    value: Signal<String>,
    validate: Memo<Result<i32, ValidationError>>,
    disabled: Memo<bool>,
) -> Element {
    rsx! {
        InputNumber {
            id,
            label: label.to_string() + " (0-10)",
            value,
            validate,
            disabled,
        }
    }
}

#[component]
pub fn InputPassword<D: 'static + Clone + Eq + PartialEq>(
    id: &'static str,
    label: &'static str,
    value: Signal<String>,
    validate: Memo<Result<D, ValidationError>>,
    disabled: Memo<bool>,
) -> Element {
    rsx! {
        div { class: "my-5",
            label { r#for: id, class: get_label_classes(), "{label}" }
            input {
                r#type: "password",
                class: get_input_classes(validate().is_ok(), disabled()),
                id,
                placeholder: "Enter input",
                value: value(),
                disabled,
                oninput: move |e| {
                    value.set(e.value());
                },
            }
            FieldMessage { validate, disabled }
        }
    }
}

#[component]
pub fn InputTextArea<D: 'static + Clone + Eq + PartialEq>(
    id: &'static str,
    label: &'static str,
    value: Signal<String>,
    validate: Memo<Result<D, ValidationError>>,
    disabled: Memo<bool>,
) -> Element {
    rsx! {
        div { class: "mb-5",
            label { r#for: id, class: get_label_classes(), "{label}" }
            textarea {
                class: get_input_classes(validate().is_ok(), disabled()),
                id,
                placeholder: "Enter input",
                value: "{value()}",
                disabled,
                oninput: move |e| {
                    value.set(e.value());
                },
            }
            FieldMessage { validate, disabled }
        }
    }
}

#[component]
pub fn InputDateTime(
    id: &'static str,
    label: &'static str,
    value: Signal<String>,
    validate: Memo<Result<DateTime<FixedOffset>, ValidationError>>,
    disabled: Memo<bool>,
) -> Element {
    rsx! {
        div { class: "mb-5",
            label { r#for: id, class: get_label_classes(), "{label}" }
            input {
                r#type: "text",
                class: get_input_classes(validate().is_ok(), disabled()),
                id,
                placeholder: "Enter input",
                value: "{value()}",
                disabled,
                oninput: move |e| {
                    value.set(e.value());
                },
            }
            ActionButton {
                on_click: move |_e| {
                    value.set(Local::now().to_rfc3339());
                },
                "Now"
            }
            FieldMessage { validate, disabled }
        }
    }
}

#[component]
pub fn InputOptionDateTimeUtc(
    id: &'static str,
    label: &'static str,
    value: Signal<String>,
    validate: Memo<Result<Option<DateTime<Utc>>, ValidationError>>,
    disabled: Memo<bool>,
) -> Element {
    rsx! {
        div { class: "mb-5",
            label { r#for: id, class: get_label_classes(), "{label}" }
            input {
                r#type: "text",
                class: get_input_classes(validate().is_ok(), disabled()),
                id,
                placeholder: "Enter input",
                value: "{value()}",
                disabled,
                oninput: move |e| {
                    value.set(e.value());
                },
            }
            ActionButton {
                on_click: move |_e| {
                    value.set(Local::now().to_rfc3339());
                },
                "Now"
            }
            FieldMessage { validate, disabled }
        }
    }
}

#[component]
pub fn InputDuration(
    id: &'static str,
    label: &'static str,
    value: Signal<String>,
    start_time: Memo<Result<DateTime<FixedOffset>, ValidationError>>,
    validate: Memo<Result<TimeDelta, ValidationError>>,
    disabled: Memo<bool>,
) -> Element {
    rsx! {
        div { class: "mb-5",
            label { r#for: id, class: get_label_classes(), "{label}" }
            input {
                r#type: "text",
                class: get_input_classes(validate().is_ok(), disabled()),
                id,
                r#type: "number",
                pattern: "[0-9]*",
                inputmode: "numeric",
                placeholder: "Enter input",
                value: "{value()}",
                disabled,
                oninput: move |e| {
                    value.set(e.value());
                },
            }
            if let Ok(start_time) = start_time() {
                ActionButton {
                    on_click: move |_e| {
                        let now: DateTime<FixedOffset> = Utc::now().into();
                        value.set((now - start_time).as_raw());
                    },
                    "Stop"
                }
            }
            FieldMessage { validate, disabled }
        }
    }
}

#[component]
pub fn InputBoolean(
    id: &'static str,
    label: &'static str,
    mut value: Signal<bool>,
    // validate: Memo<Result<D, ValidationError>>,
    disabled: Memo<bool>,
) -> Element {
    rsx! {
        div {
            label { r#for: id, class: get_label_classes(), "{label}" }
            input {
                r#type: "checkbox",
                class: get_checkbox_classes(true, disabled()),
                id,
                checked: value(),
                disabled,
                oninput: move |e| {
                    value.set(e.checked());
                },
            }
        }
    }
}

use std::{num::ParseIntError, str::FromStr};

use chrono::Local;
use dioxus::prelude::*;
use dioxus_router::ToQueryArgument;
use tap::Pipe;
use thiserror::Error;

use crate::{
    forms::{
        Dialog, EditError, FieldValue, FormSaveCancelButton, InputString, Saving, ValidationError,
        validate_action,
    },
    functions::phone_calls::{create_phone_call, delete_phone_call, update_phone_call},
    models::{
        common::MaybeSet,
        contacts::ContactId,
        phone_calls::{ChangePhoneCall, NewPhoneCall, PhoneCall, PhoneCallId},
    },
};

fn validate_contact_id(str: &str) -> Result<ContactId, ValidationError> {
    let str = str.trim();
    if str.is_empty() {
        return Err(ValidationError("Contact ID is required".to_string()));
    }
    str.parse::<i64>()
        .map(ContactId::new)
        .map_err(|_| ValidationError("Contact ID must be a number".to_string()))
}

fn validate_destination_number(str: &str) -> Result<Option<String>, ValidationError> {
    crate::forms::FieldValue::from_raw(str).map_err(|err| ValidationError(err.to_string()))
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Operation {
    Create,
    Update { phone_call: PhoneCall },
}

#[derive(Debug, Clone)]
struct Validate {
    action: Memo<Result<String, ValidationError>>,
    contact_id: Memo<Result<ContactId, ValidationError>>,
    destination_number: Memo<Result<Option<String>, ValidationError>>,
}

async fn do_save(op: &Operation, validate: &Validate) -> Result<PhoneCall, EditError> {
    let action = validate.action.read().clone()?;
    let contact_id = validate.contact_id.read().clone()?;
    let destination_number = validate.destination_number.read().clone()?;

    match op {
        Operation::Create => {
            let new_phone_call = NewPhoneCall {
                action,
                contact_id,
                destination_number,
            };
            create_phone_call(new_phone_call)
                .await
                .map_err(EditError::Server)
        }
        Operation::Update { phone_call } => {
            let changes = ChangePhoneCall {
                action: MaybeSet::Set(action),
                contact_id: MaybeSet::Set(contact_id),
                destination_number: MaybeSet::Set(destination_number),
            };
            update_phone_call(phone_call.clone(), changes)
                .await
                .map_err(EditError::Server)
        }
    }
}

#[component]
pub fn PhoneCallUpdate(
    op: Operation,
    on_cancel: Callback,
    on_save: Callback<PhoneCall>,
) -> Element {
    let action = use_signal(|| match &op {
        Operation::Create => String::new(),
        Operation::Update { phone_call } => phone_call.action.as_raw(),
    });

    let contact_id = use_signal(|| match &op {
        Operation::Create => String::new(),
        Operation::Update { phone_call } => phone_call.contact_id.to_string(),
    });

    let destination_number = use_signal(|| match &op {
        Operation::Create => String::new(),
        Operation::Update { phone_call } => phone_call.destination_number.as_raw(),
    });

    let validate = Validate {
        action: use_memo(move || validate_action(&action())),
        contact_id: use_memo(move || validate_contact_id(&contact_id())),
        destination_number: use_memo(move || validate_destination_number(&destination_number())),
    };

    let mut saving = use_signal(|| Saving::No);

    let disabled = use_memo(move || saving.read().is_saving());
    let disabled_save = use_memo(move || {
        validate.action.read().is_err()
            || validate.contact_id.read().is_err()
            || validate.destination_number.read().is_err()
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
                Ok(phone_call) => {
                    saving.set(Saving::Finished(Ok(())));
                    on_save(phone_call);
                }
                Err(err) => saving.set(Saving::Finished(Err(err))),
            }
        });
    });

    rsx! {
        h3 { class: "text-lg font-bold",
            match &op {
                Operation::Create => "Create Phone Call".to_string(),
                Operation::Update { phone_call } => format!("Edit Phone Call {}", phone_call.as_title()),
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
                id: "action",
                label: "Action",
                value: action,
                validate: validate.action,
                disabled,
            }
            InputString {
                id: "contact_id",
                label: "Contact ID",
                value: contact_id,
                validate: validate.contact_id,
                disabled,
            }
            InputString {
                id: "destination_number",
                label: "Destination Number",
                value: destination_number,
                validate: validate.destination_number,
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
pub fn PhoneCallDelete(
    phone_call: PhoneCall,
    on_cancel: Callback,
    on_delete: Callback<PhoneCall>,
) -> Element {
    let mut saving = use_signal(|| Saving::No);

    let disabled = use_memo(move || saving.read().is_saving());

    let phone_call_clone = phone_call.clone();
    let on_save = use_callback(move |()| {
        let phone_call_clone = phone_call_clone.clone();
        spawn(async move {
            saving.set(Saving::Yes);

            match delete_phone_call(phone_call_clone.clone()).await {
                Ok(_) => {
                    saving.set(Saving::Finished(Ok(())));
                    on_delete(phone_call_clone.clone());
                }
                Err(err) => saving.set(Saving::Finished(Err(EditError::Server(err)))),
            }
        });
    });

    rsx! {
        h3 { class: "text-lg font-bold",
            "Delete phone call "
            {phone_call.as_title()}
        }
        p { class: "py-4", "Press ESC key or click the button below to close" }
        PhoneCallSummary { phone_call: phone_call.clone() }
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
    Delete(PhoneCall),
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
        phone_call_id: PhoneCallId,
    },
    Delete {
        phone_call_id: PhoneCallId,
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
                let phone_call_id = PhoneCallId::new(id.parse()?);
                Self::Update { phone_call_id }
            }
            ["delete", id] => {
                let phone_call_id = PhoneCallId::new(id.parse()?);
                Self::Delete { phone_call_id }
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
            ListDialogReference::Update { phone_call_id } => format!("update-{phone_call_id}"),
            ListDialogReference::Delete { phone_call_id } => format!("delete-{phone_call_id}"),
            ListDialogReference::Idle => String::new(),
        }
    }
}

#[component]
pub fn PhoneCallDialog(
    dialog: ReadSignal<ActiveDialog>,
    on_change: Callback<PhoneCall>,
    on_delete: Callback<PhoneCall>,
    on_close: Callback<()>,
) -> Element {
    match dialog() {
        ActiveDialog::Idle => rsx! {},
        ActiveDialog::Change(op) => {
            rsx! {
                Dialog {
                    PhoneCallUpdate {
                        op,
                        on_cancel: on_close,
                        on_save: move |phone_call: PhoneCall| {
                            on_change(phone_call.clone());
                            on_close(());
                        },
                    }
                }
            }
        }
        ActiveDialog::Delete(phone_call) => {
            rsx! {
                Dialog {
                    PhoneCallDelete {
                        phone_call,
                        on_cancel: on_close,
                        on_delete: move |phone_call| {
                            on_delete(phone_call);
                            on_close(());
                        },
                    }
                }
            }
        }
    }
}

#[component]
pub fn PhoneCallSummary(phone_call: PhoneCall) -> Element {
    rsx! {
        div { {phone_call.action.clone()} }
        div { {phone_call.contact_id.to_string()} }
        div {
            if let Some(dest) = &phone_call.destination_number {
                {dest.clone()}
            }
        }
        div { {phone_call.inserted_at.with_timezone(&Local).to_string()} }
    }
}

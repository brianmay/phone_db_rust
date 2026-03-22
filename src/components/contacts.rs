use std::{num::ParseIntError, str::FromStr};

use dioxus::prelude::*;
use dioxus_router::ToQueryArgument;
use tap::Pipe;
use thiserror::Error;

use crate::{
    components::Markdown,
    forms::{
        Dialog, EditError, FieldValue, FormSaveCancelButton, InputString, InputTextArea, Saving,
        ValidationError, validate_action, validate_comments, validate_contact_name,
        validate_phone_number,
    },
    functions::contacts::{create_contact, delete_contact, update_contact},
    models::{
        common::MaybeSet,
        contacts::{ChangeContact, Contact, ContactId, NewContact},
    },
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Operation {
    Create,
    Update { contact: Contact },
}

#[derive(Debug, Clone)]
struct Validate {
    phone_number: Memo<Result<String, ValidationError>>,
    name: Memo<Result<Option<String>, ValidationError>>,
    action: Memo<Result<String, ValidationError>>,
    comments: Memo<Result<Option<String>, ValidationError>>,
}

async fn do_save(op: &Operation, validate: &Validate) -> Result<Contact, EditError> {
    let phone_number = validate.phone_number.read().clone()?;
    let name = validate.name.read().clone()?;
    let action = validate.action.read().clone()?;
    let comments = validate.comments.read().clone()?;

    match op {
        Operation::Create => {
            let updates = NewContact {
                phone_number,
                name,
                action,
                comments,
            };
            create_contact(updates).await.map_err(EditError::Server)
        }
        Operation::Update { contact } => {
            let changes = ChangeContact {
                phone_number: MaybeSet::Set(phone_number),
                name: MaybeSet::Set(name),
                action: MaybeSet::Set(action),
                comments: MaybeSet::Set(comments),
            };
            update_contact(contact.clone(), changes)
                .await
                .map_err(EditError::Server)
        }
    }
}

#[component]
pub fn ContactUpdate(op: Operation, on_cancel: Callback, on_save: Callback<Contact>) -> Element {
    let phone_number = use_signal(|| match &op {
        Operation::Create => String::new(),
        Operation::Update { contact } => contact.phone_number.as_raw(),
    });

    let name = use_signal(|| match &op {
        Operation::Create => String::new(),
        Operation::Update { contact } => contact.name.as_raw(),
    });

    let action = use_signal(|| match &op {
        Operation::Create => String::new(),
        Operation::Update { contact } => contact.action.clone(),
    });

    let comments = use_signal(|| match &op {
        Operation::Create => String::new(),
        Operation::Update { contact } => contact.comments.as_raw(),
    });

    let validate = Validate {
        phone_number: use_memo(move || validate_phone_number(&phone_number())),
        name: use_memo(move || validate_contact_name(&name())),
        action: use_memo(move || validate_action(&action())),
        comments: use_memo(move || validate_comments(&comments())),
    };

    let mut saving = use_signal(|| Saving::No);

    // disable form while waiting for response
    let disabled = use_memo(move || saving.read().is_saving());
    let disabled_save = use_memo(move || {
        validate.phone_number.read().is_err()
            || validate.name.read().is_err()
            || validate.action.read().is_err()
            || validate.comments.read().is_err()
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
                Ok(contact) => {
                    saving.set(Saving::Finished(Ok(())));
                    on_save(contact);
                }
                Err(err) => saving.set(Saving::Finished(Err(err))),
            }
        });
    });

    rsx! {

        h3 { class: "text-lg font-bold",
            match &op {
                Operation::Create => "Create Contact".to_string(),
                Operation::Update { contact } => format!("Edit Contact {}", contact.as_title()),
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
                id: "phone_number",
                label: "Phone Number",
                value: phone_number,
                validate: validate.phone_number,
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
            InputTextArea {
                id: "comments",
                label: "Comments",
                value: comments,
                validate: validate.comments,
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
pub fn ContactDelete(
    contact: Contact,
    on_cancel: Callback,
    on_delete: Callback<Contact>,
) -> Element {
    let mut saving = use_signal(|| Saving::No);

    let disabled = use_memo(move || saving.read().is_saving());

    let contact_clone = contact.clone();
    let on_save = use_callback(move |()| {
        let contact_clone = contact_clone.clone();
        spawn(async move {
            saving.set(Saving::Yes);

            match delete_contact(contact_clone.clone()).await {
                Ok(_) => {
                    saving.set(Saving::Finished(Ok(())));
                    on_delete(contact_clone.clone());
                }
                Err(err) => saving.set(Saving::Finished(Err(EditError::Server(err)))),
            }
        });
    });

    rsx! {
        h3 { class: "text-lg font-bold",
            "Delete contact "
            {contact.name.clone()}
        }
        p { class: "py-4", "Press ESC key or click the button below to close" }
        ContactSummary { contact: contact.clone() }
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
    Delete(Contact),
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
        contact_id: ContactId,
    },
    Delete {
        contact_id: ContactId,
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
                let contact_id = ContactId::new(id.parse()?);
                Self::Update { contact_id }
            }
            ["delete", id] => {
                let contact_id = ContactId::new(id.parse()?);
                Self::Delete { contact_id }
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
            ListDialogReference::Update { contact_id } => format!("update-{contact_id}"),
            ListDialogReference::Delete { contact_id } => format!("delete-{contact_id}"),
            ListDialogReference::Idle => String::new(),
        }
    }
}

#[component]
pub fn ContactDialog(
    dialog: ReadSignal<ActiveDialog>,
    on_change: Callback<Contact>,
    on_delete: Callback<Contact>,

    on_close: Callback<()>,
) -> Element {
    match dialog() {
        ActiveDialog::Idle => rsx! {},
        ActiveDialog::Change(op) => {
            rsx! {
                Dialog {
                    ContactUpdate {
                        op,
                        on_cancel: on_close,
                        on_save: move |contact: Contact| {
                            on_change(contact.clone());
                            on_close(());
                        },
                    }
                }
            }
        }
        ActiveDialog::Delete(contact) => {
            rsx! {
                Dialog {
                    ContactDelete {
                        contact,
                        on_cancel: on_close,
                        on_delete: move |contact| {
                            on_delete(contact);
                            on_close(());
                        },
                    }
                }
            }
        }
    }
}

#[component]
pub fn ContactSummary(contact: Contact) -> Element {
    rsx! {
        div { {contact.phone_number.clone()} }
        div {
            if let Some(name) = &contact.name {
                {name.clone()}
            }
        }
        div { {contact.action.clone()} }
        div {
            if let Some(comments) = &contact.comments {
                Markdown { content: comments.to_string() }
            }
        }

    }
}

use std::ops::Deref;

use dioxus::prelude::*;
use dioxus_fullstack::ServerFnError;
use dioxus_router::navigator;
use tap::Pipe;

use crate::{
    Route,
    components::{
        buttons::{ChangeButton, CreateButton},
        defaults::{ActiveDialog, DefaultDialog, ListDialogReference, Operation},
    },
    functions::defaults::{get_default_by_id, search_defaults},
    models::defaults::{Default, DefaultId},
    use_user,
};

#[component]
fn EntryRow(default: Default, selected: Signal<Option<DefaultId>>) -> Element {
    let id = default.id;

    let navigator = navigator();
    rsx! {
        tr {
            class: "hover:bg-gray-500 border-blue-300 mt-2 mb-2 p-2 border-2 w-full sm:w-auto sm:border-none inline-block sm:table-row",
            onclick: move |_| { selected.set(Some(id)) },
            td { class: "block sm:table-cell border-blue-300 sm:border-t-2",
                if let Some(order) = default.order {
                    {order.to_string()}
                }
            }
            td { class: "block sm:table-cell border-blue-300 sm:border-t-2",
                if let Some(regexp) = &default.regexp {
                    {regexp.clone()}
                }
            }
            td { class: "block sm:table-cell border-blue-300 sm:border-t-2",
                if let Some(name) = &default.name {
                    {name.clone()}
                }
            }
            td { class: "block sm:table-cell border-blue-300 sm:border-t-2",
                span { class: "sm:hidden", "Action: " }
                if let Some(action) = &default.action {
                    {action.clone()}
                }
            }
        }

        if selected() == Some(id) {
            tr {
                td { colspan: "4", class: "block sm:table-cell",
                    div { class: "flex gap-2",
                        ChangeButton {
                            on_click: move |_| {
                                navigator
                                    .push(Route::DefaultList {
                                        dialog: ListDialogReference::Update { default_id: id },
                                    });
                            },
                            "Edit"
                        }
                        ChangeButton {
                            on_click: move |_| {
                                navigator
                                    .push(Route::DefaultList {
                                        dialog: ListDialogReference::Delete { default_id: id },
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
pub fn DefaultList(dialog: ReadSignal<Option<ListDialogReference>>) -> Element {
    let user = use_user().ok().flatten();

    let Some(_user) = user.as_ref() else {
        return rsx! {
            p { class: "alert alert-error", "You are not logged in." }
        };
    };

    let selected: Signal<Option<DefaultId>> = use_signal(|| None);

    let mut query = use_signal(|| "".to_string());

    let dialog: Resource<Result<ActiveDialog, ServerFnError>> = use_resource(move || async move {
        let Some(dialog) = dialog() else {
            return Ok(ActiveDialog::Idle);
        };
        match dialog {
            ListDialogReference::Create => ActiveDialog::Change(Operation::Create).pipe(Ok),
            ListDialogReference::Update { default_id } => {
                let default = get_default_by_id(default_id)
                    .await?
                    .ok_or(ServerFnError::new("Cannot find default"))?;
                ActiveDialog::Change(Operation::Update { default }).pipe(Ok)
            }
            ListDialogReference::Delete { default_id } => {
                let default = get_default_by_id(default_id)
                    .await?
                    .ok_or(ServerFnError::new("Cannot find default"))?;
                ActiveDialog::Delete(default).pipe(Ok)
            }
            ListDialogReference::Idle => Ok(ActiveDialog::Idle),
        }
    });

    let navigator = navigator();
    let mut list: Resource<Result<Vec<Default>, ServerFnError>> =
        use_resource(move || async move { search_defaults(query()).await });

    rsx! {
        div { class: "ml-2 mr-2",
            div { class: "mb-2",
                CreateButton {
                    on_click: move |_| {
                        navigator
                            .push(Route::DefaultList {
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
                    "Error loading defaults: "
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
                                th { "Order" }
                                th { "Regexp" }
                                th { "Name" }
                                th { "Action" }
                            }
                        }
                        tbody { class: "block sm:table-row-group",
                            for default in list.iter() {
                                EntryRow {
                                    default: default.clone(),
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
                DefaultDialog {
                    dialog: dialog.clone(),
                    on_change: move |_default: Default| { list.restart() },
                    on_delete: move |_default| list.restart(),
                    on_close: move |()| {
                        navigator
                            .push(Route::DefaultList {
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

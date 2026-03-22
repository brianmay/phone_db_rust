use std::sync::Arc;

use dioxus::prelude::*;

use components::contacts::ListDialogReference;
use components::{navbar::Navbar, users};
use dioxus_fullstack::{ServerFnError, use_server_future};
use dioxus_router::{Routable, Router};
use models::users::{User, UserId};
use views::{
    ContactList, DefaultList, Home, Login, Logout, PhoneCallList, UserDetail, UserList, get_user,
};

mod components;
mod dt;
mod forms;
mod functions;
mod models;
mod version;
mod views;

#[cfg(feature = "server")]
mod server;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/login")]
    Login {},
    #[route("/logout")]
    Logout {},
    #[layout(Navbar)]
    #[route("/")]
    Home {  },
    #[route("/users?:dialog")]
    UserList { dialog: users::ListDialogReference },
    #[route("/users/:user_id?:dialog")]
    UserDetail { user_id: UserId, dialog: users::DetailsDialogReference },
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
    #[route("/contacts?:dialog")]
    ContactList { dialog: ListDialogReference },
    #[route("/phone_calls")]
    PhoneCallList {  },
    #[route("/defaults?:dialog")]
    DefaultList { dialog: components::defaults::ListDialogReference },
}

const FAVICON_SVG: Asset = asset!("/assets/favicon.svg");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[cfg(not(feature = "server"))]
fn main() {
    dioxus::launch(App);
}

#[cfg(feature = "server")]
fn main() {
    server::init(App);
}

fn reload_user() {
    let mut user_resource: Resource<Result<Option<Arc<User>>, ServerFnError>> = use_context();
    user_resource.restart();
}

fn use_user() -> Result<Option<Arc<User>>, ServerFnError> {
    let user_resource: Resource<Result<Option<Arc<User>>, ServerFnError>> = use_context();
    let user_result: &Option<Result<Option<Arc<User>>, ServerFnError>> = &user_resource.read();

    user_result.as_ref().map_or_else(
        || Err(ServerFnError::new("Mo user resource")),
        |x| x.clone(),
    )
}

#[component]
fn App() -> Element {
    let user_resource = use_server_future(move || async move {
        let data = get_user().await;
        data.map(|x| x.map(Arc::new))
    })?;

    use_context_provider(|| user_resource);

    rsx! {
        // Global app resources
        document::Link { rel: "icon", href: FAVICON_SVG }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        Router::<Route> {}
    }
}

#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
    let segments = segments.join("/");
    rsx! {
        div {
            main { role: "main", class: "container",
                h1 { "404 Not Found" }
                p { "The page you are looking for does not exist." }
                p { "Segments: {segments}" }
                p { "Please ask a friendly penguin for help." }
            }
        }
    }
}

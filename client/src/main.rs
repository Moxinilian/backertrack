#![recursion_limit="128"]

#[macro_use]
extern crate yew;

#[macro_use]
extern crate serde_derive;

mod auth;
mod model;
mod navbar;

fn main() {
    yew::initialize();
    yew::app::App::<model::Model>::new().mount_to_body();
    yew::run_loop();
}

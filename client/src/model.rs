use yew::prelude::*;

use crate::auth::AuthState;

pub enum ModelMsg {
    SetAuth(AuthState),
}

pub struct State {
    auth: AuthState,
}

impl Default for State {
    fn default() -> Self {
        Self {
            auth: AuthState::Disconnected,
        }
    }
}

pub struct Model {
    link: ComponentLink<Self>,
    state: State,
}

impl Component for Model {
    type Message = ModelMsg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            state: Default::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        true
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        html! {
            <crate::navbar::Navbar: />
        }
    }
}

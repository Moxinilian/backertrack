use yew::prelude::*;

pub enum NavbarMsg {
    BurgerToggle,
    SetAuth,
}

pub struct Navbar {
    burger_toggled: bool,
}

impl Component for Navbar {
    type Message = NavbarMsg;
    type Properties = ();

    fn create(_: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {
            burger_toggled: false,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            NavbarMsg::BurgerToggle => {
                self.burger_toggled = !self.burger_toggled;
                true
            },
            
            _ => false,
        }
    }
}

impl Navbar {
    fn button_class(&self) -> &'static str {
        if self.burger_toggled {
            "navbar-burger burger is-active"
        } else {
            "navbar-burger burger"
        }
    }

    fn menu_class(&self) -> &'static str {
        if self.burger_toggled {
            "navbar-menu is-active"
        } else {
            "navbar-menu"
        }
    }
}

impl Renderable<Navbar> for Navbar {
    fn view(&self) -> Html<Self> {
        html! {
            <nav class="navbar", role="navigation", aria-label="main navigation",>
                <div class="navbar-brand",>
                    <a class="navbar-item", href="/",>
                        <strong>{ "backertrack" }</strong>
                    </a>

                    <a role="button", class=self.button_class(), onclick=|_| NavbarMsg::BurgerToggle, aria-label="menu", aria-expanded="false", data-target="navbarBasicExample",>
                        <span aria-hidden="true",></span>
                        <span aria-hidden="true",></span>
                        <span aria-hidden="true",></span>
                    </a>
                </div>

                <div class=self.menu_class(),>
                    <div class="navbar-start",>

                    </div>

                    <div class="navbar-end",>
                        <div class="navbar-item",>
                            <div class="buttons",>
                                <a class="button is-primary",>
                                    <strong>{ "Sign up" }</strong>
                                </a>
                                <a class="button is-light",>
                                    { "Log in" }
                                </a>
                            </div>
                        </div>
                    </div>
                </div>
            </nav>
        }
    }
}
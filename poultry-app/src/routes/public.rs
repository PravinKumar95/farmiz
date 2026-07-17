use dioxus::prelude::*;

use crate::layouts::PublicLayout;
use crate::screens::{login::LoginInfo, login::SignIn, login::SignUp};

#[derive(Clone, Debug, PartialEq, Routable)]
pub enum PublicRoute {
    #[layout(PublicLayout)]
    #[route("/signin")]
    SignIn { on_login: Callback<LoginInfo> },
    #[route("/signup")]
    SignUp,
}

use dioxus::prelude::*;
fn main() {
    println!("{}", asset!("/assets/main.css").to_string());
}

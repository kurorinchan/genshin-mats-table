use leptos::*;

mod component;
mod logic;

fn main() {
    mount_to_body(|| view! { <component::App /> });
}

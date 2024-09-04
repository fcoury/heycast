mod chat;

use chat::*;
use leptos::*;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        view! {
            <ChatApp/>
        }
    })
}

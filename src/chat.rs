use leptos::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use web_sys::{RequestInit, RequestMode};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct ChatMessage {
    id: usize,
    content: String,
    is_user: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct ChatHistory {
    id: usize,
    title: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ClaudeRequest {
    prompt: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ClaudeResponse {
    completion: String,
}

async fn call_claude_api(prompt: String) -> Result<String, String> {
    let window = web_sys::window().unwrap();
    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);

    let request_body = ClaudeRequest { prompt };
    let body_str = serde_json::to_string(&request_body).map_err(|e| e.to_string())?;
    opts.body(Some(&JSON::parse(&body_str).map_err(|e| e.to_string())?));

    let request = web_sys::Request::new_with_str_and_init(
        "https://api.anthropic.com/v1/completions",
        &opts,
    )
    .map_err(|e| e.to_string())?;

    request.headers().set("Content-Type", "application/json").map_err(|e| e.to_string())?;
    request.headers().set("X-API-Key", "YOUR_API_KEY_HERE").map_err(|e| e.to_string())?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("fetch error: {:?}", e))?;

    let resp: web_sys::Response = resp_value.dyn_into().unwrap();

    let json = JsFuture::from(resp.json().map_err(|e| e.to_string())?)
        .await
        .map_err(|e| format!("json parse error: {:?}", e))?;

    let claude_response: ClaudeResponse = json.into_serde().map_err(|e| e.to_string())?;

    Ok(claude_response.completion)
}

#[component]
pub fn ChatApp() -> impl IntoView {
    let (chat_history, _set_chat_history) = create_signal(vec![
        ChatHistory {
            id: 1,
            title: "Chat 1".to_string(),
        },
        ChatHistory {
            id: 2,
            title: "Chat 2".to_string(),
        },
        ChatHistory {
            id: 3,
            title: "Chat 3".to_string(),
        },
    ]);

    let (current_chat, set_current_chat) = create_signal(Vec::new());
    let (input_text, set_input_text) = create_signal(String::new());
    let (is_streaming, set_is_streaming) = create_signal(false);
    let (dark_mode, set_dark_mode) = create_signal(false);
    let (error_message, set_error_message) = create_signal(String::new());

    let handle_send_message = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        let input = input_text.get();
        if input.trim().is_empty() {
            return;
        }

        let new_user_message = ChatMessage {
            id: current_chat.with(|chat| chat.len()),
            content: input.clone(),
            is_user: true,
        };

        set_current_chat.update(|chat| chat.push(new_user_message));
        set_input_text.set(String::new());

        // Call Claude API
        set_is_streaming.set(true);
        spawn_local(async move {
            match call_claude_api(input).await {
                Ok(response) => {
                    let new_llm_message = ChatMessage {
                        id: current_chat.with(|chat| chat.len()),
                        content: response,
                        is_user: false,
                    };
                    set_current_chat.update(|chat| chat.push(new_llm_message));
                    set_is_streaming.set(false);
                    set_error_message.set(String::new());
                }
                Err(err) => {
                    set_is_streaming.set(false);
                    set_error_message.set(format!("Error: {}", err));
                }
            }
        });
    };

    let toggle_dark_mode = move |_| set_dark_mode.update(|dm| *dm = !*dm);

    view! {
        <div class=move || format!("flex h-screen {}", if dark_mode.get() { "bg-gray-900 text-white" } else { "bg-gray-100" })>
            // Left Sidebar
            <div class=move || format!("w-64 {} p-4", if dark_mode.get() { "bg-gray-800 text-white" } else { "bg-gray-200 text-black" })>
                <h2 class="text-xl font-bold mb-4">"Chat History"</h2>
                <ul>
                    {move || chat_history.get().into_iter().map(|chat| view! {
                        <li key={chat.id} class=move || format!("mb-2 cursor-pointer {} p-2 rounded",
                            if dark_mode.get() { "hover:bg-gray-700" } else { "hover:bg-gray-300" })>
                            {chat.title}
                        </li>
                    }).collect::<Vec<_>>()}
                </ul>
                <button
                    class=move || format!("mt-4 px-4 py-2 rounded {}",
                        if dark_mode.get() { "bg-gray-600 hover:bg-gray-500" } else { "bg-gray-300 hover:bg-gray-400" })
                    on:click=toggle_dark_mode
                >
                    {move || if dark_mode.get() { "Light Mode" } else { "Dark Mode" }}
                </button>
            </div>

            // Main Chat Area
            <div class="flex-1 flex flex-col">
                // Chat Messages
                <div class="flex-1 p-4 overflow-y-auto">
                    {move || current_chat.get().into_iter().map(|message| view! {
                        <div class={format!("mb-4 {}", if message.is_user { "text-right" } else { "text-left" })}>
                            <div class=move || format!("inline-block p-2 rounded-lg {}",
                                if message.is_user {
                                    if dark_mode.get() { "bg-blue-600 text-white" } else { "bg-blue-500 text-white" }
                                } else {
                                    if dark_mode.get() { "bg-gray-700 text-white" } else { "bg-gray-300 text-black" }
                                })>
                                {message.content}
                            </div>
                        </div>
                    }).collect::<Vec<_>>()}
                    {move || is_streaming.get().then(|| view! {
                        <div class="text-left">
                            <div class=move || format!("inline-block p-2 rounded-lg {}",
                                if dark_mode.get() { "bg-gray-700" } else { "bg-gray-300" })>
                                <span class="animate-pulse">"..."</span>
                            </div>
                        </div>
                    })}
                    {move || {
                        let error = error_message.get();
                        if !error.is_empty() {
                            view! {
                                <div class="text-red-500 mt-2">
                                    {error}
                                </div>
                            }
                        } else {
                            view! { <div></div> }
                        }
                    }}
                </div>

                // Input Area
                <div class=move || format!("p-4 {}", if dark_mode.get() { "bg-gray-800" } else { "bg-white" })>
                    <form on:submit=handle_send_message class="flex items-center">
                        <textarea
                            class=move || format!("flex-1 border rounded-l-lg p-2 focus:outline-none focus:ring-2 focus:ring-blue-500 {}",
                                if dark_mode.get() { "bg-gray-700 text-white" } else { "bg-white text-black" })
                            placeholder="Type your message..."
                            prop:value=move || input_text.get()
                            on:input=move |ev| set_input_text.set(event_target_value(&ev))
                        />
                        <button
                            type="submit"
                            class="bg-blue-500 text-white p-2 rounded-r-lg hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500"
                        >
                            "Send"
                        </button>
                    </form>
                </div>
            </div>
        </div>
    }
}

#![allow(non_snake_case, unused)]
extern crate image_base64_wasm;

use crate::data::{Message, Role, SelectOption};
use crate::web_state::{Session, Store, TempSession};
use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};
use futures::StreamExt;
use js_sys::Reflect;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::window;

#[component]
fn Pulse() -> Element {
    rsx!(
        div { class: "border w-80 h-full border-blue-300 shadow rounded-md p-4 max-w-sm mx-auto",
            div { class: "animate-pulse flex space-x-4",
                div { class: "flex-1 space-y-6 py-1",
                    div { class: "h-2 bg-slate-200 rounded" }
                    div { class: "space-y-3",
                        div { class: "grid grid-cols-3 gap-4",
                            div { class: "h-2 bg-slate-200 rounded col-span-2" }
                            div { class: "h-2 bg-slate-200 rounded col-span-1" }
                        }
                        div { class: "h-2 bg-slate-200 rounded" }
                    }
                }
            }
        }
    )
}

#[component]
fn ModelConfig(
    model_id: Signal<String>,
    endpoint: Signal<String>,
    modelOptions: Signal<Vec<SelectOption>>,
) -> Element {
    rsx!(
        div {
            "aria-hidden": "true",
            tabindex: "-1",
            class: "hidden overflow-y-auto overflow-x-hidden fixed top-0 right-0 left-0 z-50 justify-center items-center w-full md:inset-0 h-[calc(100%-1rem)] max-h-full",
            id: "model-config",
            div { class: "relative p-4 w-full max-w-md max-h-full",
                div { class: "relative bg-white rounded-lg shadow dark:bg-gray-700",
                    div { class: "flex items-center justify-between p-4 md:p-5 border-b rounded-t dark:border-gray-600",
                        h3 { class: "text-lg font-semibold text-gray-900 dark:text-white",
                            "\n                    Model Setting\n                "
                        }
                        button {
                            "data-modal-toggle": "model-config",
                            r#type: "button",
                            class: "text-gray-400 bg-transparent hover:bg-gray-200 hover:text-gray-900 rounded-lg text-sm w-8 h-8 ms-auto inline-flex justify-center items-center dark:hover:bg-gray-600 dark:hover:text-white",
                            svg {
                                "xmlns": "http://www.w3.org/2000/svg",
                                "fill": "none",
                                "aria-hidden": "true",
                                "viewBox": "0 0 14 14",
                                class: "w-3 h-3",
                                path {
                                    "d": "m1 1 6 6m0 0 6 6M7 7l6-6M7 7l-6 6",
                                    "stroke": "currentColor",
                                    "stroke-linejoin": "round",
                                    "stroke-linecap": "round",
                                    "stroke-width": "2"
                                }
                            }
                            span { class: "sr-only", "Close modal" }
                        }
                    }
                    form { class: "p-4 md:p-5",
                        div { class: "grid gap-4 mb-4 grid-cols-2",
                            div { class: "col-span-2",
                                label {
                                    r#for: "model",
                                    class: "block mb-2 text-sm font-medium text-gray-900 dark:text-white",
                                    "Model"
                                }
                                select {
                                    class: "bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-primary-500 focus:border-primary-500 block w-full p-2.5 dark:bg-gray-600 dark:border-gray-500 dark:placeholder-gray-400 dark:text-white dark:focus:ring-primary-500 dark:focus:border-primary-500",
                                    id: "model",
                                    value: "{model_id}",
                                    onchange: move |event| {
                                        let value = event.value();
                                        model_id.set(value);
                                    },
                                    option { "Select model" }
                                    for model in modelOptions() {
                                    option {
                                        value: "{model.value}",
                                        selected: {model.selected},
                                        "{model.text}"
                                    }
                                   }
                                }
                            }
                            div { class: "col-span-2",
                                label {
                                    r#for: "url",
                                    class: "block mb-2 text-sm font-medium text-gray-900 dark:text-white",
                                    "Backend URL"
                                }
                                input {
                                    r#type: "text",
                                    step: "0.1",
                                    required: "",
                                    placeholder: "Backend URL",
                                    name: "url",
                                    class: "bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-primary-600 focus:border-primary-600 block w-full p-2.5 dark:bg-gray-600 dark:border-gray-500 dark:placeholder-gray-400 dark:text-white dark:focus:ring-primary-500 dark:focus:border-primary-500",
                                    id: "url",
                                    value:"{endpoint}",
                                    onchange: move |event| {
                                        let value = event.value();
                                        endpoint.set(value);
                                    },
                                }
                            }
                            div { class: "col-span-2",
                                label {
                                    r#for: "temp",
                                    class: "block mb-2 text-sm font-medium text-gray-900 dark:text-white",
                                    "Temperature"
                                }
                                input {
                                    r#type: "number",
                                    step: "0.1",
                                    required: "",
                                    min: "0",
                                    placeholder: "Temperature",
                                    name: "temp",
                                    max: "1",
                                    class: "bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-primary-600 focus:border-primary-600 block w-full p-2.5 dark:bg-gray-600 dark:border-gray-500 dark:placeholder-gray-400 dark:text-white dark:focus:ring-primary-500 dark:focus:border-primary-500",
                                    id: "temp"
                                }
                            }
                            div { class: "col-span-2",
                                label {
                                    r#for: "top_p",
                                    class: "block mb-2 text-sm font-medium text-gray-900 dark:text-white",
                                    "Top P"
                                }
                                input {
                                    required: "",
                                    min: "0",
                                    max: "1",
                                    r#type: "number",
                                    placeholder: "Top P",
                                    name: "top_p",
                                    step: "0.1",
                                    class: "bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-primary-600 focus:border-primary-600 block w-full p-2.5 dark:bg-gray-600 dark:border-gray-500 dark:placeholder-gray-400 dark:text-white dark:focus:ring-primary-500 dark:focus:border-primary-500",
                                    id: "top_p"
                                }
                            }
                            div { class: "col-span-2",
                                label {
                                    r#for: "description",
                                    class: "block mb-2 text-sm font-medium text-gray-900 dark:text-white",
                                    "System prompt"
                                }
                                textarea {
                                    rows: "4",
                                    placeholder: "Write System Prompt",
                                    class: "block p-2.5 w-full text-sm text-gray-900 bg-gray-50 rounded-lg border border-gray-300 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-600 dark:border-gray-500 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500",
                                    id: "description"
                                }
                            }
                        }
                        button {
                            "data-modal-toggle": "model-config",
                            r#type: "button",
                            class: "text-white inline-flex items-center bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800",
                            "\n                   OK\n                "
                        }
                    }
                }
            }
        }
    )
}

#[component]
fn ShowMessage(msg: Message) -> Element {
    use comrak::{markdown_to_html, ExtensionOptions, Options};
    let mut options = Options::default();
    options.extension.table = true;
    options.extension.math_code = true;
    options.extension.multiline_block_quotes = true;
    let html = markdown_to_html(msg.content.as_str(), &options);
    rsx!(if msg.role == Role::User {
           div { class: "flex justify-end mb-4",
                div { class: "bg-blue-500 text-white p-3 rounded-l-lg rounded-br-lg",
                  p { dangerous_inner_html: "{html}" }
                  if let Some(img) = msg.img.clone() {
                     img { class:"rounded-lg", src:"{img}"}
                  }
                 }
            }

         } else if msg.role == Role::Robot {
            div { class: "flex mb-4",
                div { class: "bg-gray-300 p-3 rounded-r-lg rounded-bl-lg",
                    if msg.loading {
                       Pulse {}
                    } else {
                       p { dangerous_inner_html: "{html}" }
                    }
                }
            }
        } else {
            div { class: "flex mb-4",
              div { class: "bg-blue-300 p-3 rounded-r-lg rounded-bl-lg",
                    if msg.loading {
                    Pulse {}
                    } else {
                    p { dangerous_inner_html: "{html}" }
                    }
               }
           }
        }

    )
}

fn sendMsg(
    msg: String,
    model_id: String,
    url: String,
    mut modelOptions: Signal<Vec<SelectOption>>,
    mut send_disabled: Signal<bool>,
) {
    if msg != "" {
        use reqwest::Client;

        let mut history = use_context::<Signal<Vec<Message>>>();
        let id = history().len();

        history.write().push(Message {
            id: id,
            role: Role::User,
            content: msg.clone(),
            img: None,
            loading: false,
        });

        let id = history().len();
        if msg.starts_with("/load") || msg.starts_with("/unload") {
            history.write().push(Message {
                id: id,
                role: Role::Admin,
                content: String::new(),
                img: None,
                loading: true,
            });
            let history_clone = history.read()[..id].to_owned();
            spawn(async move {
                let response = Client::new()
                    .post(format!("{}load", url))
                    .body(msg)
                    .send()
                    .await
                    .expect("Failed to post command!");
                let text = response
                    .text()
                    .await
                    .expect("Failed to get text from response!");
                let mut message = &mut history.write()[id];
                message.content.push_str(text.as_str());
                message.loading = false;

                let response = Client::new()
                    .get(format!("{}models", url))
                    .send()
                    .await
                    .unwrap()
                    .json::<Vec<String>>()
                    .await
                    .unwrap();
                let mut options: Vec<SelectOption> = response
                    .iter()
                    .map(|model| SelectOption {
                        text: model.clone(),
                        value: model.clone(),
                        selected: model_id == model.clone(),
                    })
                    .collect();
                modelOptions.write().clear();
                modelOptions.write().append(&mut options);
            });
        } else {
            history.write().push(Message {
                id: id,
                role: Role::Robot,
                content: String::new(),
                img: None,
                loading: true,
            });
            let history_clone = history.read()[..id].to_owned();

            spawn(async move {
                use crate::data::Request;
                use eventsource_stream::Eventsource;

                let mut stream = Client::new()
                    .post(format!("{}chat", url))
                    .json(&Request {
                        cmd: model_id.clone(),
                        msg_list: history_clone,
                    })
                    .send()
                    .await
                    .unwrap()
                    .bytes_stream()
                    .eventsource();
                let mut history = use_context::<Signal<Vec<Message>>>();
                let mut first_event = true;

                while let Some(event) = futures::StreamExt::next(&mut stream).await {
                    match event {
                        Ok(event) => {
                            let mut message = &mut history.write()[id];
                            if event.data == "[DONE]" {
                                if first_event {
                                    message.content.push_str(
                                        format!("Failed to find {} model server", model_id)
                                            .as_str(),
                                    );
                                    message.loading = false;
                                }

                                break;
                            }
                            message.content.push_str(event.data.as_str());
                        }
                        Err(_) => {
                            panic!("Error in event stream")
                        }
                    }
                    let mut message = &mut history.write()[id];
                    message.loading = false;
                    first_event = false;
                }
                send_disabled.set(false);
            });
        }
    }
}

pub fn swtich_session(id: &str, mut model_id: Signal<String>) {
    let mut session = use_context::<Signal<Session>>();
    let session_value = session();
    let current_id = session_value.id.clone();
    let mut temp_session = use_context::<Signal<TempSession>>();
    let mut messages = use_context::<Signal<Vec<Message>>>();
    if current_id != id {
        let mut store = Store::new().unwrap();
        if let Some(new_session) = store.get_session(id) {
            if let Some(ref history) = new_session.history {
                messages.set(history.clone());
            }
            if store.get_session(session().id.as_str()).is_none() {
                temp_session.set(TempSession::new(&session()));
            }
            session.set(Session {
                id: new_session.id,
                name: new_session.name,
                temp: new_session.temp,
                mode_id: new_session.mode_id.clone(),
                top_p: new_session.top_p,
                history: if let Some(ref history) = new_session.history {
                    Some(history.clone())
                } else {
                    None
                },
            });
            model_id.set(new_session.mode_id)
        } else {
            messages.set(Vec::<Message>::new());
            let new_session = temp_session.read();
            session.set(Session {
                id: new_session.id.clone(),
                name: new_session.name.clone(),
                temp: new_session.temp,
                mode_id: new_session.mode_id.clone(),
                top_p: new_session.top_p,
                history: Some(Vec::<Message>::new()),
            });
            model_id.set(new_session.mode_id.clone());
        }
    }
}

#[component]
pub fn Conversations(mut model_id: Signal<String>) -> Element {
    let session = use_context::<Signal<Session>>();
    let session_value = session();
    let mut store = Store::new().unwrap();

    let current_id = session_value.id.clone();
    let temp_session = use_context::<Signal<TempSession>>();
    let mut session_list = store.fetch_all_session();
    if store.get_session(current_id.as_str()).is_none() {
        session_list.push(session_value.clone());
    } else {
        if store.get_session(temp_session.read().id.as_str()).is_none() {
            session_list.push(Session {
                id: temp_session.read().id.clone(),
                name: temp_session.read().name.clone(),
                temp: temp_session.read().temp,
                mode_id: temp_session.read().mode_id.clone(),
                top_p: temp_session.read().top_p,
                history: Some(Vec::<Message>::new()),
            })
        }
    }
    session_list.reverse();
    rsx!(
        div { class: "w-2/12 shadow-lg rounded-lg text-sm font-medium text-gray-500  md:me-4 mb-4 md:mb-0 h-5/6",
            div { class: "border-b px-4 py-2 bg-gray-200",
                h1 { class: "text-lg font-semibold", "Conversations" }
            }

            ul { class: "flex-column mt-4 space-y space-y-4 text-sm font-medium text-gray-500 dark:text-gray-400 mb-4 md:me-2 md:ms-2 md:mb-0",

                for sess in  session_list {
                    if sess.id == current_id.clone() {
                        li {
                            a {
                                href: "#",
                                "aria-current": "page",
                                class: "inline-flex items-center px-4 py-3 text-white bg-blue-700 rounded-lg active w-full dark:bg-blue-600",
                                onclick: move |evt| {
                                    swtich_session(sess.id.as_str(),model_id);
                                },
                                "{sess.name}"
                            }
                        }
                    } else {
                        li {
                            a {
                                href: "#",
                                class: "inline-flex items-center px-4 py-3 rounded-lg hover:text-gray-900 bg-gray-50 hover:bg-gray-100 w-full dark:bg-gray-800 dark:hover:bg-gray-700 dark:hover:text-white",
                                onclick: move |evt| {
                                    swtich_session(sess.id.as_str(),model_id);
                                },
                                "{sess.name}"
                            }
                        }
                    }
                }


            }
        }
    )
}

fn new_conversation(mut session: Signal<Session>,mut messages: Signal<Vec<Message>>) {
    let mut store = Store::new().unwrap();
    
    if store.get_session(session().id.as_str()).is_some() {
        session.set(store.new_session());
        messages.set(Vec::<Message>::new());
    }
}

pub fn app() -> Element {
    use_context_provider(|| Signal::new(Vec::<Message>::new()));
    let href = if let Some(window) = window() {
        if let Some(document) = window.document() {
            if let Some(location) = document.location() {
                Some(format!(
                    "{}//{}/api/",
                    location.protocol().unwrap(),
                    location.host().unwrap()
                ))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let mut model_id = use_signal(|| String::from("meta-llama/Meta-Llama-3-8B-Instruct"));
    let mut endpoint = match href {
        Some(url) => use_signal(|| String::from(url)),
        None => use_signal(|| String::from("http://localhost:10201/api/")),
    };
    let mut new_msg = use_signal(String::new);
    let mut send_disabled = use_signal(|| false);
    let mut modelOptions = use_signal(Vec::<SelectOption>::new);

    let mut store = Store::new().unwrap();
    use_context_provider(|| Signal::new(store.new_session()));
    let mut session = use_context::<Signal<Session>>();
    use_context_provider(|| Signal::new(TempSession::new(&session())));

    use_effect(move || {
        let messages = use_context::<Signal<Vec<Message>>>();
        if let Some(window) = window() {
            if let Some(document) = window.document() {
                if let Some(scrollable_div) = document.get_element_by_id("list") {
                    scrollable_div.set_scroll_top(scrollable_div.scroll_height());
                }
                if let Some(last_msg) = messages().last() {
                    if !last_msg.loading {
                        if messages().len() > 1 && !send_disabled() {
                            let mut sess = session.write();
                            sess.mode_id = model_id();
                            sess.history = Some(messages().clone());
                            store.save_session(&sess);
                        }
                    }
                }

                let hljs = Reflect::get(&window, &JsValue::from_str("hljs")).unwrap();

                let highlight_all = Reflect::get(&hljs, &JsValue::from_str("highlightAll"))
                    .unwrap()
                    .dyn_into::<js_sys::Function>()
                    .unwrap();

                highlight_all.call0(&JsValue::NULL).unwrap();
            }
        }
    });

    let mut messages = use_context::<Signal<Vec<Message>>>();
    let mut send = move || {
        info!("try send message");
        if !send_disabled() {
            send_disabled.set(true);
            info!("send message");
            sendMsg(
                new_msg(),
                model_id(),
                endpoint(),
                modelOptions,
                send_disabled,
            );
            new_msg.set(String::new());
        }
    };

    rsx!(

        Conversations { model_id }

        div { class: "w-10/12 bg-white shadow-lg rounded-lg overflow-hidden flex flex-col h-5/6",
            div { class: "border-b px-4 py-2 bg-gray-200",
                h1 { class: "text-lg font-semibold", "Chat Robot" }
            }
            div { class: "flex-1 p-4 overflow-y-auto", id: "list",
                for msg in messages() {
                    ShowMessage { msg }
                }
            }
            div { class: "border-t px-4 py-2 bg-gray-200 flex items-center",
                input {
                    accept: "image/*",
                    r#type: "file",
                    class: "hidden",
                    id:"image-input",
                    onchange: move |evt| {
                        async move {
                        if let Some(file_engine) = &evt.files() {
                            let files = file_engine.files();
                            for file_name in files {
                                if let Some(file) = file_engine.read_file(&file_name).await{
                                    let id = messages().len();
                                    messages.write().push(Message {
                                        id: id,
                                        role: Role::User,
                                        content: String::new(),
                                        img: Some(image_base64_wasm::vec_to_base64(file)),
                                        loading: false,
                                    });
                                }
                            }
                        }
                    }
                }
                }
                button {
                    class: "cursor-pointer bg-gray-300 p-2 mr-4 rounded-lg hover:bg-gray-400",
                    onclick:move |_| {
                        async move {
                            use reqwest::Client;
                            let response = Client::new().get(format!("{}models",endpoint())).send().await.unwrap().json::<Vec<String>>().await.unwrap();
                            let mut options: Vec<SelectOption> = response.iter().map(|model| SelectOption {text:model.clone(),value:model.clone(),selected:model_id()==model.clone()}).collect();
                            modelOptions.write().clear();
                            modelOptions.write().append(&mut options);
                        }
                    },
                    "data-modal-target":"model-config",
                    "data-modal-toggle":"model-config",
                    svg { class: "w-6 h-6 text-gray-800 dark:text-white","aria-hidden":"true","xmlns":"http://www.w3.org/2000/svg",
                        "width":"24","height":"24","fill":"none",
                        "viewBox":"0 0 24 24",
                        path {
                            "stroke":"currentColor",
                            "stroke-linecap":"round",
                            "stroke-linejoin":"round",
                            "stroke-width":"2",
                            "d":"M21 13v-2a1 1 0 0 0-1-1h-.757l-.707-1.707.535-.536a1 1 0 0 0 0-1.414l-1.414-1.414a1 1 0 0 0-1.414 0l-.536.535L14 4.757V4a1 1 0 0 0-1-1h-2a1 1 0 0 0-1 1v.757l-1.707.707-.536-.535a1 1 0 0 0-1.414 0L4.929 6.343a1 1 0 0 0 0 1.414l.536.536L4.757 10H4a1 1 0 0 0-1 1v2a1 1 0 0 0 1 1h.757l.707 1.707-.535.536a1 1 0 0 0 0 1.414l1.414 1.414a1 1 0 0 0 1.414 0l.536-.535 1.707.707V20a1 1 0 0 0 1 1h2a1 1 0 0 0 1-1v-.757l1.707-.708.536.536a1 1 0 0 0 1.414 0l1.414-1.414a1 1 0 0 0 0-1.414l-.535-.536.707-1.707H20a1 1 0 0 0 1-1Z"
                        }
                        path { "stroke":"currentColor",
                                "stroke-linecap":"round",
                                "stroke-linejoin":"round",
                                "stroke-width":"2",
                                "d":"M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6Z"
                        }
                    }
                }
                label {
                    class: "cursor-pointer bg-gray-300 p-2 mr-4 rounded-lg hover:bg-gray-400",
                    onclick: move|evt| {
                        new_conversation(session,messages);
                    },
                    svg {
                        "stroke": "currentColor",
                        "fill": "none",
                        "xmlns": "http://www.w3.org/2000/svg",
                        "viewBox": "0 0 24 24",
                        class: "h-6 w-6 text-gray-600",
                        path {
                            "stroke-width": "2",
                            "d": "M12 4v16m8-8H4",
                            "stroke-linecap": "round",
                            "stroke-linejoin": "round"
                        }
                    }
                }
                input {
                    placeholder: "Type a message...",
                    r#type: "text",
                    class: "flex-1 px-4 py-2 mr-4 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-400",
                    value: "{new_msg}",
                    oninput: move |event| new_msg.set(event.value()),
                    onkeyup: move |event| if event.key() == Key::Enter {send();}
                }
                label {
                    r#for: "image-input",
                    class: "cursor-pointer bg-gray-300 p-2 mr-4 rounded-lg hover:bg-gray-400",
                    svg {
                        "aria-hidden": "true",
                        "fill": "none",
                        width: "24",
                        "viewBox": "0 0 24 24",
                        height: "24",
                        "xmlns": "http://www.w3.org/2000/svg",
                        class: "w-6 h-6 text-gray-800 dark:text-white",
                        path {
                            "stroke-linejoin": "round",
                            "stroke": "currentColor",
                            "d": "m3 16 5-7 6 6.5m6.5 2.5L16 13l-4.286 6M14 10h.01M4 19h16a1 1 0 0 0 1-1V6a1 1 0 0 0-1-1H4a1 1 0 0 0-1 1v12a1 1 0 0 0 1 1Z",
                            "stroke-width": "2",
                            "stroke-linecap": "round"
                        }
                    }
                }
                button { class: "bg-blue-500 text-white px-4 py-2 rounded-lg", disabled: "{send_disabled}",
                    onclick: move |_| {info!("send message");send()},
                    "Send",
                }
            }
        }
        ModelConfig { model_id, endpoint, modelOptions }
        script {
            "initFlowbite();"
        }
    )
}

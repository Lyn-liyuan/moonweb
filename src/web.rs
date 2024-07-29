#![allow(non_snake_case, unused)]
extern crate image_base64_wasm;

use crate::data::{Message, Role, SelectOption, WebUser};
use crate::web_state::{Session, Store, TempSession};
use crate::authorization::{LoginBox,get_user,show_login};
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
    modelOptions: Signal<Vec<SelectOption>>,
    mut system_prompt: Signal<String>,
) -> Element {
    let mut session = use_context::<Signal<Session>>();
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
                                        model_id.set(value.clone());
                                        let mut sess = session.write();
                                        sess.mode_id = value;
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
                                    r#for: "description",
                                    class: "block mb-2 text-sm font-medium text-gray-900 dark:text-white",
                                    "System prompt"
                                }
                                textarea {
                                    rows: "4",
                                    placeholder: "Write System Prompt",
                                    class: "block p-2.5 w-full text-sm text-gray-900 bg-gray-50 rounded-lg border border-gray-300 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-600 dark:border-gray-500 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500",
                                    value: "{system_prompt}",
                                    id: "System Prompt",
                                    onchange: move |evt| {
                                        let value = evt.value();
                                        system_prompt.set(value.clone());
                                        let mut sess = session.write();
                                        sess.system_prompt = value;
                                    }
                                    
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
    system_prompt: String,
    mut modelOptions: Signal<Vec<SelectOption>>,
    mut send_disabled: Signal<bool>,
) {
    
    if msg != "" {
        use reqwest::Client;
        let token = match get_user() {
            Some(user)=> match user.auth_key {
                Some(key) => key,
                None => "".to_string(),
            },
            None => "".to_string()
        };
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
                role: Role::Administrator,
                content: String::new(),
                img: None,
                loading: true,
            });
            let history_clone = history.read()[..id].to_owned();
            spawn(async move {
                let response = Client::new()
                    .post(format!("{}load", url))
                    .bearer_auth(token)
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
                send_disabled.set(false);
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
                    .bearer_auth(token)
                    .json(&Request {
                        cmd: model_id.clone(),
                        system_prompt: system_prompt,
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

pub fn switch_session(id: &str, mut model_id: Signal<String>,mut system_prompt: Signal<String>) {
    let mut session = use_context::<Signal<Session>>();
    let mut temp_session = use_context::<Signal<TempSession>>();
    let mut messages = use_context::<Signal<Vec<Message>>>();
    let current_id = session.read().id.clone();
    if current_id != id { 
        let mut store = Store::new().unwrap();
        // current session don't store.
        if store.get_session(current_id.as_str()).is_none() {
             temp_session.set(TempSession::new(&session()));
        }
        // load target session from store
        if let Some(new_session) = store.get_session(id) {
            if let Some(ref history) = new_session.history {
                messages.set(history.clone());
            }
            session.set(Session {
                id: new_session.id,
                name: new_session.name,
                mode_id: new_session.mode_id.clone(),
                system_prompt: new_session.system_prompt.clone(),
                history: if let Some(ref history) = new_session.history {
                    Some(history.clone())
                } else {
                    None
                },
            });
            info!(new_session.system_prompt);
            model_id.set(new_session.mode_id);
            system_prompt.set(new_session.system_prompt);
        } else {
            let new_session = Session {
                id: temp_session.read().id.clone(),
                name: temp_session.read().name.clone(),
                mode_id: temp_session.read().mode_id.clone(),
                system_prompt: temp_session.read().system_prompt.clone(),
                history:  Some(Vec::<Message>::new()),
            };
            messages.set(Vec::<Message>::new());
            model_id.set(new_session.mode_id.clone());
            info!(new_session.system_prompt);
            system_prompt.set(new_session.system_prompt.clone());
            session.set(new_session);
        }
    }
    
}

#[component]
pub fn Conversations(mut model_id: Signal<String>, mut system_prompt: Signal<String>, send_disabled: Signal<bool>) -> Element {
    let mut do_delete_conv = use_signal(|| false);
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
                mode_id: temp_session.read().mode_id.clone(),
                system_prompt: temp_session.read().system_prompt.clone(), 
                history: Some(Vec::<Message>::new()),
            })
        }
    }
    session_list.reverse();
    let items: Vec<_> = session_list.iter().map(|sess| {(sess.id.clone(),sess.id.clone(),sess.name.clone())}).collect();
    rsx!(
        div { class: "w-1/5 shadow-lg rounded-lg text-sm font-medium text-gray-500  md:me-4 mb-4 md:mb-0",
              style: "height:98%;",
            div { class: "border-b px-4 py-2 bg-gray-200",
                h1 { class: "text-lg font-semibold", "Conversations" }
            }

            div { class: "flex-column mt-4 space-y space-y-4 text-sm overflow-y-auto font-medium text-gray-500 dark:text-gray-400 mb-4 md:me-2 md:ms-2 md:mb-0",
                  style: "height:90%;",
                for (id_for_switch,id_for_delete,name) in items {
                    if id_for_switch == current_id.clone() {
                        div { class:"flex  items-center px-4 py-3 text-white bg-blue-700 rounded-lg active w-full dark:bg-blue-600",
                            a {
                                href: "#",
                                "aria-current": "page",
                                class: "inline-flex w-full",
                                onclick: move |evt| {
                                    if !send_disabled() {
                                        switch_session(id_for_switch.as_str(),model_id,system_prompt);
                                    }
                                },
                                "{name}"
                            }
                            
                        }
                    } else {
                        div { class:"flex items-center px-4 py-3 rounded-lg hover:text-gray-900 bg-gray-50 hover:bg-gray-100 w-full dark:bg-gray-800 dark:hover:bg-gray-700 dark:hover:text-white",
                            a {
                                href: "#",
                                class: "inline-flex w-full",
                                onclick: move |evt| {
                                    if !send_disabled() {
                                        switch_session(id_for_switch.as_str(),model_id,system_prompt);
                                    }
                                },
                                "{name}"
                            }
                            
                            button {
                                r#type: "button",
                                class: "text-gray-400 bg-transparent hover:bg-gray-200 hover:text-gray-900 rounded-lg text-sm w-8 h-8 ms-auto inline-flex justify-center items-center dark:hover:bg-gray-600 dark:hover:text-white",
                                onclick: move |evt| {
                                    let mut store = Store::new().unwrap();
                                    if !send_disabled() {
                                        store.remove_session(id_for_delete.clone().as_str());
                                        do_delete_conv.set(true);
                                    }
                                },
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
                                span { class: "sr-only", "Remove Conversation" }
                            }
                        }
                    }
                }


            }
        }
        if do_delete_conv() {
            span { dangerous_inner_html: "<!--delete conversation re-rendering trigger--> "}
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
    let mut system_prompt = use_signal(|| String::from("You are helpful assistant!"));
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
                            sess.system_prompt = system_prompt();
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
                system_prompt(),
                modelOptions,
                send_disabled,
            );
            new_msg.set(String::new());
        }
    };

    rsx!(

        Conversations { model_id, system_prompt, send_disabled }

        div { class: "w-4/5 bg-white shadow-lg rounded-lg overflow-hidden flex flex-col",
              style: "height:98%;",
            div { class: "flex border-b px-4 py-2 bg-gray-200",
                h1 { class: "text-lg font-semibold text-gray-500", "Chat Robot" }
                button {
                    r#type: "button",
                    class: "text-gray-400 bg-transparent hover:bg-gray-300 hover:text-gray-600 rounded-lg text-sm w-8 h-8 ms-auto inline-flex justify-center items-center dark:hover:bg-gray-600 dark:hover:text-white",
                    onclick: move |evt| {
                        show_login(true);
                    },
                    svg {
                        "fill": "none",
                        "xmlns": "http://www.w3.org/2000/svg",
                        height: "24",
                        "viewBox": "0 0 24 24",
                        "aria-hidden": "true",
                        width: "24",
                        class: "w-6 h-6 text-gray-500 dark:text-white",
                        path {
                            "stroke": "currentColor",
                            "stroke-width": "2",
                            "stroke-linecap": "round",
                            "stroke-linejoin": "round",
                            "d": "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18Zm0 0a8.949 8.949 0 0 0 4.951-1.488A3.987 3.987 0 0 0 13 16h-2a3.987 3.987 0 0 0-3.951 3.512A8.948 8.948 0 0 0 12 21Zm3-11a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z"
                        }
                    }
                    span { class: "sr-only", "Sign In or Sign Out" }
                }
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
        ModelConfig { model_id, modelOptions, system_prompt }
        LoginBox { endpoint }
        script {
            "initFlowbite();"
        }

    )
}

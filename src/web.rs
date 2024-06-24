#![allow(non_snake_case, unused)]
extern crate image_base64_wasm;
//use crate::apiserver::chat_stream;
use crate::data::{Message, Role, SelectOption};
use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
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
fn ModelConfig(model_id: Signal<String>, endpoint: Signal<String>, modelOptions: Signal<Vec<SelectOption>>) -> Element {
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

fn sendMsg(msg: String, model_id: String, url: String, mut modelOptions:Signal<Vec<SelectOption>>) {
    
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
        if msg.starts_with("/load") {
            history.write().push(Message {
                id: id,
                role: Role::Admin,
                content: String::new(),
                img: None,
                loading: true,
            });
            let history_clone = history.read()[..id].to_owned();
            spawn(async move {
                let response = Client::new().post(format!("{}load",url)).body(msg).send().await.expect("Failed to post command!");
                let text = response.text().await.expect("Failed to get text from response!");
                let mut message = &mut history.write()[id];
                message.content.push_str(text.as_str());
                message.loading = false;
                spawn(async move {
                    let response = Client::new().get(format!("{}models",url)).send().await.unwrap().json::<Vec<String>>().await.unwrap();
                    let mut options: Vec<SelectOption> = response.iter().map(|model| SelectOption {text:model.clone(),value:model.clone(),selected:model_id==model.clone()}).collect();
                    modelOptions.write().clear();
                    modelOptions.write().append(&mut options);
                }); 
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
                    .post(format!("{}chat",url))
                    .json(&Request {
                        cmd: model_id,
                        msg_list: history_clone,
                    })
                    .send()
                    .await
                    .unwrap()
                    .bytes_stream()
                    .eventsource();
                let mut history = use_context::<Signal<Vec<Message>>>();

                while let Some(event) = futures::StreamExt::next(&mut stream).await {
                    match event {
                        Ok(event) => {
                            if event.data == "[DONE]" {
                                break;
                            }
                            let mut message = &mut history.write()[id];
                            message.content.push_str(event.data.as_str());
                            message.loading = false; 
                        }
                        Err(_) => {
                            panic!("Error in event stream")
                        }
                    }
                }
            });
        }
    }
}

pub fn app() -> Element {
    use_context_provider(|| Signal::new(Vec::<Message>::new()));
    let mut model_id = use_signal(|| String::from("meta-llama/Meta-Llama-3-8B-Instruct"));
    let mut endpoint = use_signal(|| String::from("http://localhost:8080/api/"));
    let mut new_msg = use_signal(String::new);
    let mut send_disabled = use_signal(|| false);
    let mut modelOptions = use_signal(Vec::<SelectOption>::new);

    use_effect(move || {
        let messages = use_context::<Signal<Vec<Message>>>();
        if let Some(window) = window() {
            if let Some(document) = window.document() {
                if let Some(scrollable_div) = document.get_element_by_id("list") {
                    scrollable_div.set_scroll_top(scrollable_div.scroll_height());
                }
                if let Some(last_msg) = messages().last() {
                    if !last_msg.loading {
                        send_disabled.set(false);
                    }
                }
            }
        }
    });


    let mut messages = use_context::<Signal<Vec<Message>>>();
    let mut send = move || {
        info!("try send message");
        if !send_disabled() {
            send_disabled.set(true);
            info!("send message");
            sendMsg(new_msg(), model_id(), endpoint(), modelOptions);
            new_msg.set(String::new());
        }
    };
    use reqwest::Client;
    spawn(async move {
        let response = Client::new().get(format!("{}models",endpoint())).send().await.unwrap().json::<Vec<String>>().await.unwrap();
        let mut options: Vec<SelectOption> = response.iter().map(|model| SelectOption {text:model.clone(),value:model.clone(),selected:model_id()==model.clone()}).collect();
        modelOptions.write().clear();
        modelOptions.write().append(&mut options);
    });

    rsx!(
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
                r#for: "image-input",
                class: "cursor-pointer bg-gray-300 p-2 mr-4 rounded-lg hover:bg-gray-400",
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
            button { class: "bg-blue-500 text-white px-4 py-2 rounded-lg", disabled: "{send_disabled}",
                onclick: move |_| {info!("send message");send()},
                "Send",
            }
        }
        ModelConfig { model_id, endpoint, modelOptions }
        script {
            "initFlowbite();"
        }
    )
}

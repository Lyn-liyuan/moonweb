use crate::data::{AuthResponse, WebUser};
use dioxus::prelude::*;
use js_sys::Date;
use js_sys::Reflect;
use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
use sqids::Sqids;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, Document, HtmlInputElement};

const SQIDS_ALPHABET: &str = "VRHIrU2je0gxcSGlzvMWBAkpufqDiyEoY931JLTC5wN6KbaQFPOdsXn48h7mZt";
const SALT: &str = "Akpu#fqDiy@EoY931J_VRHIrU2";

fn get_input_element_by_id(document: &Document, id: &str) -> Result<HtmlInputElement, JsValue> {
    document
        .get_element_by_id(id)
        .ok_or_else(|| JsValue::from_str("element not found"))
        .and_then(|element| {
            element
                .dyn_into::<HtmlInputElement>()
                .map_err(|_| JsValue::from_str("element is not an HtmlInputElement"))
        })
}

pub fn get_user() -> Option<WebUser> {
    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(value)) = storage.get_item("auth_user") {
                if let Ok(user) = serde_json::from_str::<WebUser>(value.as_str()) {
                    return Some(user);
                }
            }
        }
    }
    return None;
}

async fn do_login(endpoint: Signal<String>, mut logined: Signal<bool>,mut login_failed: Signal<bool>) {
    if let Some(window) = window() {
        if let Some(document) = window.document() {
            let role = if let Ok(user) = get_input_element_by_id(&document, "role-1") {
                if user.checked() {
                    Some("User")
                } else {
                    None
                }
            } else {
                None
            };
            let role = match role {
                Some(r) => r,
                None => {
                    if let Ok(user) = get_input_element_by_id(&document, "role-2") {
                        if user.checked() {
                            "Administrator"
                        } else {
                            "User"
                        }
                    } else {
                        "User"
                    }
                }
            };
            let token = if let Ok(key) = get_input_element_by_id(&document, "key") {
                key.value()
            } else {
                String::new()
            };
            if token != "" {
                let token_digest = md5::compute(format!("{}{}", SALT, token).as_bytes());
                let response = Client::new()
                    .post(format!("{}signin", endpoint()))
                    .header(CONTENT_TYPE, "application/json")
                    .body(format!(
                        "{{\"role\":\"{}\",\"token\":\"{:x}\"}}",
                        role, token_digest
                    ))
                    .send()
                    .await
                    .unwrap()
                    .json::<AuthResponse>()
                    .await
                    .unwrap();
                if response.success {
                    logined.set(true);
                    login_failed.set(false);
                    let user =
                        WebUser::make(role.parse().unwrap(), response.auth_key, response.expire);
                    if let Ok(Some(storage)) = window.local_storage() {
                        storage
                            .set_item("auth_user", serde_json::json!(user).to_string().as_str())
                            .unwrap();
                    }
                } else {
                    logined.set(false);
                    if let Ok(Some(storage)) = window.local_storage() {
                        storage.delete("auth_user").unwrap();
                    }
                    login_failed.set(true);
                }
            }
        }
    }
}

fn is_signin() -> bool {
    let sqids = Sqids::builder()
        .alphabet(SQIDS_ALPHABET.chars().collect())
        .build()
        .unwrap();

    if let Some(user) = get_user() {
        if let Some(expire) = user.expire {
            let expire_raw = sqids.decode(expire.as_str());
            let expire = Date::new_with_year_month_day(
                expire_raw[0] as u32,
                expire_raw[1] as i32,
                expire_raw[2] as i32,
            );
            let now = Date::new_0();
            if now.value_of() <= expire.value_of() {
                return true;
            }
        }
    }

    return false;
}

pub fn show_login(closeable:bool) {
    if let Some(window) = window() {
        let show_login_js = Reflect::get(&window, &JsValue::from_str("showLogin"))
        .unwrap()
        .dyn_into::<js_sys::Function>()
        .unwrap();
        show_login_js.call0(&JsValue::from_bool(closeable)).unwrap();
    }
}

pub fn close_login() {
    if let Some(window) = window() {
        let close_login_js = Reflect::get(&window, &JsValue::from_str("closeLogin"))
        .unwrap()
        .dyn_into::<js_sys::Function>()
        .unwrap();
        close_login_js.call0(&JsValue::NULL).unwrap();
    }
}


#[component]
pub fn LoginBox(endpoint: Signal<String>) -> Element {
    let logined = use_signal(|| is_signin());
    let login_failed = use_signal(|| false);
    use_effect(move || {        
        if logined() {
            close_login();
        } else {
            show_login(false);
        }
    });
    rsx! {
        div {
            tabindex: "-1",
            "aria-hidden": "true",
            class: "hidden overflow-y-auto overflow-x-hidden fixed top-0 right-0 left-0 z-50 justify-center items-center w-full md:inset-0 h-full max-h-full",
            id: "login-modal",
            div { class: "relative p-4 w-full max-w-md max-h-full",
                div { class: "relative bg-white rounded-lg shadow dark:bg-gray-700",
                    div { class: "flex items-center justify-between p-4 md:p-5 border-b rounded-t dark:border-gray-600",
                        h3 { class: "text-lg font-semibold text-gray-900 dark:text-white",
                            "\n                    User Sign In\n                "
                        }
                        if logined() {
                            button {
                                r#type: "button",
                                class: "text-gray-400 bg-transparent hover:bg-gray-200 hover:text-gray-900 rounded-lg text-sm w-8 h-8 ms-auto inline-flex justify-center items-center dark:hover:bg-gray-600 dark:hover:text-white",
                                onclick: |_| {
                                    close_login();
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
                    div { class: "p-4 md:p-5",
                        p { class: "text-gray-500 dark:text-gray-400 mb-4",
                            "Select your desired position:"
                        }
                        ul { class: "space-y-4 mb-4",
                            li {
                                input {
                                    name: "role",
                                    required: true,
                                    value: "user",
                                    r#type: "radio",
                                    class: "hidden peer",
                                    id: "role-1",
                                    checked:true,
                                }
                                label {
                                    r#for: "role-1",
                                    class: "inline-flex items-center justify-between w-full p-5 text-gray-900 bg-white border border-gray-200 rounded-lg cursor-pointer dark:hover:text-gray-300 dark:border-gray-500 dark:peer-checked:text-blue-500 peer-checked:border-blue-600 peer-checked:text-blue-600 hover:text-gray-900 hover:bg-gray-100 dark:text-white dark:bg-gray-600 dark:hover:bg-gray-500",
                                    div { class: "block",
                                        div { class: "w-full text-base font-semibold", "User" }
                                    }

                                }
                            }
                            li {
                                input {
                                    name: "role",
                                    required: true,
                                    value: "administrator",
                                    r#type: "radio",
                                    class: "hidden peer",
                                    id: "role-2"
                                }
                                label {
                                    r#for: "role-2",
                                    class: "inline-flex items-center justify-between w-full p-5 text-gray-900 bg-white border border-gray-200 rounded-lg cursor-pointer dark:hover:text-gray-300 dark:border-gray-500 dark:peer-checked:text-blue-500 peer-checked:border-blue-600 peer-checked:text-blue-600 hover:text-gray-900 hover:bg-gray-100 dark:text-white dark:bg-gray-600 dark:hover:bg-gray-500",
                                    div { class: "block",
                                        div { class: "w-full text-base font-semibold", "Administrator" }

                                    }

                                }
                            }
                        }
                        if !login_failed() {
                            div { class: "space-y-4 mb-4",
                                label {
                                    r#for: "key",
                                    class: "block mb-2 text-gray-500 dark:text-white",
                                    "Authentication Key:"
                                }
                                input {
                                    r#type: "text",
                                    placeholder: "Authentication Key",
                                    required: true,
                                    name: "key",
                                    class: "bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-primary-600 focus:border-primary-600 block w-full p-2.5 dark:bg-gray-600 dark:border-gray-500 dark:placeholder-gray-400 dark:text-white dark:focus:ring-primary-500 dark:focus:border-primary-500",
                                    id: "key"
                                }
                            }
                        } else {
                            div { class: "space-y-4 mb-4",
                                label {
                                    r#for: "key",
                                    class: "block mb-2 text-red-700 dark:text-white",
                                    "Authentication Key:"
                                }
                                input {
                                    r#type: "text",
                                    placeholder: "Authentication Key",
                                    required: "true",
                                    name: "key",
                                    class: "bg-gray-50 border border-red-500 text-red-900 placeholder-red-700 text-sm rounded-lg focus:ring-primary-600 focus:border-primary-600 block w-full p-2.5 dark:bg-gray-600 dark:border-gray-500 dark:placeholder-gray-400 dark:text-white dark:focus:ring-primary-500 dark:focus:border-primary-500",
                                    id: "key"
                                }
                                p {
                                    class:"mt-2 text-sm text-red-600 dark:text-red-500",
                                    span {
                                        class:"font-medium",
                                        "Oops! "
                                    }
                                    " Authentication Key Error!"
                                }
                            }
                        }
                        button { class: "text-white inline-flex w-full justify-center bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800",
                            onclick: move |_| async move {
                                do_login(endpoint,logined,login_failed).await;
                            },
                            "\n                    Sign In\n                "
                        }
                    }
                }
            }
        }
        if logined() {
            span { dangerous_inner_html: "<!--re-rendering trigger--> "}
        }
    }
}

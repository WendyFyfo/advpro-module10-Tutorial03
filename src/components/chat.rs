use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
    color: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
}

impl Chat {
    fn parse_users(usernames: Option<Vec<String>>) -> Vec<UserProfile> {
        let palette = vec![
            "#fce4ec", "#e3f2fd", "#f3e5f5", "#e8f5e9", "#fff8e1", "#fbe9e7",
            "#ede7f6", "#e0f7fa", "#f9fbe7", "#f1f8e9"
        ];

        usernames.unwrap_or_default().into_iter().enumerate().map(|(i, u)| {
            UserProfile {
                name: u.clone(),
                avatar: format!("https://api.dicebear.com/9.x/pixel-art/svg?seed={}", u),
                color: palette[i % palette.len()].to_string(),
            }
        }).collect()
    }
}

impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx.link().context::<User>(Callback::noop()).expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        let _ = wss.tx.clone().try_send(serde_json::to_string(&message).unwrap());

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                if let Ok(msg) = serde_json::from_str::<WebSocketMessage>(&s) {
                    match msg.message_type {
                        MsgTypes::Users => {
                            self.users = Self::parse_users(msg.data_array);
                            true
                        },
                        MsgTypes::Message => {
                            if let Some(raw) = msg.data {
                                if let Ok(message_data) = serde_json::from_str(&raw) {
                                    self.messages.push(message_data);
                                    return true;
                                }
                            }
                            false
                        },
                        _ => false,
                    }
                } else {
                    false
                }
            }
            Msg::SubmitMessage => {
                if let Some(input) = self.chat_input.cast::<HtmlInputElement>() {
                    let text = input.value().trim().to_string();
                    if !text.is_empty() {
                        let message = WebSocketMessage {
                            message_type: MsgTypes::Message,
                            data: Some(text),
                            data_array: None,
                        };
                        let _ = self.wss.tx.clone().try_send(serde_json::to_string(&message).unwrap());
                        input.set_value("");
                    }
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);

        html! {
            <div class="flex w-screen bg-gradient-to-br from-blue-50 to-pink-50">
                <div class="flex-none w-56 h-screen bg-amber-25 overflow-y-auto backdrop-blur">
                    <div class="text-xl px-3 pt-3 pb-3.5 font-semibold bg-amber-200 border-l-2 border-b-2 border-amber-300">{"Users"}</div>
                    {
                        self.users.clone().iter().map(|u| {
                            html!{
                                <div class="flex m-3 rounded-lg p-2 border-1 border-amber-300" style={format!("background-color:{}", u.color)}>
                                    <div>
                                        <img class="w-12 h-12 rounded-full hover:scale-110 hover:brightness-125 transition-transform duration-300" src={u.avatar.clone()} alt="avatar"/>
                                    </div>
                                    <div class="flex-grow p-3">
                                        <div class="flex text-xs justify-between">
                                            <div class="font-semibold">{u.name.clone()}</div>
                                        </div>
                                        <div class="text-xs text-gray-600">{"Hi there!"}</div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
                <div class="grow h-screen flex flex-col">
                    <div class="w-full h-14 border-b-2 border-pink-300 border-l-2 bg-pink-200"><div class="text-xl p-3 font-semibold">{"UwU Cafee Chat"}</div></div>
                    <div class="w-full grow overflow-auto border-l-1 border-b-2 border-pink-300 bg-pink-50 px-4 py-2">
                        {
                            self.messages.iter().map(|m| {
                                let user_opt = self.users.iter().find(|u| u.name == m.from);
                                let (avatar, color) = user_opt
                                    .map(|u| (u.avatar.clone(), u.color.clone()))
                                    .unwrap_or_else(|| ("https://api.dicebear.com/9.x/pixel-art/svg?seed=unknown".to_string(), "#ffffff".to_string()));
                                html! {
                                    <div class="flex items-end max-w-md m-4 rounded-tl-[0.25rem] rounded-tr-[1rem] rounded-br-[1rem] border" style={format!("background-color:{}; border-color:{}", color, color)}>
                                        <img class="w-8 h-8 rounded-full m-3" src={avatar} alt="avatar"/>
                                        <div class="p-3">
                                            <div class="text-sm font-semibold">{m.from.clone()}</div>
                                            <div class="text-xs text-gray-800">
                                                { if m.message.ends_with(".gif") {
                                                    html! { <img class="mt-3" src={m.message.clone()} /> }
                                                } else {
                                                    html! { <span>{m.message.clone()}</span> }
                                                } }
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                    <div class="w-full h-14 flex px-3 items-center bg-pink-200 border-pink-300 border-l-2 backdrop-blur">
                        <input ref={self.chat_input.clone()} type="text" placeholder="Message" class="bg-white text-gray-700 border border-pink-300 focus:border-blue-400 focus:ring-2 focus:ring-blue-200 rounded-full px-4 py-2 transition-all duration-300 w-full placeholder-gray-500" name="message" required=true />
                        <button onclick={submit} class="ml-3 transition-transform hover:scale-110 active:translate-x-1 bg-pink-500 hover:bg-pink-600 text-white p-2 rounded-full">
                            <svg class="w-5 h-5 fill-current" viewBox="0 0 24 24"><path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"/></svg>
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}

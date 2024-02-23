#![allow(unused_variables)]
#![allow(dead_code)]

use gloo::utils::window;
use nanoid::nanoid;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::HtmlImageElement;
use yew::platform::spawn_local;
use yew::prelude::*;

use crate::db::friend::FriendRepo;
use crate::db::user::UserRepo;
use crate::icons::{MsgPhoneIcon, VideoRecordIcon};
use crate::model::message::{InviteMsg, InviteType, Message};
use crate::model::user::User;
use crate::pages::RecSendMessageState;
use crate::{components::right::friend_card::FriendCard, model::ContentType};

pub struct MsgItem {
    show_img_preview: bool,
    show_friend_card: bool,
    msg_state: Rc<RecSendMessageState>,
}

pub enum MsgItemMsg {
    PreviewImg(AttrValue),
    ShowFriendCard(MouseEvent),
    CallVideo,
    None,
    CallAudio,
}

#[derive(Properties, Clone, PartialEq)]
pub struct MsgItemProps {
    pub user_id: AttrValue,
    pub friend_id: AttrValue,
    pub avatar: AttrValue,
    pub msg: Message,
}

impl Component for MsgItem {
    type Message = MsgItemMsg;
    type Properties = MsgItemProps;

    fn create(_ctx: &Context<Self>) -> Self {
        let (msg_state, _listener) = _ctx
            .link()
            .context(_ctx.link().callback(|_| MsgItemMsg::None))
            .expect("need msg context");
        Self {
            show_img_preview: false,
            show_friend_card: false,
            msg_state,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            MsgItemMsg::PreviewImg(src) => {
                self.show_img_preview = true;
                // let win = window().open().unwrap().unwrap();
                // let img = win
                //     .document()
                //     .unwrap()
                //     .create_element("img")
                //     .unwrap()
                //     .dyn_into::<HtmlImageElement>()
                //     .unwrap();
                // img.set_src(&src);
                true
            }
            MsgItemMsg::ShowFriendCard(event) => {
                self.show_friend_card = !self.show_friend_card;
                // 获取xy
                let x = event.x();
                let y = event.y();

                // let friend_id = if ctx.props().msg.is_self {
                //     ctx.props().user_id.clone()
                // } else {
                //     ctx.props().friend_id.clone()
                // };
                let is_self = ctx.props().msg.is_self;
                let friend_id = ctx.props().friend_id.clone();
                let user_id = ctx.props().user_id.clone();
                spawn_local(async move {
                    // 查询好友信息
                    log::debug!("friend id in msg item: {}", &friend_id);
                    let user = if is_self {
                        UserRepo::new().await.get(user_id).await.unwrap()
                    } else {
                        let friend = FriendRepo::new().await.get_friend(friend_id).await;
                        User::from(friend)
                    };
                    FriendCard::show(user, None, true, x, y);
                });
                false
            }
            MsgItemMsg::CallVideo => {
                self.msg_state.call_event.emit(InviteMsg {
                    msg_id: nanoid!().into(),
                    send_id: ctx.props().user_id.clone(),
                    friend_id: ctx.props().friend_id.clone(),
                    create_time: chrono::Local::now().timestamp_millis(),
                    invite_type: InviteType::Video,
                });
                false
            }
            MsgItemMsg::CallAudio => {
                self.msg_state.call_event.emit(InviteMsg {
                    msg_id: nanoid!().into(),
                    send_id: ctx.props().user_id.clone(),
                    friend_id: ctx.props().friend_id.clone(),
                    create_time: chrono::Local::now().timestamp_millis(),
                    invite_type: InviteType::Audio,
                });
                false
            }
            MsgItemMsg::None => {
                // 不需要监听值得变化，这里只是占位符的作用
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let id = ctx.props().msg.create_time;
        let mut classes = Classes::from("msg-item");
        let msg_type = ctx.props().msg.content_type;

        let mut msg_content_classes = Classes::from("msg-item-content");
        if ctx.props().msg.is_self {
            msg_content_classes.push("background-self");
            classes = Classes::from("msg-item-reverse");
        } else {
            msg_content_classes.push("background-other");
        }

        let img_preview = html! {};
        let content = match msg_type {
            ContentType::Text => {
                html! {
                    <div class={msg_content_classes}>
                        {ctx.props().msg.content.clone()}
                    </div>
                }
            }
            // todo 限制图片宽度，高度自适应，聊天列表展示缩略图，点击查看原图
            ContentType::Image => {
                let img_url = if ctx.props().msg.file_content.is_empty() {
                    AttrValue::from(format!("/api/file/get/{}", ctx.props().msg.content.clone()))
                } else {
                    ctx.props().msg.file_content.clone()
                };
                let src = img_url.clone();
                let onclick = ctx
                    .link()
                    .callback(move |_: MouseEvent| MsgItemMsg::PreviewImg(src.clone()));
                html! {
                <>
                    <div class="msg-item-content">
                        <div class="img-mask">
                        </div>
                        <img class="msg-item-img" src={img_url} {onclick}/>
                    </div>
                </>
                }
            }
            ContentType::Video => html! {
                <div class="msg-item-content">
                    <video class="msg-item-video">
                        <source src={ctx.props().msg.content.clone()} type="video/mp4" />
                    </video>
                </div>
            },
            ContentType::File => {
                let full = ctx.props().msg.content.clone();
                let file_name = ctx.props().msg.content.split("-").last().unwrap_or(&full);
                html! {
                    <div class="msg-item-content">
                        <span class="msg-item-file-name">
                            {file_name}
                        </span>
                    </div>
                }
            }
            ContentType::Emoji => {
                html! {
                    <div class="msg-item-emoji">
                        // <span class="msg-item-emoji">
                            <img class="emoji" src={ctx.props().msg.content.clone()} />
                        // </span>
                    </div>
                }
            }
            ContentType::VideoCall => {
                let onclick = ctx.link().callback(|_| MsgItemMsg::CallVideo);
                html! {
                    <div class={msg_content_classes} {onclick} style="cursor: pointer;">
                        {ctx.props().msg.content.clone()}
                        {"\t"}
                        <VideoRecordIcon/>
                    </div>
                }
            }
            ContentType::AudioCall => {
                let onclick = ctx.link().callback(|_| MsgItemMsg::CallAudio);
                html! {
                    <div class={msg_content_classes} {onclick} style="cursor: pointer;">
                        {ctx.props().msg.content.clone()}
                        {"\t"}
                         <MsgPhoneIcon />
                    </div>
                }
            }
            ContentType::Default => html! {},
            ContentType::Audio => html!(),
        };

        html! {
            <>
                {
                    if self.show_img_preview {
                       html!{img_preview}
                    }else {
                        html!{}
                    }
                }
            <div class={classes} id={id.to_string()} >
                <div class="msg-item-avatar">
                    <img class="avatar" src={ctx.props().avatar.clone()} onclick={ctx.link().callback(MsgItemMsg::ShowFriendCard)} />
                </div>
                <div class="content-wrapper">
                    {content}
                </div>
            </div>
            </>
        }
    }
}
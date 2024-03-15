use std::{cell::RefCell, rc::Rc};

use gloo::utils::window;
use yew::{AttrValue, Context, NodeRef};

use crate::model::{ComponentType, CurrentItem};
use crate::pages::CreateConvState;
use crate::{
    db::{
        current_item, friend::FriendRepo, friend_ship::FriendShipRepo, group_msg::GroupMsgRepo,
        message::MessageRepo, user::UserRepo, QueryError, QueryStatus, DB_NAME, TOKEN, WS_ADDR,
    },
    model::{
        friend::{Friend, FriendShipWithUser},
        message::{InviteMsg, Message, Msg, SingleCall, DEFAULT_HELLO_MESSAGE},
        notification::{Notification, NotificationState, NotificationType},
        user::User,
        ContentType, FriendShipStateType,
    },
    pages::{
        home_page::HomeMsg, AppState, ConvState, FriendListState, FriendShipState,
        RecSendCallState, RecSendMessageState, RemoveConvState, RemoveFriendState, UnreadState,
        WaitState,
    },
    ws::WebSocketManager,
};

use super::{Home, QueryResult, WAIT_COUNT};

async fn query(id: AttrValue) -> Result<QueryResult, QueryError> {
    let user_repo = UserRepo::new().await;
    let user = user_repo.get(id).await.unwrap();
    Ok((
        user,
        current_item::get_conv(),
        current_item::get_friend(),
        current_item::get_com_type(),
    ))
}

impl Home {
    pub fn new(ctx: &Context<Self>) -> Self {
        // 测试数据库
        // 查询当前登录用户放到登录中
        let id = ctx.props().id.clone();
        // 每次创建Home组件时，检查一下数据库名是否存在，不存在则创建
        // 这样就能保证每次创建Home组件时，数据库名都是当前登录用户的id
        DB_NAME.get_or_init(|| format!("im-{}", id.clone()));
        let cloned_id = id.clone();
        ctx.link().send_future(async move {
            match query(cloned_id).await {
                Ok(data) => HomeMsg::Query(QueryStatus::QuerySuccess(data)),
                Err(err) => HomeMsg::Query(QueryStatus::QueryFail(err)),
            }
        });

        // 使用ctx发送一个正在查询的状态
        ctx.link()
            .send_message(HomeMsg::Query(QueryStatus::Querying));
        let callback = ctx.link().callback(HomeMsg::SwitchComponent);
        let switch_friend_callback = ctx.link().callback(HomeMsg::SwitchFriend);
        let switch_conv_callback = ctx.link().callback(HomeMsg::SwitchConv);
        let remove_conv_callback = ctx.link().callback(HomeMsg::RemoveConv);
        let remove_event = ctx.link().callback(HomeMsg::RemoveFriend);
        let add_contact_count = ctx.link().callback(|_| HomeMsg::AddUnreadContactCount);
        let sub_contact_count = ctx.link().callback(HomeMsg::SubUnreadContactCount);
        let sub_msg_count = ctx.link().callback(HomeMsg::SubUnreadMsgCount);
        let add_msg_count = ctx.link().callback(HomeMsg::AddUnreadMsgCount);
        let ready = ctx.link().callback(|_| HomeMsg::WaitStateChanged);
        let rec_msg_event = ctx.link().callback(HomeMsg::RecSendMsgStateChange);
        let rec_listener = ctx.link().callback(HomeMsg::ReceiveMessage);
        let send_msg_event = ctx.link().callback(HomeMsg::SendMessage);
        let send_back_event = ctx.link().callback(HomeMsg::SendBackMsg);
        let call_event = ctx.link().callback(HomeMsg::SendCallInvite);
        let rec_friend_req_event = ctx.link().callback(HomeMsg::ReceiveFriendShipReq);
        let rec_friend_res_event = ctx.link().callback(HomeMsg::FriendShipResponse);
        let error_event = ctx.link().callback(HomeMsg::Notification);
        let create_friend_conv = ctx.link().callback(HomeMsg::CreateFriendConv);
        let create_group_conv = ctx.link().callback(HomeMsg::CreateGroupConv);
        // 不能用这么多unwrap()
        let token = window()
            .local_storage()
            .unwrap()
            .unwrap()
            .get(TOKEN)
            .unwrap()
            .unwrap();
        let addr = window()
            .local_storage()
            .unwrap()
            .unwrap()
            .get(WS_ADDR)
            .unwrap()
            .unwrap();
        let url = format!("{}/{}/conn/{}/{}", addr, id.clone(), token, id);
        let ws = Rc::new(RefCell::new(WebSocketManager::new(url, rec_listener)));
        Self {
            state: Rc::new(AppState {
                component_type: ComponentType::Messages,
                switch_com_event: callback,
                ..Default::default()
            }),
            msg_state: Rc::new(RecSendMessageState {
                msg: Msg::Single(Message::default()),
                send_back_event,
                send_msg_event: send_msg_event.clone(),
                call_event: call_event.clone(),
            }),
            user: User::default(),
            conv_state: Rc::new(ConvState {
                conv: CurrentItem::default(),
                state_change_event: switch_conv_callback,
            }),
            remove_conv_state: Rc::new(RemoveConvState {
                id: AttrValue::default(),
                remove_event: remove_conv_callback,
            }),
            unread_state: Rc::new(UnreadState {
                unread: current_item::get_unread_count(),
                add_contact_count,
                add_msg_count,
                sub_contact_count,
                sub_msg_count,
            }),
            ws,
            friend_ship_state: Rc::new(FriendShipState {
                ship: None,
                friend: None,
                state_type: FriendShipStateType::Req,
                req_change_event: rec_friend_req_event,
                res_change_event: rec_friend_res_event,
            }),
            friend_state: Rc::new(FriendListState {
                friend: Default::default(),
                state_change_event: switch_friend_callback,
            }),
            notifications: vec![],
            notification: Rc::new(NotificationState {
                notify: error_event,
            }),
            notification_node: NodeRef::default(),
            notification_interval: None,
            call_state: Rc::new(RecSendCallState {
                msg: InviteMsg::default(),
                send_msg_event,
                rec_msg_event,
                call_event,
            }),
            call_msg: SingleCall::default(),
            wait_state: Rc::new(WaitState {
                wait_count: WAIT_COUNT,
                ready,
            }),
            remove_friend_state: Rc::new(RemoveFriendState::with_event(remove_event)),
            create_conv: Rc::new(CreateConvState {
                friend: None,
                group: None,
                type_: crate::model::RightContentType::Default,
                create_friend: create_friend_conv,
                create_group: create_group_conv,
            }),
        }
    }
    pub fn send_msg(&self, msg: &Msg) {
        // 发送已收到消息给服务器
        match self
            .ws
            .borrow()
            .send_message(&serde_json::to_string(&msg).unwrap())
        {
            Ok(_) => {
                log::info!("发送成功")
            }
            Err(e) => {
                log::error!("发送失败: {:?}", e)
            }
        };
    }

    pub fn info(&mut self, value: AttrValue) {
        self.notifications.push(Notification {
            type_: NotificationType::Info,
            title: AttrValue::from("INFO"),
            content: value,
        });
    }

    pub fn warn(&mut self, value: AttrValue) {
        self.notifications.push(Notification {
            type_: NotificationType::Info,
            title: AttrValue::from("WARN"),
            content: value,
        });
    }

    pub fn error(&mut self, value: AttrValue) {
        self.notifications.push(Notification {
            type_: NotificationType::Error,
            title: AttrValue::from("ERROR"),
            content: value,
        });
    }

    pub fn notify(&mut self, notify: Notification) {
        match notify.type_ {
            NotificationType::Info => self.info(notify.content),
            // NotificationType::Success => {}
            NotificationType::Warn => self.warn(notify.content),
            NotificationType::Error => self.error(notify.content),
        }
    }

    pub fn handle_friendship_req(
        &mut self,
        ctx: &Context<Self>,
        friendship: FriendShipWithUser,
    ) -> bool {
        log::debug!("ReceiveFriendShipReq:{:?}", &friendship);
        let id = friendship.friendship_id.clone().to_string();
        let state = Rc::make_mut(&mut self.friend_ship_state);
        state.ship = Some(friendship.clone());
        state.state_type = FriendShipStateType::Req;
        // 入库
        ctx.link().send_future(async move {
            FriendShipRepo::new()
                .await
                .put_friendship(&friendship)
                .await;
            // 发送收到通知
            HomeMsg::SendBackMsg(Msg::FriendshipDeliveredNotice(id))
        });
        // 显示通知
        // self.info(AttrValue::from("收到好友请求"));
        ctx.link().send_message(HomeMsg::Notification(Notification {
            type_: NotificationType::Info,
            title: AttrValue::default(),
            content: AttrValue::from("收到好友请求"),
        }));
        true
    }
    pub fn handle_friendship_res(
        &mut self,
        ctx: &Context<Self>,
        friendship_id: AttrValue,
        friend: Friend,
    ) -> bool {
        log::debug!("同意好友添加请求消息:{:?}", &friend);
        let state = Rc::make_mut(&mut self.friend_ship_state);
        state.friend = Some(friend.clone());
        state.state_type = FriendShipStateType::Res;

        let send_id = self.state.login_user.id.clone();
        // 入库
        ctx.link().send_future(async move {
            FriendShipRepo::new().await.agree(friendship_id).await;
            FriendRepo::new().await.put_friend(&friend).await;
            let mut msg = Message {
                msg_id: nanoid::nanoid!().into(),
                send_id,
                friend_id: friend.friend_id.clone(),
                content_type: ContentType::Text,
                content: friend
                    .hello
                    .clone()
                    .unwrap_or_else(|| AttrValue::from(DEFAULT_HELLO_MESSAGE)),
                create_time: chrono::Local::now().timestamp_millis(),
                is_read: true,
                is_self: true,
                file_content: AttrValue::default(),
                id: 0,
            };
            let _ = MessageRepo::new()
                .await
                .add_message(&mut msg)
                .await
                .map_err(|err| log::error!("添加好友打招呼消息入库失败:{:?}", err));
            log::debug!("发送打招呼:{:?}", &msg);
            HomeMsg::SendMessage(Msg::Single(msg))
        });
        true
    }

    pub fn handle_receive_message(&mut self, ctx: &Context<Self>, mut message: Msg) -> bool {
        match message {
            Msg::Single(ref mut msg) => {
                let friend_id = msg.send_id.clone();
                msg.send_id = msg.friend_id.clone();
                msg.friend_id = friend_id;
                msg.is_read = false;

                let mut msg = msg.clone();
                let msg_id = msg.msg_id.to_string();
                if self.conv_state.conv.item_id != msg.friend_id {
                    let conv_state = Rc::make_mut(&mut self.conv_state);
                    let _ = current_item::save_conv(&conv_state.conv)
                        .map_err(|err| log::error!("save conv fail{:?}", err));
                }
                ctx.link().send_future(async move {
                    // 数据入库
                    if let Err(err) = MessageRepo::new().await.add_message(&mut msg).await {
                        HomeMsg::Notification(Notification::error_from_content(
                            format!("内部错误:{:?}", err).into(),
                        ))
                    } else {
                        HomeMsg::SendBackMsg(Msg::SingleDeliveredNotice(msg_id))
                    }
                });

                ctx.link()
                    .send_message(HomeMsg::RecSendMsgStateChange(message));
            }
            Msg::Group(ref msg) => {
                let msg = msg.clone();
                let msg_id = msg.msg_id.to_string();
                if self.conv_state.conv.item_id != msg.friend_id {
                    let conv_state = Rc::make_mut(&mut self.conv_state);
                    let _ = current_item::save_conv(&conv_state.conv)
                        .map_err(|err| log::error!("save conv fail{:?}", err));
                }
                ctx.link().send_future(async move {
                    // 数据入库
                    if let Err(err) = GroupMsgRepo::new().await.put(&msg).await {
                        HomeMsg::Notification(Notification::error_from_content(
                            format!("内部错误:{:?}", err).into(),
                        ))
                    } else {
                        HomeMsg::SendBackMsg(Msg::SingleDeliveredNotice(msg_id))
                    }
                });

                ctx.link()
                    .send_message(HomeMsg::RecSendMsgStateChange(message));
            }
            Msg::SendRelationshipReq(_msg) => {}
            Msg::RecRelationship(msg) => {
                // 收到好友请求
                ctx.link().send_message(HomeMsg::ReceiveFriendShipReq(msg));
            }
            Msg::ReadNotice(_) | Msg::SingleDeliveredNotice(_) => {}
            Msg::OfflineSync(_) => {}
            Msg::SingleCall(m) => {
                // 保存电话信息，通知phone call组件
                self.call_msg = m;
                return true;
            }
            Msg::FriendshipDeliveredNotice(_) => {}
            Msg::RelationshipRes(friend) => {
                // 收到好友同意消息
                self.info(AttrValue::from("好友同意"));
                let send_id = self.state.login_user.id.clone();
                // 需要通知联系人列表更新
                // 数据入库
                let cloned_ctx = ctx.link().clone();
                ctx.link().send_future(async move {
                    FriendShipRepo::new()
                        .await
                        .agree_by_friend_id(friend.friend_id.clone())
                        .await;
                    FriendRepo::new().await.put_friend(&friend).await;
                    // send received message
                    cloned_ctx.send_message(HomeMsg::SendBackMsg(Msg::FriendshipDeliveredNotice(
                        friend.id.to_string(),
                    )));
                    // send hello message
                    let mut msg = Message {
                        msg_id: nanoid::nanoid!().into(),
                        send_id,
                        friend_id: friend.friend_id.clone(),
                        content_type: ContentType::Text,
                        content: friend
                            .hello
                            .unwrap_or_else(|| AttrValue::from(DEFAULT_HELLO_MESSAGE)),
                        create_time: chrono::Local::now().timestamp_millis(),
                        is_read: true,
                        is_self: true,
                        file_content: AttrValue::default(),
                        id: 0,
                    };
                    let _ = MessageRepo::new()
                        .await
                        .add_message(&mut msg)
                        .await
                        .map_err(|err| log::error!("save message fail:{:?}", err));
                    HomeMsg::SendMessage(Msg::Single(msg))
                });
            }
            Msg::GroupInvitation(_) => {
                // receive create group message
                ctx.link()
                    .send_message(HomeMsg::RecSendMsgStateChange(message));
            }
            Msg::GroupInvitationReceived(_) => {}
        }
        false
    }
}

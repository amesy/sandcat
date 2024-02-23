#![allow(dead_code)]

use std::rc::Rc;
use yew::prelude::*;

use crate::components::left::add_friend::AddFriend;
use crate::db::friend_ship::FriendShipRepo;
use crate::pages::{CurrentItem, FriendListState, FriendShipState};
use crate::{
    components::{left::list_item::ListItem, top_bar::TopBar},
    db::{friend::FriendRepo, RightContentType},
    model::friend::Friend,
    pages::{CommonProps, ComponentType},
};

#[derive(Properties, PartialEq, Debug)]
pub struct ContactsProps {
    pub user_id: AttrValue,
}

pub struct Contacts {
    pub list: Vec<Friend>,
    // 维护一个查询结果集
    pub result: Vec<Friend>,
    pub friendships_unread_count: usize,
    // 是否正在搜索
    pub is_searching: bool,
    pub is_add_friend: bool,
    pub friendship_state: Rc<FriendShipState>,
    pub _listener: ContextHandle<Rc<FriendShipState>>,
    pub friend_state: Rc<FriendListState>,
    pub _friend_listener: ContextHandle<Rc<FriendListState>>,
}

pub enum QueryState<T> {
    Querying,
    Success(T),
    Failure,
}

pub enum ContactsMsg {
    FilterContact(AttrValue),
    CleanupSearchResult,
    QueryFriends(QueryState<Vec<Friend>>),
    AddFriend(bool),
    RecFriendShipReq(Rc<FriendShipState>),
    FriendListStateChanged(Rc<FriendListState>),
    QueryFriendship(usize),
    NewFriendClicked,
}

impl Component for Contacts {
    type Message = ContactsMsg;

    type Properties = ContactsProps;

    fn create(ctx: &Context<Self>) -> Self {
        // 查询联系人列表
        ctx.link().send_future(async {
            let friend_repo = FriendRepo::new().await;
            let friends = friend_repo.get_list().await;

            ContactsMsg::QueryFriends(QueryState::Success(friends))
        });
        // 查询好友请求列表
        ctx.link().send_future(async {
            let friend_repo = FriendShipRepo::new().await;
            let count = friend_repo.get_unread_count().await;
            log::debug!("查询好友请求列表, 未读数量{}", count);
            ContactsMsg::QueryFriendship(count)
        });
        let (friendship_state, _listener) = ctx
            .link()
            .context(ctx.link().callback(ContactsMsg::RecFriendShipReq))
            .expect("need friend ship state");
        let (friend_state, _friend_listener) = ctx
            .link()
            .context(ctx.link().callback(ContactsMsg::FriendListStateChanged))
            .expect("need friend ship state");
        Self {
            list: vec![],
            result: vec![],
            friendships_unread_count: 0,
            is_searching: false,
            is_add_friend: false,
            friendship_state,
            _listener,
            friend_state,
            _friend_listener,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ContactsMsg::FilterContact(pattern) => {
                self.is_searching = true;
                // 过滤联系人列表
                if pattern.is_empty() {
                    self.result.clear();
                } else {
                    self.list.iter().for_each(|item| {
                        if item.name.contains(pattern.as_str()) {
                            self.result.push((*item).clone());
                        }
                    });
                }
                true
            }
            // 清空搜索结果
            ContactsMsg::CleanupSearchResult => {
                gloo::console::log!("filter contacts clean");
                self.is_searching = false;
                self.result.clear();
                true
            }
            ContactsMsg::QueryFriends(friends) => match friends {
                QueryState::Success(list) => {
                    self.list = list;
                    true
                }
                QueryState::Failure => {
                    gloo::console::log!("query friends failure");
                    false
                }
                QueryState::Querying => {
                    gloo::console::log!("query friends querying");
                    false
                }
            },
            ContactsMsg::AddFriend(is_add) => {
                self.is_add_friend = is_add;
                true
            }
            ContactsMsg::RecFriendShipReq(friendship) => {
                // self.list.insert(0, friendship.friend.clone());
                match friendship.state_type {
                    crate::pages::FriendShipStateType::Req => {
                        self.friendships_unread_count += 1;
                    }
                    crate::pages::FriendShipStateType::Res => {
                        self.list
                            .insert(0, friendship.friend.as_ref().unwrap().clone());
                    }
                }
                true
            }
            ContactsMsg::QueryFriendship(count) => {
                self.friendships_unread_count = count;
                true
            }
            ContactsMsg::FriendListStateChanged(state) => {
                self.friend_state = state;
                true
            }
            ContactsMsg::NewFriendClicked => {
                log::debug!("new friend clicked");
                self.friendships_unread_count = 0;
                self.friend_state.state_change_event.emit(CurrentItem {
                    item_id: AttrValue::default(),
                    content_type: RightContentType::FriendShipList,
                    unread_count: 0,
                });
                true
            }
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        // 根据搜索结果显示联系人列表，
        // 如果是搜索状态，那么搜索结果为空时需要提示用户没有结果
        let content = if self.is_searching {
            if self.result.is_empty() {
                html! {<div class="no-result">{"没有搜索结果"}</div>}
            } else {
                self.result
                    .iter()
                    .map(|item| get_list_item(item))
                    .collect::<Html>()
            }
        } else {
            self.list
                .iter()
                .map(|item| get_list_item(item))
                .collect::<Html>()
        };
        let search_callback = ctx.link().callback(ContactsMsg::FilterContact);
        let clean_callback = ctx
            .link()
            .callback(move |_| ContactsMsg::CleanupSearchResult);
        let plus_click = ctx.link().callback(ContactsMsg::AddFriend);
        let friendship_click = ctx.link().callback(|_| ContactsMsg::NewFriendClicked);

        html! {
            <div class="list-wrapper">
                {
                    if self.is_add_friend {
                        html!{
                            <AddFriend user_id={ctx.props().user_id.clone()} {plus_click}/>
                        }
                    } else {
                        html!{
                            <>
                                <TopBar components_type={ComponentType::Contacts} {search_callback} {clean_callback} {plus_click} />
                                <div class="contacts-list">
                                    <div class="new-friends" onclick={friendship_click}>
                                        {"新的朋友: "}
                                        if self.friendships_unread_count > 0{
                                            {self.friendships_unread_count}
                                        }
                                    </div>
                                    {content}
                                </div>
                            </>
                        }
                    }
                }
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {}
    }
}

fn get_list_item(item: &Friend) -> Html {
    html! {
        <ListItem
            component_type={ComponentType::Contacts}
            props={
                CommonProps{
                    name:item.name.clone(),
                    avatar:item.avatar.clone(),
                    time:item.create_time.clone().timestamp_millis(),
                    remark:item.remark.clone().unwrap_or_default().clone(),
                    id: item.friend_id.clone()
                }
            }
            unread_count={0}
            conv_type={RightContentType::Friend}
            key={item.friend_id.clone().as_str()} />
    }
}
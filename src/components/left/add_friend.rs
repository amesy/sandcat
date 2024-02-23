#![allow(dead_code)]
use yew::prelude::*;

use crate::api::user::search_friend;
use crate::components::left::user_info::UserInfoCom;
use crate::model::user::User;
use crate::{components::top_bar::TopBar, pages::ComponentType};

#[derive(Properties, PartialEq, Debug)]
pub struct AddFriendProps {
    pub plus_click: Callback<bool>,
    pub user_id: AttrValue,
}

pub struct AddFriend {
    // 维护一个查询结果集
    pub result: Vec<User>,
    // 是否正在搜索
    pub is_searching: bool,
}

pub enum QueryState<T> {
    Querying,
    Success(T),
    Failure,
}

pub enum AddFriendMsg {
    SearchFriend(AttrValue),
    CleanupSearchResult,
    QueryFriends(QueryState<Vec<User>>),
    Cancel(bool),
}

impl Component for AddFriend {
    type Message = AddFriendMsg;

    type Properties = AddFriendProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            result: vec![],
            is_searching: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AddFriendMsg::SearchFriend(pattern) => {
                self.is_searching = true;
                let user_id = ctx.props().user_id.clone();
                ctx.link().send_future(async move {
                    match search_friend(pattern.to_string(), user_id).await {
                        Ok(list) => AddFriendMsg::QueryFriends(QueryState::Success(list)),
                        Err(err) => {
                            log::error!("搜索用户错误:{:?}", err);
                            AddFriendMsg::QueryFriends(QueryState::Failure)
                        }
                    }
                });
                true
            }
            // 清空搜索结果
            AddFriendMsg::CleanupSearchResult => {
                gloo::console::log!("filter contacts clean");
                self.is_searching = false;
                self.result.clear();
                true
            }
            AddFriendMsg::QueryFriends(friends) => match friends {
                QueryState::Success(list) => {
                    self.result = list;
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
            AddFriendMsg::Cancel(_) => {
                ctx.props().plus_click.emit(false);
                false
            }
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let content = if self.result.is_empty() {
            html! {<div class="no-result">{"没有搜索结果"}</div>}
        } else {
            self.result
                .iter()
                .map(|item| {
                    html! {
                        <UserInfoCom info={item.clone()} key={item.id.clone().as_str()} />
                    }
                })
                .collect::<Html>()
        };
        let search_callback = ctx.link().callback(AddFriendMsg::SearchFriend);
        let clean_callback = ctx
            .link()
            .callback(move |_| AddFriendMsg::CleanupSearchResult);
        let plus_click = ctx.link().callback(AddFriendMsg::Cancel);

        html! {
            <>
            // <div class="list-wrapper">
                <TopBar components_type={ComponentType::Setting} {search_callback} {clean_callback} {plus_click} />
                <div class="contacts-list">
                    {content}
                </div>
            // </div>
            </>
        }
    }
}
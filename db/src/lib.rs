use impls::indexed_db;
pub use impls::indexed_db::*;
use once_cell::sync::OnceCell;

use self::{
    conv::ConvRepo,
    conversations::Conversations,
    friend::FriendRepo,
    friend_ship::FriendShipRepo,
    friends::Friends,
    friendships::Friendships,
    group::GroupRepo,
    group_members::GroupMembers,
    group_msg::GroupMessages,
    groups::GroupInterface,
    impls::indexed_db::{group_members::GroupMembersRepo, group_msg::GroupMsgRepo, seq::SeqRepo},
    message::MessageRepo,
    messages::Messages,
    seq::SeqInterface,
    user::UserRepo,
    users::Users,
};

pub mod conversations;
pub mod friends;
pub mod friendships;
pub mod group_members;
pub mod group_msg;
pub mod groups;
pub mod impls;
pub mod messages;
pub mod seq;
pub mod users;

static DB_INSTANCE: OnceCell<Db> = OnceCell::new();

pub fn db_ins() -> &'static Db {
    DB_INSTANCE.get().unwrap()
}

pub async fn init_db() {
    if DB_INSTANCE.get().is_some() {
        return;
    }
    let db = Db::new().await;
    if let Err(err) = DB_INSTANCE.set(db) {
        log::error!("{:?}", err);
    }
}

unsafe impl Sync for Db {}
unsafe impl Send for Db {}
#[derive(Debug)]
pub struct Db {
    pub convs: Box<dyn Conversations>,
    pub groups: Box<dyn GroupInterface>,
    pub friends: Box<dyn Friends>,
    pub friendships: Box<dyn Friendships>,
    pub group_members: Box<dyn GroupMembers>,
    pub messages: Box<dyn Messages>,
    pub group_msgs: Box<dyn GroupMessages>,
    pub users: Box<dyn Users>,
    pub seq: Box<dyn SeqInterface>,
}

impl Db {
    pub async fn new() -> Self {
        let repo = indexed_db::repository::Repository::new().await;
        Self {
            convs: Box::new(ConvRepo::new(repo.clone())),
            groups: Box::new(GroupRepo::new(repo.clone())),
            friends: Box::new(FriendRepo::new(repo.clone())),
            friendships: Box::new(FriendShipRepo::new(repo.clone())),
            group_members: Box::new(GroupMembersRepo::new(repo.clone())),
            messages: Box::new(MessageRepo::new(repo.clone())),
            group_msgs: Box::new(GroupMsgRepo::new(repo.clone())),
            users: Box::new(UserRepo::new(repo.clone())),
            seq: Box::new(SeqRepo::new(repo)),
        }
    }
}
/*
pub async fn convs() -> Box<dyn Conversations> {
    Box::new(ConvRepo::new().await)
}

pub async fn groups() -> Box<dyn GroupInterface> {
    Box::new(GroupRepo::new().await)
}

pub async fn friends() -> Box<dyn Friends> {
    Box::new(FriendRepo::new().await)
}

pub async fn friendships() -> Box<dyn Friendships> {
    Box::new(FriendShipRepo::new().await)
}

pub async fn group_members() -> Box<dyn GroupMembers> {
    Box::new(GroupMembersRepo::new().await)
}

pub async fn messages() -> Box<dyn Messages> {
    Box::new(MessageRepo::new().await)
}

pub async fn group_msgs() -> Box<dyn GroupMessages> {
    Box::new(GroupMsgRepo::new().await)
}

pub async fn users() -> Box<dyn Users> {
    Box::new(UserRepo::new().await)
}

pub async fn seq() -> Box<dyn SeqInterface> {
    Box::new(SeqRepo::new().await)
}
 */

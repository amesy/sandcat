use serde::{Deserialize, Serialize};
use yew::AttrValue;

use super::{friend::Friend, ItemInfo, ItemType};

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Group {
    pub id: AttrValue,
    pub name: AttrValue,
    pub avatar: AttrValue,
    pub members_id: Vec<String>,
    pub create_time: i64,
    pub publish_msg: AttrValue,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct GroupRequest {
    pub group_name: String,
    pub members_id: Vec<String>,
}
/// Group member information
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GroupMember {
    pub id: i64,
    pub user_id: AttrValue,
    pub group_id: AttrValue,
    pub name: AttrValue,
    pub group_name: Option<AttrValue>,
    pub avatar: AttrValue,
    pub region: Option<AttrValue>,
}

impl From<Friend> for GroupMember {
    fn from(value: Friend) -> Self {
        Self {
            id: 0,
            user_id: value.friend_id,
            group_id: AttrValue::default(),
            name: value.name,
            group_name: None,
            avatar: value.avatar,
            region: value.address,
        }
    }
}

impl ItemInfo for Group {
    fn name(&self) -> AttrValue {
        self.name.clone()
    }

    fn id(&self) -> AttrValue {
        self.id.clone()
    }

    fn get_type(&self) -> ItemType {
        ItemType::Group
    }

    fn avatar(&self) -> AttrValue {
        self.avatar.clone()
    }
}
use abi::model::{
    message::{Message, SendStatus},
    ContentType,
};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use yew::AttrValue;

use crate::AppState;

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize, Default)]
pub struct Msg {
    pub id: i32,
    pub seq: i64,
    pub local_id: String,
    pub server_id: String,
    pub send_id: String,
    pub friend_id: String,
    pub content_type: u8,
    pub content: String,
    pub create_time: i64,
    pub send_time: i64,
    pub send_status: u8,
    pub is_read: u8,
    pub is_self: bool,
}

impl Into<Message> for Msg {
    fn into(self) -> Message {
        Message {
            local_id: self.local_id.into(),
            server_id: self.server_id.into(),
            send_id: self.send_id.into(),
            friend_id: self.friend_id.into(),
            content_type: ContentType::from(self.content_type),
            content: self.content.into(),
            create_time: self.create_time,
            send_time: self.send_time,
            send_status: SendStatus::from(self.send_status),
            is_read: self.is_read,
            id: self.id,
            seq: self.seq,
            is_self: self.is_self,
            file_content: AttrValue::default(),
        }
    }
}

#[tauri::command]
pub async fn get_messages(
    friend_id: String,
    page: u32,
    page_size: u32,
    state: tauri::State<'_, AppState>,
) -> Result<IndexMap<AttrValue, Message>, String> {
    let offset = (page - 1) * page_size;
    // concat the table name
    let table_name = format!("msg_{}", friend_id);
    let messages: Vec<Msg> =
        sqlx::query_as("SELECT * FROM $1 ORDER BY send_time DESC LIMIT $2 OFFSET $3")
            .bind(&table_name)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| e.to_string())?;
    let mut map = IndexMap::with_capacity(page_size as usize);
    for msg in messages {
        map.insert(msg.local_id.clone().into(), msg.into());
    }
    Ok(map)
}

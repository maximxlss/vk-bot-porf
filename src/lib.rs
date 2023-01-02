#![feature(never_type)]

pub mod error;

use parking_lot::Mutex;
use rvk::{APIClient, Params};
use rvk_methods::groups::get_long_poll_server;
use std::future::Future;
use std::time::{Duration, UNIX_EPOCH};
use lazy_static::lazy_static;
use rvk_objects::message::Message;
use rvk_objects::user::User;
use serde_json::Value;


#[macro_export]
macro_rules! param_err {
    () => { rvk::error::Error::from("Failed to get a parameter") }
}

const CONTEXT_SYMBOL_LIMIT: usize = 500;

lazy_static! {
    static ref REQWEST_CLIENT: reqwest::Client = reqwest::Client::new();
    static ref API_CLIENT: APIClient = rvk_methods::supported_api_client(include_str!("token"));
    static ref CUR_SELF_ID: Mutex<Option<i32>> = Mutex::new(None);
    static ref LAST_AUTHOR_ID: Mutex<i32> = Mutex::new(-1);
    static ref MESSAGE_HISTORY: Mutex<String> = Mutex::new(String::new());
}

pub fn clone_history() -> String {
    (*MESSAGE_HISTORY.lock()).clone()
}

pub fn clear_history() {
    MESSAGE_HISTORY.lock().clear();
    *LAST_AUTHOR_ID.lock() = -1;
}

pub async fn get_self_id() -> i32 {
    if CUR_SELF_ID.lock().is_none() {
        *CUR_SELF_ID.lock() = Some(get_this_group_id().await)
    }
    CUR_SELF_ID.lock().unwrap()
}

fn add_string_to_history(s: &str) {
    let mut history = MESSAGE_HISTORY.lock();
    history.push_str(s);
    while history.len() > CONTEXT_SYMBOL_LIMIT {
        history.remove(0);
    }
}

async fn add_msg_to_history(msg: &TextMessage) {
    add_msg_to_history_raw(msg.from_id, &msg.text).await
}

async fn add_msg_to_history_raw(from_id: i32, text: &str) {
    if from_id != *LAST_AUTHOR_ID.lock() {
        let name = if from_id == -1 {
            "Порфирьевич".to_string()
        } else {
            let user = get_user(from_id).await;
            user.first_name + " " + &user.last_name
        };
        add_string_to_history(&format!("{name}:\n"));
        *LAST_AUTHOR_ID.lock() = from_id as i32;
    }
    add_string_to_history(&format!("- {text}\n"));
}

fn cur_unix_time() -> u64 {
    std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

pub async fn send_message(peer_id: i32, msg: &str) -> u32 {
    add_msg_to_history_raw(-1, msg).await;
    let mut params = Params::new();
    params.insert("peer_id".to_string(), peer_id.to_string());
    params.insert("message".to_string(), msg.to_string());
    let random_id = cur_unix_time() % 2147483648;
    params.insert("random_id".to_string(), random_id.to_string());
    rvk_methods::messages::send::<u32>(&API_CLIENT, params).await.expect("failed to send")
}

pub async fn set_typing(peer_id: i32) {
    let mut params = Params::new();
    params.insert("peer_id".to_string(), peer_id.to_string());
    params.insert("type".to_string(), "typing".to_string());
    rvk_methods::messages::set_activity::<Value>(&API_CLIENT, params).await.expect("failed to set activity");
}

pub async fn get_user(user_id: i32) -> User {
    let mut params = Params::new();
    params.insert("user_ids".to_string(), user_id.to_string());
    let users = rvk_methods::users::get::<Vec<User>>(&API_CLIENT, params).await.expect("failed to get users");
    users[0].clone()
}

async fn get_this_group_id() -> i32 {
    let mut params = Params::new();
    params.insert("".to_string(), "".to_string());
    let groups = rvk_methods::groups::get_by_id::<Vec<Value>>(&API_CLIENT, params).await.expect("failed to get group");
    groups[0]["id"].as_i64().unwrap() as i32
}

pub async fn poll_for_messages<F, Fut>(callback: F) -> rvk::error::Result<!>
where
    F: Fn(TextMessage) -> Fut,
    Fut: Future<Output = ()> + Send + 'static,
{
    let mut params = Params::new();
    params.insert("group_id".to_string(), "217990903".to_string());
    let res = get_long_poll_server::<Params>(&API_CLIENT, params).await?;
    let server = res.get("server").ok_or(param_err!())?.clone();
    let key = res.get("key").ok_or(param_err!())?.clone();
    let mut ts = res.get("ts").ok_or(param_err!())?.clone();

    loop {
        let body: Value = serde_json::from_str(&reqwest::get(format!("{server}?act=a_check&key={key}&ts={ts}&wait=25"))
            .await?
            .text()
            .await?)?;

        ts = body["ts"].as_str().ok_or(param_err!())?.to_string();
        let updates = body["updates"].as_array().ok_or(param_err!())?;
        for update in updates {
            let t = update["type"].as_str().ok_or(param_err!())?;
            if t == "message_new" {
                let msg: Message = serde_json::from_value(update["object"]["message"].clone())?;
                let Ok(msg): Result<TextMessage, &'static str> = msg.try_into() else { continue; };
                add_msg_to_history(&msg).await;
                tokio::spawn(callback(msg));
            } else {
                println!("{} - unknown event", t);
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}


pub struct TextMessage {
    pub peer_id: i32,
    pub from_id: i32,
    pub text: String
}


impl TryFrom<Message> for TextMessage {
    type Error = &'static str;

    fn try_from(msg: Message) -> Result<Self, Self::Error> {
        let Some(text) = msg.text else {
            return Err("Message contains no text");
        };
        let Some(peer_id) = msg.peer_id else {
            return Err("Message contains no text");
        };
        Ok(TextMessage {
            peer_id: peer_id as i32,
            from_id: msg.from_id as i32,
            text
        })
    }
}

impl TextMessage {
    pub async fn reply(&self, text: &str) -> u32 {
        send_message(self.peer_id, text).await
    }
}

pub async fn porfirevich_get(prompt: &str, length: u32) -> Result<String, error::Error> {
    let mut body = String::with_capacity(prompt.len() + 50);
    body.push_str("{\"prompt\":\"");
    body.push_str(&prompt.replace("\\", "\\\\").replace("\"", "\\\""));
    body.push_str("\",\"length\":");
    body.push_str(&length.to_string());
    body.push_str("}");
    body = body.replace("\n", "\\n");
    let res: Value = serde_json::from_str(&REQWEST_CLIENT.post("https://pelevin.gpt.dobro.ai/generate/")
        .header("Content-Type", "application/json")
        .body(body)
        .send().await?.text().await?)?;
    let Some(ans) = res["replies"][0].as_str() else {
        let err = "weird response from porfirevich: {}".to_string();
        println!("{err}"); return Err(err.into());
    };

    Ok(ans.to_string())
}
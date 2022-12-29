#![feature(never_type, async_closure)]

use parking_lot::Mutex;
use std::time::UNIX_EPOCH;
use rvk::Params;
use rvk::APIClient;
use rvk_objects::Integer;
use rvk_objects::message::Message;
use vk_bot_porf::{poll_for_messages, porfirevich_get};
use lazy_static::lazy_static;
use serde_json::Value;
use rand::prelude::*;
use rvk_objects::user::User;


const CONTEXT_SYMBOL_LIMIT: usize = 350;
const HELP_TEXT: &'static str = r#"
Команды:
- /p -- сгенерировать текст. Например: `/p он сказал`
- /c -- установить шанс случайных сообщений в процентах. Например: `/c 25`. По умолчанию: 10%.
- /l -- установить длину дополнения в словах. Например: `/l 30`. По умолчанию: 60 слов.
- /h -- вывести это сообщение.
- /r -- сбросить историю сообщений в памяти бота.
"#;


lazy_static!{
    static ref API_CLIENT: APIClient = rvk_methods::supported_api_client(include_str!("token"));
    static ref LENGTH: Mutex<u32> = Mutex::new(60);
    static ref CHANCE: Mutex<u32> = Mutex::new(10);
    static ref LAST_AUTHOR_ID: Mutex<i32> = Mutex::new(-1);
    static ref MESSAGE_HISTORY: Mutex<String> = Mutex::new(String::new());
}


async fn send_reply(ctx: &Message, msg: &str) -> u32 {
    let peer_id = ctx.peer_id.expect("failed to extract peer_id");
    send_message(peer_id, msg).await
}

async fn send_message(peer_id: Integer, msg: &str) -> u32 {
    let mut params = Params::new();
    params.insert("peer_id".to_string(), peer_id.to_string());
    params.insert("message".to_string(), msg.to_string());
    let random_id = std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() % 2147483648;
    params.insert("random_id".to_string(), random_id.to_string());
    rvk_methods::messages::send::<u32>(&API_CLIENT, params).await.expect("failed to send")
}

async fn set_typing(peer_id: Integer) {
    let mut params = Params::new();
    params.insert("peer_id".to_string(), peer_id.to_string());
    params.insert("type".to_string(), "typing".to_string());
    rvk_methods::messages::set_activity::<Value>(&API_CLIENT, params).await.expect("failed to set activity");
}

async fn rand_ans(msg: &Message) {
    set_typing(msg.peer_id.unwrap()).await;
    let cur_length = {*LENGTH.lock()};
    let context = {
        let mut history = MESSAGE_HISTORY.lock();
        while history.len() > CONTEXT_SYMBOL_LIMIT {
            history.remove(0);
        }
        history.push_str("Порфирьевич:\n- ");
        history.clone()
    };
    {*LAST_AUTHOR_ID.lock() = -1};
    println!("{context}");
    if let Ok(ans) = porfirevich_get(&context, cur_length).await {
        send_reply(&msg, &ans[context.len()..]).await;
        let mut history = MESSAGE_HISTORY.lock();
        history.push_str(&ans[context.len()..].trim());
        history.push('\n');
    } else {
        send_reply(&msg, "Ошибка!").await;
    }
}

#[tokio::main]
async fn main() -> Result<!, rvk::error::Error> {
    poll_for_messages(&API_CLIENT, async move |msg| {
        let Some(text) = msg.text.clone() else {
            println!("failed to extract text"); return;
        };
        if text.starts_with("/p") {
            let peer_id = msg.peer_id.expect("failed to extract peer_id");
            set_typing(peer_id).await;
            let cur_length = {*LENGTH.lock()};
            if let Ok(ans) = porfirevich_get(&text[2..], cur_length).await {
                send_reply(&msg, &ans).await;
            } else {
                send_reply(&msg, "Ошибка!").await;
            }
        } else if text.starts_with("/c ") {
            let Ok(value) = text[3..].parse() else {
                send_reply(&msg, "Ты нормально пиши то").await; return
            };
            *CHANCE.lock() = value;
            send_reply(&msg, &format!("Установлен шанс в {value}%")).await;
        } else if text.starts_with("/l ") {
            let Ok(value) = text[3..].parse() else {
                send_reply(&msg, "Ты нормально пиши то").await; return
            };
            *LENGTH.lock() = value;
            send_reply(&msg, &format!("Установлена длина в {value} слов")).await;
        } else if text.starts_with("/h") {
            let cur_length = {*LENGTH.lock()};
            let cur_chance = {*CHANCE.lock()};
            send_reply(&msg, &(format!("Текущая длина: {cur_length} слов\nТекущий шанс: {cur_chance}%") + HELP_TEXT)).await;
        } else if text.starts_with("/r") {
            MESSAGE_HISTORY.lock().clear();
            *LAST_AUTHOR_ID.lock() = -1;
            send_reply(&msg, "История сообщений сброшена").await;
        } else {
            if msg.from_id != {*LAST_AUTHOR_ID.lock()} as i64 {
                let mut params = Params::new();
                params.insert("user_ids".to_string(), msg.from_id.to_string());
                let users = rvk_methods::users::get::<Vec<User>>(&API_CLIENT, params).await.expect("failed to get users");
                {
                    let mut history = MESSAGE_HISTORY.lock();
                    history.push_str(&users[0].first_name); history.push(' '); history.push_str(&users[0].last_name); history.push(':');
                    history.push('\n');
                }
                {*LAST_AUTHOR_ID.lock() = msg.from_id as i32;}
            }
            {let mut history = MESSAGE_HISTORY.lock();
            history.push_str("- ");
            history.push_str(&text);
            history.push('\n');}
            let random_value = {
                let mut rng = rand::thread_rng();
                rng.gen_range(1..=100)
            };
            let cur_chance = {*CHANCE.lock()};
            if random_value <= cur_chance {
                rand_ans(&msg).await;
            }
        }
    }).await
}

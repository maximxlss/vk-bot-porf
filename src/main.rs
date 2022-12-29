#![feature(never_type, async_closure)]

use parking_lot::Mutex;
use vk_bot_porf::*;
use lazy_static::lazy_static;
use rand::prelude::*;


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
    static ref LENGTH: Mutex<u32> = Mutex::new(60);
    static ref CHANCE: Mutex<u32> = Mutex::new(10);
    static ref LAST_AUTHOR_ID: Mutex<i32> = Mutex::new(-1);
    static ref MESSAGE_HISTORY: Mutex<String> = Mutex::new(String::new());
}

fn get_history() -> String {
    (*MESSAGE_HISTORY.lock()).clone()
}

fn add_string_to_history(s: &str) {
    let mut history = MESSAGE_HISTORY.lock();
    history.push_str(s);
    while history.len() > CONTEXT_SYMBOL_LIMIT {
        history.remove(0);
    }
}

async fn add_msg_to_history(msg: &TextMessage) {
    if msg.from_id != *LAST_AUTHOR_ID.lock() {
        let user = get_user(msg.from_id).await;
        let name = user.first_name + " " + &user.last_name;
        add_string_to_history(&format!("{name}:\n"));
        *LAST_AUTHOR_ID.lock() = msg.from_id as i32;
    }
    let text = &msg.text;
    add_string_to_history(&format!("- {text}\n"));
}

async fn rand_ans(msg: &TextMessage) {
    set_typing(msg.peer_id).await;
    let cur_length = *LENGTH.lock();
    add_string_to_history("Порфирьевич:\n-");
    *LAST_AUTHOR_ID.lock() = -1;
    let context = get_history();
    println!("{context}");
    if let Ok(ans) = porfirevich_get(&context, cur_length).await {
        msg.reply(&ans).await;
        add_string_to_history(&ans);
    } else {
        msg.reply("Ошибка!").await;
    }
    *LAST_AUTHOR_ID.lock() = -1;
}

#[tokio::main]
async fn main() -> Result<!, rvk::error::Error> {
    poll_for_messages(async move |msg| {
        let Ok(msg): Result<TextMessage, &'static str> = msg.try_into() else { return; };
        // --- Generate text
        if msg.text.starts_with("/p ") {
            set_typing(msg.peer_id).await;
            let mut prompt = msg.text[3..].to_string();
            let cur_len = { *LENGTH.lock() };
            if let Ok(ans) = porfirevich_get(&prompt, cur_len).await {
                let res = msg.text[3..].to_string() + &ans;
                msg.reply(&res).await;
            } else {
                msg.reply("Ошибка!").await;
            }
        // --- Configuration commands
        } else if msg.text.starts_with("/c ") {
            let Ok(value) = msg.text[3..].parse() else {
                msg.reply("Ты нормально пиши то").await; return
            };
            *CHANCE.lock() = value;
            msg.reply(&format!("Установлен шанс в {value}%")).await;
        } else if msg.text.starts_with("/l ") {
            let Ok(value) = msg.text[3..].parse() else {
                msg.reply("Ты нормально пиши то").await; return
            };
            *LENGTH.lock() = value;
            msg.reply(&format!("Установлена длина в {value} слов")).await;
        } else if msg.text.starts_with("/h") {
            let cur_length = *LENGTH.lock();
            let cur_chance = *CHANCE.lock();
            msg.reply(&(format!("Текущая длина: {cur_length} слов\nТекущий шанс: {cur_chance}%") + HELP_TEXT)).await;
        } else if msg.text.starts_with("/r") {
            MESSAGE_HISTORY.lock().clear();
            *LAST_AUTHOR_ID.lock() = -1;
            msg.reply("История сообщений сброшена").await;
        // --- Random messages
        } else {
            add_msg_to_history(&msg).await;
            let random_value = {
                let mut rng = thread_rng();
                rng.gen_range(1..=100)
            };
            let cur_chance = *CHANCE.lock();
            if random_value <= cur_chance {
                set_typing(msg.peer_id).await;
                rand_ans(&msg).await;
            }
        }
    }).await
}

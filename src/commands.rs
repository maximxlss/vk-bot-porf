use vk_bot_porf::{clear_history, porfirevich_get, set_typing, TextMessage};
use crate::STORAGE;

const HELP_TEXT: &'static str = r#"
Команды:
- /p -- сгенерировать текст (`_` в конце меняется на ` `). Например: `/p он сказал`
- /c -- установить шанс случайных ответов от бота. Например: `/c 25`. По умолчанию: 10%.
- /l -- установить длину дополнения в словах. Например: `/l 30`. По умолчанию: 60 слов.
- /h -- вывести это сообщение.
- /r -- сбросить историю сообщений в памяти бота.
"#;

pub async fn handle_commands(msg: &TextMessage) -> bool {
    // --- Generate text
    if msg.text.starts_with("/p ") {
        set_typing(msg.peer_id).await;
        let mut prompt = msg.text[3..].to_string();
        if prompt.ends_with("_") {
            prompt.pop(); prompt.push(' ');
        }
        let len = STORAGE.lock().get_length();
        if let Ok(ans) = porfirevich_get(&prompt, len).await {
            let res = prompt + &ans;
            msg.reply(&res).await;
        } else {
            msg.reply("Ошибка!").await;
        }
    // --- Configuration commands
    } else if msg.text.starts_with("/c ") {
        let Ok(value) = msg.text[3..].parse() else {
            msg.reply("Ты нормально пиши то").await; return true;
        };
        STORAGE.lock().set_chance(value);
        msg.reply(&format!("Установлен шанс в {value}%")).await;
    } else if msg.text.starts_with("/l ") {
        let Ok(value) = msg.text[3..].parse() else {
            msg.reply("Ты нормально пиши то").await; return true;
        };
        STORAGE.lock().set_length(value);
        msg.reply(&format!("Установлена длина в {value} слов")).await;
    } else if msg.text.starts_with("/h") {
        let cur_length = STORAGE.lock().get_length();
        let cur_chance = STORAGE.lock().get_chance();
        msg.reply(&(format!("Текущая длина: {cur_length} слов\nТекущий шанс: {cur_chance}%") + HELP_TEXT)).await;
    } else if msg.text.starts_with("/r") {
        clear_history();
        msg.reply("История сообщений сброшена").await;
    } else { return false; };
    return true;
}

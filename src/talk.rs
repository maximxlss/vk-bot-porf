mod rules;

use vk_bot_porf::{porfirevich_get, set_typing, TextMessage};
use crate::STORAGE;

pub(crate) async fn handle_talking(msg: &TextMessage) {
    let mut context = String::with_capacity(500);
    let mut will_answer = false;
    will_answer |= rules::add_history(&mut context).await;
    will_answer |= rules::replace_bot_with_porfirevich(&mut context).await;
    will_answer |= rules::random_answers(&mut context).await;
    will_answer |= rules::mention_answer(&mut context).await;
    will_answer |= rules::add_prompt(&mut context).await;
    println!("{:?}", will_answer);
    println!("{:?}", context);
    if will_answer {
        set_typing(msg.peer_id).await;
        let len = STORAGE.lock().get_length();
        match porfirevich_get(&context, len).await {
            Ok(ans) => msg.reply(&ans).await,
            Err(e) => msg.reply(&format!("Ошибка: {:?}", e)).await
        };
    }
}

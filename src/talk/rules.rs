use rand::{Rng, thread_rng};
use vk_bot_porf::{clone_history, get_self_id};
use crate::STORAGE;

pub(crate) async fn add_history(context: &mut String) -> bool {
    context.push_str(&clone_history());
    false
}

pub(crate) async fn replace_bot_with_porfirevich(context: &mut String) -> bool {
    let mut occurence = context.to_lowercase().find("бот");
    while occurence.is_some() {
        let i = occurence.unwrap();
        let no_letter_before = i > 0 && !char_before(context, i).is_alphabetic();
        let no_letter_after = i < context.len() - 1 && !char_after(context, i + "бот".len()).is_alphabetic();
        if no_letter_before && no_letter_after {
            for _ in 0..3 { context.remove(i); }
            context.insert_str(i, "порфирьевич");
        }
        occurence = context.to_lowercase().find("бот");
    }
    false
}

pub(crate) async fn random_answers(context: &mut String) -> bool {
    let mut rng = thread_rng();
    let random_value = rng.gen_range(0..=100);
    random_value < STORAGE.lock().get_chance()
}

fn char_before(s: &str, byte_offset: usize) -> char {
    s[s.floor_char_boundary(byte_offset - 1)..].chars().next().unwrap()
}

fn char_after(s: &str, byte_offset: usize) -> char {
    s[s.ceil_char_boundary(byte_offset + 1)..].chars().next().unwrap()
}

pub(crate) async fn mention_answer(context: &mut String) -> bool {
    if context.contains(&get_self_id().await.to_string()) {
        return true;
    }
    context.to_lowercase().find("порфирьевич").is_some()
}

pub(crate) async fn add_prompt(context: &mut String) -> bool {
    context.push_str("Порфирьевич:\n-");
    false
}

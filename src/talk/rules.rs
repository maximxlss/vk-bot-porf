#![allow(unused_variables)]

use rand::{Rng, thread_rng};
use vk_bot_porf::{clone_history, get_self_id, TextMessage};
use crate::STORAGE;

pub(crate) async fn add_history(context: &mut String, msg: &mut TextMessage) -> bool {
    context.push_str(&clone_history());
    false
}

fn replace(s: &mut String, pat: &str, with: &str) {
    let mut offset = 0;
    let mut occurrence = s.to_lowercase().find(pat);
    while occurrence.is_some() && offset < s.len() && s.ceil_char_boundary(offset) < s.len() {
        let i = occurrence.unwrap() + s.ceil_char_boundary(offset);
        let there_is_a_letter_before = i > 0 && char_before(s, i).is_alphabetic();
        if !there_is_a_letter_before {
            for _ in 0..pat.chars().count() { s.remove(i); }
            s.insert_str(i, with);
        }
        offset = i + 1;
        occurrence = s[s.ceil_char_boundary(offset)..].to_lowercase().find(pat);
    }
}

pub(crate) async fn replace_bot_with_porfirevich(context: &mut String, msg: &mut TextMessage) -> bool {
    replace(context, "бот", "Порфирьевич");
    replace(&mut msg.text, "бот", "Порфирьевич");
    false
}

pub(crate) async fn random_answers(context: &mut String, msg: &mut TextMessage) -> bool {
    let mut rng = thread_rng();
    let random_value = rng.gen_range(0..=100);
    random_value < STORAGE.lock().get_chance()
}

fn char_before(s: &str, byte_offset: usize) -> char {
    s[s.floor_char_boundary(s.floor_char_boundary(byte_offset) - 1)..].chars().next().unwrap()
}

pub(crate) async fn mention_answer(context: &mut String, msg: &mut TextMessage) -> bool {
    if msg.text.contains(&get_self_id().await.to_string()) {
        return true;
    }
    msg.text.to_lowercase().find("порфирьевич").is_some()
}

pub(crate) async fn add_prompt(context: &mut String, msg: &mut TextMessage) -> bool {
    context.push_str("Порфирьевич:\n-");
    false
}

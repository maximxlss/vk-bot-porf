#![feature(never_type, async_closure)]
#![feature(round_char_boundary)]

mod commands;
mod storage;
mod talk;

use parking_lot::Mutex;
use vk_bot_porf::*;
use lazy_static::lazy_static;
use crate::commands::handle_commands;
use crate::storage::Storage;
use crate::talk::handle_talking;


lazy_static! {
    static ref STORAGE: Mutex<Storage> = Mutex::new(Storage::load());
}


#[tokio::main]
async fn main() -> Result<!, rvk::error::Error> {
    poll_for_messages(async move |msg| {
        if !handle_commands(&msg).await {
            handle_talking(&msg).await;
        };
    }).await
}

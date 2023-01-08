# [VKontakte](https://vk.com/) bot that generates text with [Porfirevich](https://porfirevich.ru/)
To use:
- get the token in group settings (you also have to give it the group management permissions, enable long-poll API and may enable the ability to invite the bot to groups)
- put the token in `src/token` file in plain text.
- put the group id (can be seen in the link at the end, the numbers only) in `src/group_id` file in plain text.
- run with `cargo run --release` or build with `cargo build --release`
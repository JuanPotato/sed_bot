extern crate tg_botapi;
extern crate regex;

use tg_botapi::args;
use tg_botapi::BotApi;
use tg_botapi::types::Message;

use regex::Regex;

use std::sync::Arc;
use std::thread;
use std::env;

fn main() {
    let token = &env::var("TOKEN").expect("No bot token provided, please set the environment variable TOKEN");
    let bot_arc = Arc::new(BotApi::new(token));
    let admin_id = 0123456789;

    let mut update_args = args::GetUpdatesBuilder::default().timeout(600).offset(0).build().unwrap();

    loop {
        let res_updates = bot_arc.get_updates(&update_args);

        match res_updates {
            Ok(updates) => {
                for update in updates {
                    update_args.offset = Some(update.update_id + 1);

                    if let Some(message) = update.message {
                        let bot = bot_arc.clone();

                        thread::spawn(move || {
                            handle_message(bot, message);
                        });
                    }
                }
            }
            Err(err) => {
                let new_msg = args::SendMessageBuilder::default()
                    .text(format!("`{}`", err.to_string()))
                    .chat_id(admin_id)
                    .parse_mode(String::from("Markdown")).build().unwrap();

                let _ = bot_arc.send_message(&new_msg);
            }
        }
    }
}

fn handle_message(bot: Arc<BotApi>, message: Message) {
    if message.text.is_none() {
        return;
    }

    if message.reply_to_message.is_none() {
        return;
    }

    let reply_msg = message.reply_to_message.unwrap();

    if reply_msg.text.is_none() {
        return;
    }

    if message.from.is_none() {
        return;
    }

    let msg_text = message.text.unwrap();
    let reply_msg_id = reply_msg.message_id;
    let reply_msg_text = reply_msg.text.unwrap();

    if msg_text.starts_with("s/") || msg_text.starts_with("/s/") {
        let boundaries = get_boundaries(&msg_text);
        let len = boundaries.len();
        match len {
            2 | 3 => {
                let pattern = &msg_text[boundaries[0]+1 .. boundaries[1]].replace("\\/", "/");
                let to = if len == 3 {
                    msg_text[boundaries[1]+1 .. boundaries[2]].to_string().replace("\\/", "/")
                } else {
                    String::new()
                };
                let re = Regex::new(pattern);
                match re {
                    Ok(result) => {
                        let after = result.replace_all(&reply_msg_text, to.as_str());

                        let new_msg = args::SendMessageBuilder::default()
                            .text(after.into_owned())
                            .chat_id(message.chat.id)
                            .reply_to_message_id(reply_msg_id)
                            .build().unwrap();
                        
                        let _ = bot.send_message(&new_msg);
                    }

                    Err(err) => {
                        let new_msg = args::SendMessageBuilder::default()
                            .text(err.to_string())
                            .chat_id(message.chat.id)
                            .reply_to_message_id(message.message_id)
                            .build().unwrap();

                        let _ = bot.send_message(&new_msg);
                    }
                }
            }
            _ => {
                let new_msg = args::SendMessageBuilder::default()
                    .text("Invalid number of delimiters!")
                    .chat_id(message.chat.id)
                    .reply_to_message_id(message.message_id)
                    .build().unwrap();
                    
                let _ = bot.send_message(&new_msg);
            }
        }
    }  
}

fn get_boundaries(string: &str) -> Vec<usize> { // Better than regex
    let mut boundaries = Vec::new();
    let mut previous_char = '/';

    for (index,cha) in string.char_indices() {
        if '/' == cha && previous_char != '\\' {
            boundaries.push(index);
        }

        previous_char = cha;
        
        if cha == '\\' && previous_char == '\\' {
            previous_char = ' ';
        }
    }

    if boundaries[0] == 0 {
        let _ = boundaries.remove(0);
    }

    if boundaries[boundaries.len() - 1] != string.len() - 1 {
        boundaries.push(string.len());
    }

    boundaries
}

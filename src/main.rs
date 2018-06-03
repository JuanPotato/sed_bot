extern crate telegram_bot;
extern crate tokio_core;
extern crate futures;
extern crate regex;
use regex::Regex;

use telegram_bot::{
    Api,
    ParseMode,
    MessageKind,
    UpdateKind,
    MessageOrChannelPost,
    CanReplySendMessage,
};

use tokio_core::reactor::Core;

use futures::stream::Stream;

use std::env;



fn main() {
    let mut core = Core::new().unwrap();

    let token = env::var("TOKEN").unwrap();
    let api = Api::configure(token).build(core.handle()).unwrap();

    let future = api.stream().for_each(|update| {

        if let UpdateKind::Message(message) = update.kind {
            if message.reply_to_message.is_none() {
                return Ok(());
            }

            let reply_msg = message.reply_to_message.clone().unwrap();

            let msg_text_opt = get_text(&message.kind);
            let reply_text_opt = get_reply_text(&reply_msg);

            if let (Some(text), Some(reply_text)) = (msg_text_opt, reply_text_opt) {
                if !(text.starts_with("s/") || text.starts_with("/s/")) {
                    return Ok(());
                }

                println!("{}", &text);

                match handle_message(&text, &reply_text) {
                    Ok(s) => {
                        api.spawn(
                            reply_msg
                            .text_reply(s)
                            //.parse_mode(ParseMode::Markdown)
                            .disable_preview());
                    },

                    Err(e) => {
                        api.spawn(
                            message
                            .text_reply(e)
                            .parse_mode(ParseMode::Markdown)
                            .disable_preview());
                    }
                }

            }
        }

        Ok(())
    });

    core.run(future).unwrap();
}

fn get_reply_text(reply_msg: &MessageOrChannelPost) -> Option<String> {
    match reply_msg {
        MessageOrChannelPost::Message(message) => {
            get_text(&message.kind)
        },

        MessageOrChannelPost::ChannelPost(post) => {
            get_text(&post.kind)
        },
    }
}


fn get_text(msg_kind: &MessageKind) -> Option<String> {
    match msg_kind {
        MessageKind::Text {ref data, ..} => {
            Some(data.to_string())
        },

        MessageKind::Document {ref caption, ..} => {
            caption.clone()
        },

        MessageKind::Photo {ref caption, ..} => {
            caption.clone()
        },

        MessageKind::Video {ref caption, ..} => {
            caption.clone()
        },

        _ => {
            None
        }
    }
}

fn handle_message(text: &str, reply_text: &str) -> Result<String, String> {
    // Assumes messages starts with s/ or /s/
    let boundaries = get_boundaries(&text);
    let len = boundaries.len();

    match len {
        2 | 3 => {
            let pattern = &text[boundaries[0] + 1 .. boundaries[1]].replace("\\/", "/").replace("\\\\", "\\");

            let to = if len == 3 {
                text[boundaries[1] + 1 .. boundaries[2]].to_string().replace("\\/", "/").replace("\\\\", "\\")
            } else {
                String::new()
            };

            let re = Regex::new(pattern);

            match re {
                Ok(result) => {
                    let after = result.replace_all(&reply_text, to.as_str());

                    if after == "" {
                        Err("`java.lang.NullPointerException: Empty Message`".into())
                    } else {
                        Ok(after.into_owned())
                    }
                }

                Err(err) => Err(err.to_string())
            }
        }
        _ => Err("Invalid number of delimiters!".into()),
    }
}

fn get_boundaries(string: &str) -> Vec<usize> { // Better than regex
    let mut boundaries = Vec::new();
    let mut previous_char = '/';

    for (index,cha) in string.char_indices() {
        if '/' == cha && previous_char != '\\' {
            boundaries.push(index);
        }

        if cha == '\\' && previous_char == '\\' {
            previous_char = ' ';
        } else {
            previous_char = cha;
        }

    }

    if boundaries[0] == 0 {
        let _ = boundaries.remove(0);
    }

    if boundaries[boundaries.len() - 1] != string.len() - 1 {
        boundaries.push(string.len());
    }

    let s1 = &string[boundaries[0]+1..boundaries[1]];
    let s2 = if boundaries.len() > 2 {
        &string[boundaries[1]+1..boundaries[2]]
    } else {
        &string[boundaries[1]..boundaries[1]]
    };

    println!("\nsubstitute [{}] for [{}]", s1, s2);

    boundaries
}

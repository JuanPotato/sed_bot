extern crate telegram_bot;
extern crate tokio_core;
extern crate futures;
extern crate pcre;

use pcre::Pcre;

use telegram_bot::{
    Api,
    ParseMode,
    UpdateKind,
    MessageKind,
    MessageId,
    MessageOrChannelPost,
    ToMessageId,
    SendMessage,
};

use tokio_core::reactor::Core;

use futures::stream::Stream;

use std::env;

use std::collections::VecDeque;


fn main() {
    let mut core = Core::new().unwrap();
    let mut past_messages = VecDeque::from(vec![(String::new(), MessageId::from(0)); 10]);

    let token = env::var("TOKEN").unwrap();
    let api = Api::configure(token).build(core.handle()).unwrap();

    let future = api.stream().for_each(|update| {

        if let UpdateKind::Message(message) = update.kind {
            let text = match get_text(&message.kind) {
                Some(t) => t,
                None => return Ok(()),
            };

            past_messages.pop_back();
            past_messages.push_front((text.clone(), message.id));

            if !(text.starts_with("s/") || text.starts_with("/s/")) {
                return Ok(());
            }


            let reply_text = match message.reply_to_message.as_ref().map(|m| get_reply_text(m)) {
                Some(Some(t)) => t,
                _ => {
                    match handle_noreply_message(&text, &past_messages) {
                        Ok(Some((text, id))) => {
                            api.spawn(
                                SendMessage::new(message.chat, text)
                                .reply_to(id)
                                .parse_mode(ParseMode::Markdown)
                                .disable_preview());
                        },
                        _ => {}
                    }

                    return Ok(());
                }
            };

            let reply_msg = message.reply_to_message.unwrap();

            println!("{}", &text);

            match handle_message(&text, &reply_text) {
                Ok(s) => {
                    api.spawn(
                        SendMessage::new(message.chat, s)
                        .reply_to(reply_msg.to_message_id())
                        .parse_mode(ParseMode::Markdown)
                        .disable_preview());
                },

                Err(e) => {
                    api.spawn(
                        SendMessage::new(message.chat, e)
                        .reply_to(message.id)
                        .parse_mode(ParseMode::Markdown)
                        .disable_preview());
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

fn handle_noreply_message(text: &str, messages: &VecDeque<(String, MessageId)>) -> Result<Option<(String, MessageId)>, String> {
    let (pattern, to) = get_pattern_and_replacement(text)?;

    let re = Pcre::compile(&pattern);

    match re {
        Ok(mut result) => {
            let mut msg_iter = messages.iter();
            msg_iter.next();

            for (reply_text, id) in msg_iter {
                println!("{:?} ~ {:?}", reply_text, id);
                // loop through the past message
                // once we find one that matches, replace stuff in it
                if result.exec(reply_text).is_some() && !reply_text.is_empty() {
                    let after = replace_all(&mut result, reply_text, &to);

                    if after == "" {
                        return Err("`java.lang.NullPointerException: Empty Message`".into());
                    } else {
                        return Ok(Some((after, *id)));
                    }
                }
            }

            Ok(None)
        }

        Err(err) => Err(err.to_string())
    }
}



fn handle_message(text: &str, reply_text: &str) -> Result<String, String> {
    let (pattern, to) = get_pattern_and_replacement(text)?;

    let re = Pcre::compile(&pattern);

    match re {
        Ok(mut result) => {
            let after = replace_all(&mut result, reply_text, &to);

            if after == "" {
                Err("`java.lang.NullPointerException: Empty Message`".into())
            } else {
                Ok(after)
            }
        }

        Err(err) => Err(err.to_string())
    }
}

fn get_pattern_and_replacement(text: &str) -> Result<(String, String), String> {
    // Assumes messages starts with s/ or /s/

    match split_text(text) {
        Some((s1, s2)) => {
            let pattern = s1.replace("\\/", "/").replace("\\\\", "\\");
            let to = s2.replace("\\/", "/").replace("\\\\", "\\");

            Ok((pattern, to))
        }
        _ => Err("Invalid number of delimiters!".into()),
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
struct PsuedoVec<T: Copy> {
    data: [T; 32],
    length: usize,
}

impl<T: Copy> PsuedoVec<T> {
    #[inline]
    pub fn new(default: T) -> Self {
        PsuedoVec {
            data: [default; 32],
            length: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, item: T) {
        self.data[self.length] = item;
        self.length += 1;
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.length
    }
}

impl<T: Copy> std::ops::Index<usize> for PsuedoVec<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &T {
        &self.data[index]
    }
}

#[inline]
fn get_boundaries(string: &str) -> PsuedoVec<usize> { // Better than regex
    let mut boundaries = PsuedoVec::new(0usize);
    let mut previous_char = '/';

    for (index,cha) in string[1..].char_indices() {
        if '/' == cha && previous_char != '\\' {
            boundaries.push(index + 1);

            if boundaries.len() > 3 {
                break;
            }
        }

        if cha == '\\' && previous_char == '\\' {
            previous_char = ' ';
        } else {
            previous_char = cha;
        }
    }

    if boundaries[boundaries.len() - 1] != string.len() - 1 {
        boundaries.push(string.len());
    }

    if boundaries.len() == 2 {
        let i = boundaries[1];
        boundaries.push(i + 1);
    }

    boundaries
}

#[inline]
fn split_text(text: &str) -> Option<(&str, &str)> {
    let boundaries = get_boundaries(text);

    if boundaries.len() != 3 {
        return None;
    }

    let s1 = &text[boundaries[0]+1..boundaries[1]];
    let s2 = &text[boundaries[1]+1..boundaries[2]];

    println!("\nsubstitute [{}] for [{}]", s1, s2);

    Some((s1, s2))
}


static GROUPS: [&'static str; 10] = [
    "\\0",
    "\\1",
    "\\2",
    "\\3",
    "\\4",
    "\\5",
    "\\6",
    "\\7",
    "\\8",
    "\\9",
];

fn replace_all(pattern: &mut Pcre, subject: &str, replace_str: &str) -> String {
    let mut new_str = String::new();
    let mut last_end: usize = 0;

    for m in pattern.matches(subject) {
        // push the space between the end of the last match and the beginning of this one
        new_str.push_str(&subject[last_end .. m.group_start(0)]);
        let replaced = replace(&m, replace_str);
        new_str.push_str(&replaced);
        last_end = m.group_end(0);
    }

    new_str.push_str(&subject[last_end..]);

    new_str
}

fn replace(pcre_match: &pcre::Match, replace_str: &str) -> String {
    let mut new_str = String::from(replace_str);

    for i in 0..pcre_match.string_count() {
        new_str = new_str.replace(GROUPS[i], pcre_match.group(i));
    }

    new_str
}


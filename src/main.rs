extern crate regex;
use regex::Regex;

fn main() {
    let tests: &[&str] = &[
        "s/sed/potato",
        "/s/sed/potato"
    ][..];

    let delimiters = [
        ("/s/", "s/")
    ];

    for &(b1, b2) in delimiters[..].iter() {
        for string in tests {
            if string.starts_with(b1) || string.starts_with(b2) {
                let boundaries = get_boundaries(string);
                for i in 0 .. (boundaries.len() - 1) {
                    println!("{:?}", &string[boundaries[i]..boundaries[i+1]]);
                }
            } 
        }
    }
}

// I probably shouldnt be returning a Vec *shrug*

fn get_boundaries(string: &str) -> Vec<i64> {
    let mut boundaries: Vec<i64> = Vec::new();
    let mut previous_char = '/';

    for (index,cha) in string.char_indices() {
        match cha {
            '/' => {
                if previous_char != '\\' {
                    boundaries.push(index as i64);
                }
            }
            _ => previous_char = cha
        }
    }

    boundaries
}

fn main() {
    let tests = [
        "s/sed/potato",
        "/s/sed/potato"
    ];

    let char_iter = tests[0].char_indices();

    let stage = 0;

    for (index, chara) in char_iter {
        match chara {
            '/' => {
                if index == 0 {
                    stage = 1;
                } else if (index == 1 || index == 2) && stage == 2 {
                    if 
                } else {
                    stage = -1;
                    break;
                }
            },
            's' => {
                if index == 0 || (index == 1 && stage == 1) {
                    stage = 2;
                } else {
                    stage = -1;
                    break;
                }
            }
        }
    } // what am I even doing here
}

fn get_boundaries(char_iter: &str::CharIndices, ) {
    let middle = 0;
    while let Some(chara) = char_iter.next() {

    }
}
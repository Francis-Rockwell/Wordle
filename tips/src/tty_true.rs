use {
    crate::common::{color, diffcult, exist, input, stats, ACCEPTSET, LENGTH, LIMIT},
    std::io,
};

/*
function: to print a letter according the color
input: letter: letter to be printed
        color: the color of the letter
output: None
*/
pub fn char_color_print(letter: &char, color: &char) {
    match color {
        'G' => print!("{}", console::style(letter).bold().green()),
        'Y' => print!("{}", console::style(letter).bold().yellow()),
        'R' => print!("{}", console::style(letter).bold().red()),
        'X' => print!("{}", console::style(letter).bold().white()),
        _ => {}
    }
}

/*
function: to give back a letter's loaction on keyboard
input: letter: the target letter
output: the coordinate of the letter
*/
pub(crate) fn char2location(letter: char) -> (usize, usize) {
    let keyboard = vec![
        vec!['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P'],
        vec!['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L'],
        vec!['Z', 'X', 'C', 'V', 'B', 'N', 'M'],
    ];
    for h in 0..3 {
        for v in 0..keyboard[h].len() {
            if letter == keyboard[h][v] {
                return (h, v);
            }
        }
    }
    panic!("char 2 loaction error");
}

/*
function: to color a keyboard from the new guess and the answer
input: guess: new guess
        color: the color of the guess
        keyboard: a 26-length array of chars each says the color of the letter
output: None
*/
pub(crate) fn change_keyboard(guess: &String, color: &Vec<char>, keyboard: &mut Vec<Vec<char>>) {
    let guess = guess.chars().collect::<Vec<char>>();
    for i in 0..LENGTH {
        let sub = char2location(guess[i]);
        if int(color[i]) > int(keyboard[sub.0][sub.1]) {
            keyboard[sub.0][sub.1] = color[i];
        }
    }
}

/*
function: to display a colored keyboard
input: keyboard: the vec contains each letter's colorchar
output: None
*/
pub fn keyboardout(keyboard: &Vec<Vec<char>>) {
    let stdkeyboard = vec![
        vec!['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P'],
        vec!['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L'],
        vec!['Z', 'X', 'C', 'V', 'B', 'N', 'M'],
    ];
    for i in 0..3 {
        for j in 0..keyboard[i].len() {
            char_color_print(&stdkeyboard[i][j], &keyboard[i][j]);
        }
        println!();
    }
}

/*
function: to change the color into an i32 for easier comparison
input: color: the color to be transformed
output: an i32 from the color char
*/
pub fn int(color: char) -> i32 {
    match color {
        'G' => 4,
        'Y' => 3,
        'R' => 2,
        'X' => 1,
        _ => 0,
    }
}

/*
function: to print a word according the color
input: guess: word to be printed
        color: the colors of each letter in the word
output: None
*/
pub fn string_color_print(guess: &str, color: &Vec<char>) {
    let guess: Vec<char> = guess.clone().chars().collect();
    for i in 0..LENGTH {
        char_color_print(&guess[i], &color[i]);
    }
}

/*
function: to play a round of game in an interactive terminal
input: ans: answer for this round of game
        guesses: where stores all the valid guesses
        results: where stores the results of all games, including win/lose, mean trails to win a game
output: None
*/
pub fn tty(
    ans: &str,
    hard: bool,
    guesses: &mut Vec<String>,
    results: &mut Vec<Option<(Option<bool>, Option<usize>)>>,
) -> Result<(), String> {
    let mut total: usize = 0;
    let mut colors: Vec<Vec<char>> = vec![];
    let mut keyboard = ['X'; 26];
    let mut invaid: Vec<String> = vec![];
    let mut kboard = vec![
        vec!['X', 'X', 'X', 'X', 'X', 'X', 'X', 'X', 'X', 'X'],
        vec!['X', 'X', 'X', 'X', 'X', 'X', 'X', 'X', 'X'],
        vec!['X', 'X', 'X', 'X', 'X', 'X', 'X'],
    ];
    let mut possible_answers = vec![];
    let mut round_guesses = vec![];
    let mut entrophies = vec![];
    loop {
        if total != 0 {
            // not first guess, give suggest guess here
            let (word, infor) = max_entrophy(&possible_answers);
            println!("Suggested guess: {} {:.4}", word, infor);
        }
        println!("Input your guess: ");
        let guess = input()?;
        let guessfmt = guess.to_ascii_uppercase();
        if exist(&guess, unsafe { &ACCEPTSET }) == false {
            invaid.push(guess.clone());
            println!("INVALID");
        } else if hard
            && total > 0
            && !diffcult(&guess, &guesses[guesses.len() - 1], &colors[total - 1])
        {
            invaid.push(guess.clone());
            println!("INVALID");
        } else {
            // valid
            guesses.push(guessfmt.clone());
            round_guesses.push(guessfmt.clone().to_ascii_lowercase());
            total += 1;
            let color = color(ans, &guess);
            colors.push(color.clone());
            keyboarder(&guessfmt, &color, &mut keyboard);
            change_keyboard(&guessfmt, &color, &mut kboard);
            let entro: f64;
            if total == 1 {
                entro = entrophy(&guessfmt, &color, unsafe { &ACCEPTSET });
            } else {
                entro = entrophy(&guessfmt, &color, &possible_answers);
            } // get the amount of information increased
            entrophies.push(entro);
            if total == 1 {
                // change possible answers
                possible_answers = possible1(&guessfmt, &color, &keyboard);
            } else {
                possible(&round_guesses, &colors, &keyboard, &mut possible_answers);
            }
        }
        for i in guesses.len() - total..guesses.len() {
            // print valid answer
            string_color_print(&guesses[i], &colors[i + total - guesses.len()]);
            println!(" {:.4}", entrophies[i + total - guesses.len()]);
        }
        for i in 0..invaid.len() {
            // print invalid answer
            println!("{}: INVALID", invaid[i]);
        }
        keyboardout(&kboard);
        if guessfmt == ans {
            println!("CORRECT {}", total);
            results.push(Some((Some(true), Some(total))));
            break Ok(());
        } else if total == LIMIT {
            println!("FAILED, {}", ans);
            results.push(Some((Some(false), Some(LIMIT))));
            break Ok(());
        } else {
            //display possible answers
            println!("possible answers are as follows:");
            for possibiled in quantify(&mut possible_answers) {
                println!("{}: {:.4}", possibiled.0, possibiled.1);
            }
        }
    }
}

/*
function: to decide whether or not to play another game
input: None
output: true to play another game, false to quit
*/
pub fn go_on_tty() -> Result<bool, String> {
    println!("Another round? (Input 'Y' to start another round, 'N' to end the game.)");
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(0) => Ok(false),
        Ok(2) => match input.trim() {
            "Y" => Ok(true),
            "N" => Ok(false),
            _ => Err(String::from("Input Error")),
        },
        _ => Err(String::from("Input Error")),
    }
}

/*
function: to display the statistics in an iteractive terminal
input: guesses: where stores all the valid guesses
        results: where stores the results of all games, including win/lose, mean trails to win a game
output: None
*/
pub fn stats_tty(guesses: &Vec<String>, results: &Vec<Option<(Option<bool>, Option<usize>)>>) {
    let append = stats(guesses, results);
    print!("Up to now, you have won {} round", (append.0).0);
    if (append.0).0 > 1 {
        print!("s");
    }
    print!(", lost {} round", (append.0).1);
    if (append.0).1 > 1 {
        print!("s");
    }
    print!(", and tried {:.2} time", (append.0).2);
    if (append.0).2 > 1.0 {
        print!("s");
    }
    println!(" on average to win a round.");
    let mut last: usize = 5;
    if (append.1).len() < 5 {
        last = (append.1).len();
    }
    println!("These are the words you have used relatively more frequently: ");
    for i in 0..last {
        print!("{},{}time", (append.1)[i].0, (append.1)[i].1);
        if (append.1)[i].1 > 1 {
            print!("s");
        }
        print!("; ");
    }
    println!();
}

/*
function: to color a keyboard from the new guess and the answer
input: guess: new guess
        color: the color of the guess
        keyboard: a 26-length array of chars each says the color of the letter
output: None
*/
pub fn keyboarder(guess: &String, color: &Vec<char>, keyboard: &mut [char; 26]) {
    let guess = guess.to_ascii_uppercase().chars().collect::<Vec<char>>();
    let mut sub = [0; 6];
    for i in 0..LENGTH {
        sub[i] = guess[i] as usize - 65;
        if int(color[i]) > int(keyboard[sub[i]]) {
            keyboard[sub[i]] = color[i];
        }
    }
}

/*
function: to check what words in the possible answers fits all the guesses-colors results
input: guesses: the guesses for this round
        colors: the colors of the guess
        acceptable: a vec of unchecked possible answers
output: the actual possible answers
*/
pub fn check(
    guesses: &Vec<String>,
    colors: &Vec<Vec<char>>,
    acceptable: &Vec<String>,
) -> Vec<String> {
    let mut possible_answers = vec![];
    for ans in acceptable {
        let mut judge = true;
        for i in 0..guesses.len() {
            if colors[i] != color(ans, &guesses[i]) {
                // simulate every possible answer
                judge = false;
                break;
            }
        }
        if judge {
            possible_answers.push(ans.clone());
        }
    }
    possible_answers
}

/*
    the following functions with the concept entrophy are inspired by the video with the link:
    "https://www.bilibili.com/video/BV1zZ4y1k7Jw", author: 3Blue1Brown
*/

/*
function: to change the possible answers based on new guess-color
input: guess: new guess
        color: the color of the guess
        keyboard: a 26-length array of chars each says the color of the letter
        possible_answers: the previous possible answers to be selected
output: None
*/
pub fn possible(
    guesses: &Vec<String>,
    colors: &Vec<Vec<char>>,
    keyboard: &[char; 26],
    possible_answers: &mut Vec<String>,
) {
    let mut guess = vec![];
    let mut green = vec![];
    let mut yellow = vec![];
    let mut red = vec![];
    let mut acceptcable = vec![];
    for g in guesses {
        guess.push(
            g.clone()
                .to_ascii_lowercase()
                .chars()
                .collect::<Vec<char>>(),
        );
    }
    for i in 0..colors.len() {
        for j in 0..LENGTH {
            if colors[i][j] == 'G' && !green.contains(&(guess[i][j], j)) {
                green.push((guess[i][j], j));
            }
        }
    } // get the order and letter of the greens
    for i in 0..26 {
        if keyboard[i] == 'Y' {
            yellow.push(char::from_u32((i + 97) as u32).unwrap());
        } else if keyboard[i] == 'R' {
            red.push(char::from_u32((i + 97) as u32).unwrap());
        }
    } // get the letters of the yellows and reds
    for i in 0..possible_answers.len() {
        let accept = possible_answers[i].chars().collect::<Vec<char>>();
        let mut judge = true;
        for i in 0..green.len() {
            if accept[green[i].1] != green[i].0 {
                judge = false;
                break;
            }
        } // greens can't be changed
        if judge {
            for i in 0..red.len() {
                if accept.contains(&red[i]) {
                    judge = false;
                    break;
                }
            } // can't have reds
            if judge {
                for i in 0..yellow.len() {
                    if !accept.contains(&yellow[i]) {
                        judge = false;
                        break;
                    }
                } // must have yellows
                if judge {
                    acceptcable.push(accept.iter().collect::<String>());
                }
            } else {
                continue;
            }
        } else {
            continue;
        }
    }
    *possible_answers = check(guesses, colors, &acceptcable); // further check the possible answers
}

/*
function: to select the possible answers from the ACCEPTSET based on new guess-color
input: guess: new guess
        color: the color of the guess
        keyboard: a 26-length array of chars each says the color of the letter
        possible_answers: the previous possible answers to be selected
output: the possible answers
*/
pub fn possible1(guess: &String, thiscolor: &Vec<char>, keyboard: &[char; 26]) -> Vec<String> {
    let guesses = guess
        .clone()
        .to_ascii_lowercase()
        .chars()
        .collect::<Vec<char>>();
    let mut green = vec![];
    let mut yellow = vec![];
    let mut red = vec![];
    let mut acceptable = vec![];
    for i in 0..LENGTH {
        if thiscolor[i] == 'G' && !green.contains(&(guesses[i], i)) {
            green.push((guesses[i], i));
        }
    } // get the order and letter of the greens
    for i in 0..26 {
        if keyboard[i] == 'Y' {
            yellow.push(char::from_u32((i + 97) as u32).unwrap());
        } else if keyboard[i] == 'R' {
            red.push(char::from_u32((i + 97) as u32).unwrap());
        }
    } // get the letters of the yellows and reds
    for accept in unsafe { &ACCEPTSET } {
        let accept = accept.chars().collect::<Vec<char>>();
        let mut _judge = true;
        for i in 0..green.len() {
            if accept[green[i].1] != green[i].0 {
                _judge = false;
                break;
            } // greens can't be changed
        }
        if _judge {
            for i in 0..red.len() {
                if accept.contains(&red[i]) {
                    _judge = false;
                    break;
                } // can't have reds
            }
            if _judge {
                for i in 0..yellow.len() {
                    if !accept.contains(&yellow[i]) {
                        _judge = false;
                        break;
                    } // must have yellows
                }
                if _judge {
                    acceptable.push(accept.iter().collect::<String>());
                }
            } else {
                continue;
            }
        } else {
            continue;
        }
    }
    let mut possible_answers = vec![];
    for ans in acceptable {
        if *thiscolor == color(&ans, guess) {
            possible_answers.push(ans.clone());
        }
    } // further check the possible answers
    possible_answers
}

/*
function: to quantify every possible answer's expected entrophy and sort them base on this
input: possible_answers: the previous possible answers to be sorted
output: the sorted possible answers with their entrophy
*/
pub fn quantify(possible_answers: &mut Vec<String>) -> Vec<(String, f64)> {
    let mut values = vec![];
    let mut results = vec![];
    for i in 0..possible_answers.len() {
        let mut results: Vec<(Vec<char>, i32)> = vec![];
        let mut entrophy = 0.0;
        for j in 0..possible_answers.len() {
            // simulate every possible answer and count the each color mode
            let result = color(&possible_answers[j], &possible_answers[i]);
            let mut judge = true;
            for k in 0..results.len() {
                if results[k].0 == result {
                    results[k].1 += 1;
                    judge = false;
                    break;
                }
            }
            if judge {
                results.push((result, 1));
            }
        }
        for j in 0..results.len() {
            // calculate entrophy
            let possibility = ((results[j].1) as f64) / (possible_answers.len() as f64);
            entrophy += possibility * (-f64::log2(possibility));
        }
        values.push(entrophy);
    }
    for i in 0..possible_answers.len() {
        //sort
        let mut max = i;
        for j in i..possible_answers.len() {
            if values[j] > values[max] {
                max = j;
            }
        }
        results.push((possible_answers[max].clone(), values[max]));
        (possible_answers[i], possible_answers[max]) =
            (possible_answers[max].clone(), possible_answers[i].clone());
        (values[i], values[max]) = (values[max], values[i]);
    }
    results
}

/*
function: to quantify every word in the ACCEPTSET's expected entrophy and give out the top a fews
input: wants: the number of the expexted results
output: the words with top a fews entrophies
*/
pub fn quantify1(wants: usize) -> Vec<(String, f64)> {
    if wants > unsafe { ACCEPTSET.len() } {
        return vec![(String::from("want too much"), 0.0)];
    }
    let mut values = vec![];
    let mut results = vec![];
    for i in 0..unsafe { ACCEPTSET.len() } {
        let mut results: Vec<(Vec<char>, i32)> = vec![];
        let mut entrophy = 0.0;
        for j in 0..unsafe { ACCEPTSET.len() } {
            // simulate every possible answer and count the each color mode
            let result = color(&unsafe { &ACCEPTSET }[j], &unsafe { &ACCEPTSET }[i]);
            let mut judge = true;
            for k in 0..results.len() {
                if results[k].0 == result {
                    results[k].1 += 1;
                    judge = false;
                    break;
                }
            }
            if judge {
                results.push((result, 1));
            }
        }
        for j in 0..results.len() {
            // calculate entrophy
            let possibility = ((results[j].1) as f64) / (unsafe { ACCEPTSET.len() } as f64);
            entrophy += possibility * (-f64::log2(possibility));
        }
        values.push(entrophy);
    }
    for i in 0..wants {
        //sort
        let mut max = i;
        for j in i..unsafe { ACCEPTSET.len() } {
            if values[j] > values[max] {
                max = j;
            }
        }
        results.push((unsafe { &ACCEPTSET }[max].clone(), values[max]));
        (values[i], values[max]) = (values[max], values[i]);
    }
    results
}

/*
function: to calculate the information the guess-color gives
input: guess: new guess
        color: the color of the guess
        possible_answers: the previous possible answers
output: the sorted possible answers with their entrophy
*/
pub fn entrophy(guess: &String, thiscolor: &Vec<char>, possible_answers: &Vec<String>) -> f64 {
    let mut num = 0;
    for j in 0..possible_answers.len() {
        // simulate every possible answer and count the each color mode
        let result = color(&possible_answers[j].to_ascii_uppercase(), &guess);
        if &result == thiscolor {
            num += 1;
        }
    }
    let possibility = (num as f64) / (possible_answers.len() as f64); // calculate entrophy
    if possibility == 1.0 {
        0.0
    } else {
        -f64::log2(possibility)
    }
}

/*
function: to find the word expected to give the most information
input: possible_answers: possible answers
output: the actual possible answers and the bits of information it is expected to give
*/
pub fn max_entrophy(possible_answers: &Vec<String>) -> (String, f64) {
    if possible_answers.len() == 1 {
        return (possible_answers[0].clone(), 0.0);
    }
    let mut values = vec![];
    for i in 0..unsafe { &ACCEPTSET }.len() {
        let mut results: Vec<(Vec<char>, i32)> = vec![];
        let mut entrophy = 0.0;
        for j in 0..possible_answers.len() {
            // simulate every possible answer and count the each color mode
            let result = color(&possible_answers[j], &unsafe { &ACCEPTSET }[i]);
            let mut judge = true;
            for k in 0..results.len() {
                if results[k].0 == result {
                    results[k].1 += 1;
                    judge = false;
                    break;
                }
            }
            if judge {
                results.push((result, 1));
            }
        }
        for j in 0..results.len() {
            // calculate entrophy
            let possibility = ((results[j].1) as f64) / (possible_answers.len() as f64);
            entrophy += possibility * (-f64::log2(possibility));
        }
        values.push(entrophy);
    }
    let mut max = 0;
    for i in 0..unsafe { &ACCEPTSET }.len() {
        // find the max
        if values[i] > values[max] {
            max = i;
        }
    }
    (unsafe { &ACCEPTSET[max] }.clone(), values[max])
}

/*
function: to test the average trail of the algorithm and top a few suggestted beginning words
input: none
output: the average trail of the algorithm and top a few suggestted beginning words
*/

pub fn test() {
    let total = unsafe { &ACCEPTSET }.len();
    let begin = String::from("tares");
    let mut mean_round = 0.0;
    for j in 0..total {
        // simulate every possible answer
        let mut round = 1;
        let ans = unsafe { &ACCEPTSET }[j].clone();
        if begin == ans {
            mean_round += 1.0 / total as f64;
            continue;
        }
        let mut colors: Vec<Vec<char>> = vec![];
        let mut guesses = vec![];
        let mut keyboard = ['X'; 26];
        let begincolor = color(&ans, &begin);
        keyboarder(&begin, &begincolor, &mut keyboard);
        // initiate possible answers
        let mut possible_answers = possible1(&begin, &begincolor, &keyboard);
        loop {
            // playing the game according to my algorithm
            round += 1;
            let guess = max_entrophy(&possible_answers).0; // greedy every round
            if guess == ans {
                mean_round += round as f64 / total as f64; // calculate average trails
                break;
            }
            let thiscolor = color(&ans, &guess);
            guesses.push(guess.clone());
            colors.push(thiscolor.clone());
            keyboarder(&guess, &thiscolor, &mut keyboard);
            possible(&guesses, &colors, &keyboard, &mut possible_answers);
        }
    }
    println!("Average trail: {}", mean_round);
}

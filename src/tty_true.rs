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
    println!();
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
    let mut invaid: Vec<String> = vec![];
    let mut keyboard = vec![
        vec!['X', 'X', 'X', 'X', 'X', 'X', 'X', 'X', 'X', 'X'],
        vec!['X', 'X', 'X', 'X', 'X', 'X', 'X', 'X', 'X'],
        vec!['X', 'X', 'X', 'X', 'X', 'X', 'X'],
    ];
    loop {
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
            total += 1;
            let thiscolor = color(ans, &guessfmt);
            colors.push(thiscolor.clone());
            change_keyboard(&guessfmt, &thiscolor, &mut keyboard);
        }
        for i in guesses.len() - total..guesses.len() {
            // print valid answer
            string_color_print(&guesses[i], &colors[i + total - guesses.len()])
        }
        for i in 0..invaid.len() {
            // print invalid answer
            println!("{}: INVALID", invaid[i]);
        }
        keyboardout(&keyboard);
        if guessfmt == ans {
            println!("CORRECT {}", total);
            results.push(Some((Some(true), Some(total))));
            break Ok(());
        } else if total == LIMIT {
            println!("FAILED, {}", ans);
            results.push(Some((Some(false), Some(LIMIT))));
            break Ok(());
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

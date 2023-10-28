use {
    crate::common::{color, diffcult, exist, input, stats, ACCEPTSET, LENGTH, LIMIT},
    std::io,
};
/*
function: to change the color into an i32 for easier comparison
input: color: the color to be transformed
output: an i32 from the color char
*/
pub fn int(color: &char) -> i32 {
    match color {
        'G' => 4,
        'Y' => 3,
        'R' => 2,
        'X' => 1,
        _ => 0,
    }
}

/*
function: to color and display a keyboard from the new guess and the answer
input: guess: new guess
        color: the color of the guess
        keyboard: a 26-length array of chars each says the color of the letter
output: None
*/
pub fn keyboard_out(guess: &str, color: &Vec<char>, keyboard: &mut [char; 26]) {
    let guess: Vec<char> = guess.chars().collect();
    let mut sub = [0; 6];
    for i in 0..LENGTH {
        sub[i] = guess[i] as usize - 97; // order in alphabet
        if int(&color[i]) > int(&keyboard[sub[i]]) {
            keyboard[sub[i]] = color[i];
        }
    }
    for i in 0..26 {
        print!("{}", keyboard[i]);
    }
}

/*
function: to play a round of game in a non interactive terminal
input: ans: answer for this round of game
        guesses: where stores all the valid guesses
        results: where stores the results of all games, including win/lose, mean trails to win a game
output: None
*/
pub fn notty(
    ans: &str,
    hard: bool,
    guesses: &mut Vec<String>,
    results: &mut Vec<Option<(Option<bool>, Option<usize>)>>,
) -> Result<(), String> {
    let mut total: usize = 0;
    let mut keyboard = ['X'; 26];
    let mut colors: Vec<Vec<char>> = vec![];
    loop {
        let guess = input()?;
        if exist(&guess[..], unsafe { &ACCEPTSET }) == false {
            println!("INVALID");
        } else if hard
            && total > 0
            && !diffcult(
                &guess.to_ascii_uppercase(),
                &guesses[&guesses.len() - 1],
                &colors[total - 1],
            )
        {
            println!("INVALID");
        } else {
            // valid guess
            total += 1;
            guesses.push(guess.clone().to_ascii_uppercase());
            let color = color(ans, &guess);
            colors.push(color.clone());
            print!("{} ", color.iter().collect::<String>());
            keyboard_out(&guess, &color, &mut keyboard);
            println!();
        }

        if guess == ans {
            println!("CORRECT {}", total); // win
            results.push(Some((Some(true), Some(total))));
            break Ok(());
        } else if total == LIMIT {
            // lose
            println!("FAILED {}", ans.to_ascii_uppercase());
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
pub fn go_on_notty() -> Result<bool, String> {
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(x) => {
            if x == 0 {
                return Ok(true);
            } else {
                match input.trim() {
                    "Y" => Ok(true),
                    "N" => Ok(false),
                    _ => Err(String::from("Input Error")),
                }
            }
        }
        Err(_) => Err(String::from("Input Error")),
    }
}

/*
function: to display the statistics in a non iteractive terminal
input: guesses: where stores all the valid guesses
        results: where stores the results of all games, including win/lose, mean trails to win a game
output: None
*/
pub fn stats_notty(guesses: &Vec<String>, results: &Vec<Option<(Option<bool>, Option<usize>)>>) {
    let append = stats(&guesses, &results);
    println!("{} {} {:.2}", (append.0).0, (append.0).1, (append.0).2);
    let mut last: usize = 5;
    if (append.1).len() < 5 {
        last = (append.1).len();
    }
    for i in 0..last - 1 {
        print!("{} {} ", ((append.1)[i].0), (append.1)[i].1);
    }
    println!("{} {}", ((append.1)[last - 1].0), (append.1)[last - 1].1);
}

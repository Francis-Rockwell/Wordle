use {
    crate::common::{color, diffcult, exist, input, stats, ACCEPTSET, LENGTH, LIMIT},
    std::io,
};

pub fn int(color: &char) -> i32 {
    match color {
        'G' => 4,
        'Y' => 3,
        'R' => 2,
        'X' => 1,
        _ => 0,
    }
}

pub fn keyboard_out(guess: &str, color: &Vec<char>, keyboard: &mut [char; 26]) {
    let guess: Vec<char> = guess.chars().collect();
    let mut sub = [0; 6];
    for i in 0..LENGTH {
        sub[i] = guess[i] as usize - 97;
        if int(&color[i]) > int(&keyboard[sub[i]]) {
            keyboard[sub[i]] = color[i];
        }
    }
    for i in 0..26 {
        print!("{}", keyboard[i]);
    }
}

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
            total += 1;
            guesses.push(guess.clone().to_ascii_uppercase());
            let color = color(ans, &guess);
            colors.push(color.clone());
            print!("{} ", color.iter().collect::<String>());
            keyboard_out(&guess, &color, &mut keyboard);
            println!();
        }

        if guess == ans {
            println!("CORRECT {}", total);
            results.push(Some((Some(true), Some(total))));
            break Ok(());
        } else if total == LIMIT {
            println!("FAILED {}", ans.to_ascii_uppercase());
            results.push(Some((Some(false), Some(LIMIT))));
            break Ok(());
        }
    }
}

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

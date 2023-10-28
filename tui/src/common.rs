use {
    crate::builtin_words::{ACCEPTABLE, FINAL},
    clap::{App, Arg, ArgMatches},
    rand::{rngs::StdRng, seq::SliceRandom, SeedableRng},
    serde_derive::{Deserialize, Serialize},
    std::{
        cmp::Ordering,
        io,
        io::{Read, Write},
    },
    tui::layout::{Constraint, Direction, Layout, Rect},
};

// the struct of how the json file stores a round of game
#[derive(Serialize, Deserialize)]
pub struct Round {
    pub answer: Option<String>,
    pub guesses: Option<Vec<String>>,
}

//the struct of the json file's content
#[derive(Serialize, Deserialize)]
pub struct Content {
    pub total_rounds: Option<usize>,
    pub games: Option<Vec<Round>>,
}

//the struct for the config file
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub word: Option<String>,
    pub random: Option<bool>,
    pub seed: Option<u64>,
    pub day: Option<usize>,
    pub difficult: Option<bool>,
    pub stats: Option<bool>,
    pub final_set: Option<String>,
    pub acceptable_set: Option<String>,
    pub state: Option<String>,
}

// global varibles
pub static LIMIT: usize = 6; // max trail
pub static LENGTH: usize = 5; // word length
pub static DEFAULT_SEED: u64 = 42; // default random seed
pub static mut FINALSET: Vec<String> = vec![]; // final set
pub static mut ACCEPTSET: Vec<String> = vec![]; // acceptable set

/*
    the method to deal with Rect are cited from the website: "https://www.cnblogs.com/xueweihan/p/15992139.html",
function: to cut the block off some margin
input: father: the original block
        margin: the percentage wanted to be cut off
        Direction: Horizonal or Vertical
output: the cutted block
*/
pub(crate) fn cut(father: Rect, margin: u16, side: Direction) -> Rect {
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Percentage(margin),
                Constraint::Percentage(100 - 2 * margin),
                Constraint::Percentage(margin),
            ]
            .as_ref(),
        )
        .direction(side)
        .split(father);
    chunks[1]
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
function: to transform the color char into style::Color
input: letter: the color letter
output: color style
*/
pub(crate) fn char2color(c: char) -> tui::style::Color {
    match c {
        'G' => tui::style::Color::Green,
        'R' => tui::style::Color::Red,
        'Y' => tui::style::Color::Yellow,
        _ => tui::style::Color::Gray,
    }
}

/*
function: to transform the color char into bordercolor
input: letter: the color letter
output: color style
*/
pub(crate) fn char2border(c: char) -> tui::style::Color {
    match c {
        'B' => tui::style::Color::Blue,
        _ => tui::style::Color::Black,
    }
}

/*
function: to color a keyboard from the new guess and the answer
input: guess: new guess
        color: the color of the guess
        keyboard: a 26-length array of chars each says the color of the letter
output: None
*/
pub(crate) fn change_keyboard(guess: &Vec<char>, color: &Vec<char>, keyboard: &mut Vec<Vec<char>>) {
    for i in 0..LENGTH {
        let sub = char2location(guess[i]);
        if int(color[i]) > int(keyboard[sub.0][sub.1]) {
            keyboard[sub.0][sub.1] = color[i];
        }
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
function: to get a trimed String from the standard input
input: None
output: a trimed string from the standard input
*/
pub(crate) fn input() -> Result<String, String> {
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Err(_) => return Err(String::from("Input Error")),
        Ok(0) => return Err(String::from("Noting to input")),
        _ => {}
    }
    Ok(String::from(input.trim()))
}

/*
function: to read and check the arguments from the command line
input: None
output: the arguments formed in ArgMatches
*/
pub(crate) fn args() -> Result<ArgMatches<'static>, String> {
    let matches = App::new("wordle")
        .arg(
            Arg::with_name("word")
                .long("word")
                .short("w")
                .takes_value(true),
        )
        .arg(Arg::with_name("difficult").long("difficult").short("D"))
        .arg(Arg::with_name("random").long("random").short("r"))
        .arg(
            Arg::with_name("day")
                .long("day")
                .short("d")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("seed")
                .long("seed")
                .short("s")
                .takes_value(true),
        )
        .arg(Arg::with_name("stats").long("stats").short("t"))
        .arg(
            Arg::with_name("final-set")
                .long("final-set")
                .short("f")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("acceptable-set")
                .long("acceptable-set")
                .short("a")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("state")
                .long("state")
                .short("S")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("config")
                .long("config")
                .short("c")
                .takes_value(true),
        )
        .get_matches();
    if matches.is_present("random") {
        if matches.is_present("word") {
            return Err(String::from("Args Error"));
        } // set random and word both
    } else {
        if matches.is_present("day") || matches.is_present("seed") {
            return Err(String::from("Args Error"));
        } // no random but with seed or day
    }
    Ok(matches)
}

/*
function: to check whether or not the word is in the dictionary
input: word: a &str,
        dictionary: a Vec of String
output: a bool that says whether the word exits
*/
pub(crate) fn exist(word: &str, dictionary: &Vec<String>) -> bool {
    let word = &word.to_ascii_lowercase();
    let mut judge = false;
    for item in dictionary {
        if item == word {
            judge = true;
            break;
        }
    }
    judge
}

/*
function: to determine either the letter should be colored yellow or red
input: ans: the answer of this round of game
        guess: where the to-be-colored letter lies
        num: the order of the to-be-colored letter in the guess
output: true for yellow, false for red
*/
pub(crate) fn y_or_r(ans: &Vec<char>, guess: &Vec<char>, num: usize) -> bool {
    let mut ans_count: i32 = 0;
    let mut guess_order: i32 = 1;
    for i in 0..LENGTH {
        if ans[i] == guess[num] {
            ans_count += 1;
        }
    } // count the number of the target letter in the answer
    for i in 0..num {
        if guess[i] == guess[num] {
            guess_order += 1;
        }
    } // count the number of the target letter in the guess before it
    for i in num + 1..LENGTH {
        if guess[i] == guess[num] && guess[i] == ans[i] {
            guess_order += 1;
        }
    } //add the green target letter in the guess after it
    if guess_order > ans_count {
        false
    } else {
        true
    }
}

/*
function: to check whether a word fits the requirments in the difficult mode
input: word: the word to be checked
        last_guess: the last valid guess
        last_color: the color of the last valid guess
output: true if the word fits, false if it doesn't
*/
pub(crate) fn diffcult(word: &str, last_guess: &str, last_color: &Vec<char>) -> bool {
    let mut judge = true;
    let word: Vec<char> = word.chars().collect();
    let last_guess: Vec<char> = last_guess.chars().collect();
    for i in 0..LENGTH {
        if last_color[i] == 'G' && word[i] != last_guess[i] {
            judge = false;
            break;
        } // greens can't be changed
        if last_color[i] == 'Y' {
            let letter = last_guess[i];
            let mut count_new = 0;
            let mut count_old = 0;
            for j in 0..LENGTH {
                if word[j] == letter {
                    count_new += 1;
                } // count the target letter in new guess
                if last_guess[j] == letter && (last_color[j] == 'Y' || last_color[j] == 'G') {
                    count_old += 1;
                } // count the target letter in yellow or green in the last guess
            }
            if count_new < count_old {
                // not enough target letter
                judge = false;
            }
        }
    }
    return judge;
}

/*
function: to get the color of a word based on the answer
input: ans: answer for this round of game
    guess: the word to be colored
output: a Vec of chars standing for the color of each letter
*/
pub(crate) fn color(ans: &str, guess: &str) -> Vec<char> {
    let mut color: Vec<char> = vec![];
    let ans: Vec<char> = ans.clone().to_ascii_lowercase().chars().collect();
    let guess: Vec<char> = guess.clone().to_ascii_lowercase().chars().collect();
    for i in 0..LENGTH {
        if guess[i] == ans[i] {
            color.push('G');
        } else if ans.contains(&guess[i]) {
            if y_or_r(&ans, &guess, i) {
                color.push('Y');
            } else {
                color.push('R');
            }
        } else {
            color.push('R');
        }
    }
    color
}

/*
function: to get an answer when the answer is asigned rather than randomly picked
input: word: the "--word" argument
output: the answer from the "--word" argument
*/
pub fn word(word: &Option<String>) -> Result<String, String> {
    let ans = match word {
        Some(x) => Ok(String::from(x)),
        None => input(), //input answer if not assigned
    };
    match ans {
        Ok(s) => {
            if exist(&s, unsafe { &FINALSET }) {
                Ok(s) //answer has to be in FINALSET
            } else {
                Err(String::from("Answer not Exist"))
            }
        }
        Err(s) => Err(s),
    }
}

/*
function: to get an answer when the mode is set random
input: day: the "--day" argument
        seed: the "--seed" argument
        list: a vec of usize number from 0 to the length of FINALSET
output: the order of ( (the order of the answer in the FINALSET) in the shuffled list )
*/
pub fn random(day: &Option<usize>, seed: &Option<u64>, list: &mut Vec<usize>) -> usize {
    let seed = seed.unwrap();
    list.shuffle(&mut StdRng::seed_from_u64(seed));
    day.unwrap()
}

/*
function: compare the tuple of the word and its occurence first according occurence, then alphabet
input: (guess1,time1), (guess2,time2) are the tuples of word and their times of occurence to be compared
output: Greater if the latter is more prioritiesd (the Greater, the further back to display)
*/
pub(crate) fn compare(
    (guess1, time1): (&String, &i32),
    (guess2, time2): (&String, &i32),
) -> Ordering {
    if time1 > time2 {
        Ordering::Less // more occurrence, Less, former to display
    } else if time1 < time2 {
        Ordering::Greater
    } else {
        if guess1 > guess2 {
            Ordering::Greater
        } else if guess1 < guess2 {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }
}

/*
function: to get won rounds, lost rounds, average trails when winning
         and frequently used words from guesses and results
input: guesses: a vector of all the valid guessed the player inputs
        results: a vector of all the win/lose results and the trails used to win a game
output: ((wins, losses, average trails), frequently used words)
*/
pub(crate) fn stats(
    guesses: &Vec<String>,
    results: &Vec<Option<(Option<bool>, Option<usize>)>>,
) -> ((i32, i32, f64), Vec<(String, i32)>) {
    let mut win = 0;
    let mut lose = 0;
    let mut tryout = 0;
    let mut output: ((i32, i32, f64), Vec<(String, i32)>) = ((0, 0, 0.0), vec![]);
    for i in results {
        if i.is_some() && i.unwrap().0.is_some() {
            if (i.unwrap().0).unwrap() {
                win += 1;
                tryout += (i.unwrap().1).unwrap();
            } else {
                lose += 1;
            }
        }
    }
    if win == 0 {
        output.0 = (0, lose, 0.0);
    } else {
        output.0 = (win, lose, tryout as f64 / win as f64);
    }
    for i in guesses {
        let mut contain = false;
        for j in (output.1).iter_mut() {
            if *i == j.0 {
                j.1 += 1; // count j.0's occurence
                contain = true;
                break;
            }
        }
        if !contain {
            // new word
            (output.1).push((i.to_string(), 1));
        }
    }
    (output.1).sort_by(|(a, b), (c, d)| compare((a, b), (c, d)));
    output
}

/*
function: to get the contents in a file
input: instructions: the argument from "--acceptable" or "--final" or "--config" or "--state"
output: the contents of the file
*/
pub(crate) fn fread(instruction: &Option<String>, name: &str) -> Result<String, String> {
    let mut text: String = String::new();
    let mut file = std::fs::File::open(instruction.as_ref().unwrap()).unwrap();
    if let Err(_) = file.read_to_string(&mut text) {
        let err = String::from(name);
        return Err(err + " open error!");
    }
    Ok(text)
}

/*
function: to check whether a word is a five-letter word
input: the word to be checked
output: true if it passes the check, false if it doesn't
*/
pub fn wordcheck(word: &str) -> bool {
    if word.len() != LENGTH {
        return false;
    } else {
        for letter in word.chars() {
            if !letter.is_ascii_alphabetic() {
                return false;
            }
        }
    }
    true
}

/*
function: to set the finalset and the acceptable set
input: finalset: the "--finalset" argument
        acceptset: the "--acceptable" argument
output: the Vec of String for FINALSET and ACCEPTSET
*/
pub(crate) fn set(
    finalset: &Option<String>,
    acceptset: &Option<String>,
) -> Result<(Vec<String>, Vec<String>), String> {
    let f = finalset.is_some();
    let mut finalvec: Vec<&str> = vec![];
    let mut _finalstr = String::new();
    let mut finalvec2: Vec<String> = vec![];
    let a = acceptset.is_some();
    let mut acceptvec: Vec<&str> = vec![];
    let mut acceptvec2: Vec<String> = vec![];
    let mut _acceptstr = String::new();
    if f {
        // change final set
        _finalstr = fread(finalset, "final-set")?;
        finalvec = _finalstr.split("\n").collect::<Vec<&str>>();
        finalvec.sort();
        finalvec2.push(finalvec[0].to_ascii_lowercase());
    } else {
        finalvec2 = FINAL.iter().map(|s| s.to_string()).collect();
    }
    if a {
        // change accpetable set
        _acceptstr = fread(acceptset, "acceptable-set")?;
        acceptvec = _acceptstr.split("\n").collect::<Vec<&str>>();
        acceptvec.sort();
        acceptvec2.push(acceptvec[0].to_ascii_lowercase());
    } else {
        acceptvec2 = ACCEPTABLE.iter().map(|s| s.to_string()).collect();
    }
    if f || a {
        for i in 1..acceptvec.len() {
            if !wordcheck(&acceptvec[i]) {
                return Err(String::from("acceptable-set wrong"));
            } //check words in acceptable set
            if acceptvec[i] == acceptvec[i - 1] {
                return Err(String::from("acceptable-set duplicate"));
            } // duplicate?
            acceptvec2.push(acceptvec[i].to_ascii_lowercase());
        }
        if !acceptvec.contains(&finalvec[0]) {
            return Err(String::from("acceptable-set dosen't cover final-set"));
        }
        for i in 1..finalvec.len() {
            if !acceptvec.contains(&finalvec[i]) {
                return Err(String::from("acceptable-set dosen't cover final-set"));
            } // cover?
            finalvec2.push(finalvec[i].to_ascii_lowercase());
        }
    }
    Ok((finalvec2, acceptvec2))
}

/*
function: to load state from json file
input: state: the "--state"
        answers: the vec stores all the answers
        guesses: the vec stores all the guesses
        results: the vec stores all the results including wins/lose, mean trails to win
output: None
*/
pub(crate) fn stateload(
    state: &Option<String>,
    answers: &mut Vec<Option<String>>,
    guesses: &mut Vec<String>,
    results: &mut Vec<Option<(Option<bool>, Option<usize>)>>,
) -> Result<(), String> {
    let contents = fread(state, "state")?;
    let text: Result<Content, _> = serde_json::from_str(&contents);
    if text.is_err() {
        return Err(String::from("state format error"));
    }
    let text = text.unwrap();
    if text.games.is_some() {
        for i in text.games.unwrap() {
            answers.push(i.answer.clone()); // deal answers
            if i.guesses.as_ref().is_some() {
                for j in 0..i.guesses.as_ref().unwrap().len() {
                    guesses.push(i.guesses.as_ref().unwrap()[j].clone()); // deal guesses
                }
                if i.answer.as_ref().is_some() {
                    // answer and guesses both are some
                    if i.answer.unwrap() == *i.guesses.as_ref().unwrap().last().unwrap() {
                        // answer match last guess, win
                        results.push(Some((Some(true), Some(i.guesses.as_ref().unwrap().len()))));
                    } else {
                        //otherwisw, lose
                        results.push(Some((Some(false), Some(i.guesses.as_ref().unwrap().len()))));
                    }
                } else {
                    // answer is none, guesses is some, can't decide win/lose
                    results.push(Some((None, Some(i.guesses.unwrap().len()))));
                }
            } else {
                // guesses is none
                results.push(None);
            } // deal results
        }
    }
    Ok(())
}

/*
function: to save state into json file
input: state: the "--state"
        answers: the vec stores all the answers
        guesses: the vec stores all the guesses
        results: the vec stores all the results including wins/lose, mean trails to win
output: None
*/
pub(crate) fn statesave(
    state: &Option<String>,
    answers: &Vec<Option<String>>,
    guesses: &Vec<String>,
    results: &Vec<Option<(Option<bool>, Option<usize>)>>,
) -> Result<(), String> {
    if state.is_none() {
        return Ok(());
    }
    let mut games = vec![];
    for i in 0..answers.len() {
        let mut former = 0;
        for j in 0..i {
            if results[j].is_some() && (results[j].unwrap().1).is_some() {
                former += (results[j].unwrap().1).unwrap();
            }
        } //count guesses before this round
        let mut guess: Vec<String> = vec![];
        if results[i].is_some() && (results[i].unwrap().1).is_some() {
            for j in 0..(results[i].unwrap().1).unwrap() {
                guess.push(guesses[former + j].clone().to_ascii_uppercase());
            } //guess if the guesses for this round
        }
        let answer: Option<String>;
        if answers[i].is_some() {
            answer = Some(answers[i].as_ref().unwrap().clone().to_ascii_uppercase());
        } else {
            answer = None;
        } // answer is the answer for this round
        let mut guess = Some(guess);
        if guess.as_ref().unwrap().is_empty() {
            guess = None;
        } //empty guess save as None
        let r = Round {
            answer,
            guesses: guess,
        }; // a Round can never be empty
        games.push(r);
    }
    let mut games = Some(games);
    if games.as_ref().unwrap().is_empty() {
        games = None;
    } // an empty games save as None
    let text = Content {
        total_rounds: Some(answers.len()),
        games,
    };
    let text = serde_json::to_string_pretty(&text).unwrap();
    let mut file = std::fs::File::create(state.as_ref().unwrap()).unwrap();
    if let Err(_) = file.write_all(text.as_bytes()) {
        Err(String::from("state save error"))
    } else {
        Ok(())
    }
}

/*
function: to transform a brach of argument to a Option<String>
input: arg: the arguments read from the command line
        name: the name of the branch to be transformed
output: None if the argument dosen't exist, Option<String> with its value if it does
*/
pub(crate) fn arg2opstring(arg: &ArgMatches, name: &str) -> Result<Option<String>, String> {
    if arg.is_present(name) {
        let resultstr = arg.value_of(name);
        let mut _result: Option<String> = None;
        if resultstr.is_some() {
            _result = Some(resultstr.unwrap().to_string());
        } else {
            //present without value, Err
            let err = String::from(name);
            return Err(err + " open error!");
        }
        Ok(_result)
    } else {
        // not present
        Ok(None)
    }
}

/*
function: to transform the arguments read from the command line into a Config struct
input: arg: the arguments read from the command line
output: the Config form of the arguments
*/
pub fn arg2config(arg: &ArgMatches) -> Result<Config, String> {
    unsafe {
        (FINALSET, ACCEPTSET) = set(
            &arg2opstring(arg, "final-set")?,
            &arg2opstring(arg, "acceptable-set")?,
        )?;
    }
    let wordstr = arg.value_of("word");
    let mut word: Option<String> = None;
    if wordstr.is_some() {
        word = Some(wordstr.unwrap().to_string());
    }
    let seedstr = arg.value_of("seed");
    let mut _seed: Option<u64> = None;
    if seedstr.is_some() {
        if seedstr.unwrap().parse::<u64>().is_ok() {
            _seed = Some(seedstr.unwrap().parse::<u64>().unwrap());
        } else {
            return Err(String::from("Seed Error"));
        }
    } else {
        // seed not present or present with no value
        _seed = Some(DEFAULT_SEED);
    }
    let daystr = arg.value_of("day");
    let mut _day: Option<usize> = None;
    if daystr.is_some() {
        if daystr.unwrap().parse::<usize>().is_ok() {
            _day = Some(daystr.unwrap().parse::<usize>().unwrap());
            if _day.unwrap() > unsafe { FINALSET.len() } || _day.unwrap() == 0 {
                return Err(String::from("Day Error"));
            }
        } else {
            // day present with not a usize
            return Err(String::from("Day Error"));
        }
    } else {
        _day = Some(1); // day not present or present with no value
    }
    Ok(Config {
        word,
        random: Some(arg.is_present("random")),
        seed: _seed,
        day: _day,
        difficult: Some(arg.is_present("difficult")),
        stats: Some(arg.is_present("stats")),
        final_set: arg2opstring(arg, "final-set")?,
        acceptable_set: arg2opstring(arg, "acceptable-set")?,
        state: arg2opstring(arg, "state")?,
    })
}

/*
function: when the config file and the command line arguments both exist,
            call this function to modify the Config struct from the command line arguments
input: arg: the arguments read from the command line
output: the Config struct from arg
*/
pub fn arg2config_modifier(arg: &ArgMatches) -> Result<Config, String> {
    let mut config = arg2config(arg)?;
    if !arg.is_present("random") {
        config.random = None; // not Some(false)
    }
    if !arg.is_present("seed") {
        config.seed = None; // not Some(default)
    }
    if !arg.is_present("day") {
        config.day = None;
    }
    if !arg.is_present("difficult") {
        config.difficult = None;
    }
    if !arg.is_present("stats") {
        config.stats = None;
    }
    Ok(config)
}

/*
function: to form the ultimat Config struct for the program under every condition
input: arg: the arguments read from the commnd line
output: the ultimat Config struct for the program under every condition
*/
pub fn config(arg: &ArgMatches) -> Result<Config, String> {
    if arg.is_present("config") {
        let config = arg2opstring(arg, "config")?;
        let text = fread(&config, "config")?;
        let config: Result<Config, _> = serde_json::from_str(&text);
        if config.is_err() {
            return Err(String::from("Config Error"));
        }
        let config = config.unwrap(); // forming config
        let mut args = arg2config_modifier(arg)?; // forming args
                                                  // dealing conflicts
        if args.random.is_none() && config.random.is_some() {
            args.random = config.random;
        }
        if args.difficult.is_none() && config.difficult.is_some() {
            args.difficult = config.difficult;
        }
        if args.seed.is_none() && config.seed.is_some() {
            args.seed = config.seed;
        }
        if args.day.is_none() && config.day.is_some() {
            args.day = config.day;
        }
        if args.final_set.is_none() && config.final_set.is_some() {
            args.final_set = config.final_set;
        }
        if args.acceptable_set.is_none() && config.acceptable_set.is_some() {
            args.acceptable_set = config.acceptable_set;
        }
        if args.stats.is_none() && config.stats.is_some() {
            args.stats = config.stats;
        }
        if args.state.is_none() && config.state.is_some() {
            args.state = config.state;
        }
        if args.random.is_some() && args.random.unwrap() {
            if args.word.is_some() {
                return Err(String::from("Config Error"));
            }
        } else {
            if args.seed.is_some() || args.day.is_some() {
                return Err(String::from("Config Error"));
            }
        }
        Ok(args)
    } else {
        arg2config(arg)
    }
}

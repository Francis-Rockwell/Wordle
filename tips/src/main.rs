pub mod builtin_words;
pub mod common;
pub mod tty_true;
use crate::{
    common::{args, config, random, stateload, statesave, word, Config, FINALSET},
    tty_true::{go_on_tty, quantify1, stats_tty, test, tty},
};

/*
function: to play a round of game
input:  ans: the answer of this
        arg: arguments from command line and config file (already processed)
        guesses: where stores every valid guesses the player inputs
        results: where stores the results of every round of games, including win/fail, the times player tried
output: true to play another game, false to quit
*/
fn round(
    arg: &Config,
    ans: &str,
    guesses: &mut Vec<String>,
    results: &mut Vec<Option<(Option<bool>, Option<usize>)>>,
) -> Result<bool, String> {
    let mut next = false;
    tty(
        ans,
        arg.difficult.is_some() && arg.difficult.unwrap(),
        guesses,
        results,
    )?;
    if arg.stats.is_some() && arg.stats.unwrap() {
        stats_tty(&guesses, &results);
    } // show statistics
    if !arg.word.is_some() {
        next = go_on_tty()?; // another round?
    }
    Ok(next)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut guesses: Vec<String> = vec![];
    let mut results: Vec<Option<(Option<bool>, Option<usize>)>> = vec![];
    let mut answers: Vec<Option<String>> = vec![];
    let arg = config(&args()?)?;
    /*test();*/
    println!("Suggested guesses: ");
    let suggested = quantify1(10); // suggestted first guess
    for i in 0..10 {
        println!("{}: {:.4}", suggested[i].0, suggested[i].1);
    }
    if arg.random.is_some() && arg.random.unwrap() {
        // when answers are random
        let list = 0..unsafe { FINALSET.len() };
        let mut list = list.collect::<Vec<usize>>();
        let mut sub = random(&arg.day, &arg.seed, &mut list);
        if arg.state.is_some() {
            // load information from json file
            stateload(&arg.state, &mut answers, &mut guesses, &mut results)?;
        }
        loop {
            let mut ans = unsafe { &FINALSET[list[sub]] }.to_string();
            ans = ans.to_ascii_uppercase();
            if answers.contains(&Some(ans.clone())) {
                // ensure the random answers don't duplicate
                if answers.len() == list.len() {
                    panic!("All answers in the finalset played");
                }
                sub = (sub + 1) % list.len(); //try next answer
                continue;
            }
            answers.push(Some(ans.clone().to_ascii_uppercase()));
            let next = round(&arg, &ans, &mut guesses, &mut results)?; // another round?
            if next {
                sub = (sub + 1) % unsafe { FINALSET.len() }; // change sub to get a new random answer
            } else {
                break;
            }
        }
        if arg.state.is_some() {
            //save information to json file
            statesave(&arg.state, &answers, &guesses, &results)?;
        }
    } else {
        if arg.word.is_some() {
            // when answer is decided in the arguments
            let mut ans = word(&arg.word)?;
            ans = ans.to_ascii_uppercase();
            answers.push(Some(ans.clone().to_ascii_uppercase()));
            round(&arg, &ans, &mut guesses, &mut results)?;
        } else {
            // when answers are from input
            loop {
                println!("Input the answer:");
                let mut ans = word(&arg.word)?;
                ans = ans.to_ascii_uppercase();
                answers.push(Some(ans.clone().to_ascii_uppercase()));
                let next = round(&arg, &ans, &mut guesses, &mut results)?;
                if !next {
                    break;
                }
            }
        }
    }
    Ok(())
}

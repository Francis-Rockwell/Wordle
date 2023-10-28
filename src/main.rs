pub mod builtin_words;
pub mod common;
pub mod tty_false;
pub mod tty_true;
use crate::{
    common::{args, config, random, stateload, statesave, word, Config, FINALSET},
    tty_false::{go_on_notty, notty, stats_notty},
    tty_true::{go_on_tty, stats_tty, tty},
};
/*
function: to play a round of game
input: is_tty: whether or not this is a interactive terminal
        arg: arguments from command line and config file (already processed)
        guesses: where stores every valid guesses the player inputs
        results: where stores the results of every round of games, including win/fail, the times player tried
output: true to play another game, false to quit
*/
fn round(
    is_tty: bool,
    arg: &Config,
    ans: &str,
    guesses: &mut Vec<String>,
    results: &mut Vec<Option<(Option<bool>, Option<usize>)>>,
) -> Result<bool, String> {
    let mut next = false;
    if is_tty {
        //interactive
        tty(
            ans,
            arg.difficult.is_some() && arg.difficult.unwrap(),
            guesses,
            results,
        )?;
        if arg.stats.is_some() && arg.stats.unwrap() {
            stats_tty(&guesses, &results);
        }
        if !arg.word.is_some() {
            // no answer asigned in arg
            next = go_on_tty()?;
        }
    } else {
        notty(
            ans,
            arg.difficult.is_some() && arg.difficult.unwrap(),
            guesses,
            results,
        )?;
        if arg.stats.is_some() && arg.stats.unwrap() {
            stats_notty(&guesses, &results);
        }
        if !arg.word.is_some() {
            next = go_on_notty()?;
        }
    }
    Ok(next)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let is_tty = atty::is(atty::Stream::Stdout);
    let mut guesses: Vec<String> = vec![];
    let mut results: Vec<Option<(Option<bool>, Option<usize>)>> = vec![];
    let mut answers: Vec<Option<String>> = vec![];
    let arg = config(&args()?)?;
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
            if is_tty {
                // when interactive, default inputs are uppercase
                ans = ans.to_ascii_uppercase();
            }
            if answers.contains(&Some(ans.clone())) {
                // ensure the random answers don't duplicate
                if answers.len() == list.len() {
                    panic!("All answers in the finalset played");
                }
                sub = (sub + 1) % list.len(); //try next answer
                continue;
            }
            answers.push(Some(ans.clone().to_ascii_uppercase()));
            let next = round(is_tty, &arg, &ans, &mut guesses, &mut results)?; //another round?
            if next {
                sub = (sub + 1) % list.len(); // change sub to get a new random answer
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
            if is_tty {
                ans = ans.to_ascii_uppercase();
            }
            answers.push(Some(ans.clone().to_ascii_uppercase()));
            round(is_tty, &arg, &ans, &mut guesses, &mut results)?;
        } else {
            loop {
                // when answers are from input
                if is_tty {
                    println!("Input the answer:");
                }
                let mut ans = word(&arg.word)?;
                if is_tty {
                    ans = ans.to_ascii_uppercase();
                }
                answers.push(Some(ans.clone().to_ascii_uppercase()));
                let next = round(is_tty, &arg, &ans, &mut guesses, &mut results)?;
                if !next {
                    break;
                }
            }
        }
    }
    Ok(())
}

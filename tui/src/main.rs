pub mod builtin_words;
pub mod common;

use {
    common::{
        args, change_keyboard, char2border, char2color, char2location, color, config, cut,
        diffcult, exist, random, stateload, statesave, stats, Config, ACCEPTSET, FINALSET, LENGTH,
        LIMIT,
    },
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    std::{io, time::Duration, vec},
    tui::{
        backend::{Backend, CrosstermBackend},
        layout::{Alignment, Constraint, Direction, Layout},
        style::{Color, Modifier, Style},
        text::Span,
        widgets::{Block, BorderType, Borders, Paragraph},
        Frame, Terminal,
    },
};

struct Information {
    pub message: String,
    pub mode: String,
    pub result: String,
    pub preference: String,
}
struct History {
    pub guesses: Vec<Vec<char>>,
    pub colors: Vec<Vec<char>>,
    pub buf: Vec<char>,
}

/*
    the basic structure of the main function and the methods to deal with blocks cite
    the contents on the website: "https://www.cnblogs.com/xueweihan/p/15992139.html",
    based on which I developed the logic and patterns for my tui
*/
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // set terminal
    let arg = config(&args()?)?;
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    // initialize
    let mut guesses: Vec<String> = vec![];
    let mut results: Vec<Option<(Option<bool>, Option<usize>)>> = vec![];
    let mut answers: Vec<Option<String>> = vec![];
    let mut hard = false;
    let mut history = History {
        guesses: vec![],
        colors: vec![],
        buf: vec![],
    };
    let mut information = Information {
        message: String::from(""),
        mode: String::from(""),
        result: String::from(""),
        preference: String::from(""),
    };
    let mut key_board = vec![
        vec!['X', 'X', 'X', 'X', 'X', 'X', 'X', 'X', 'X', 'X'],
        vec!['X', 'X', 'X', 'X', 'X', 'X', 'X', 'X', 'X'],
        vec!['X', 'X', 'X', 'X', 'X', 'X', 'X'],
    ];
    if arg.state.is_some() {
        // load from json
        stateload(&arg.state, &mut answers, &mut guesses, &mut results)?;
    }
    if arg.difficult.is_some() && arg.difficult.unwrap() {
        //difficult?
        hard = true;
        information.mode = String::from("Difficult mode");
    } else {
        information.mode = String::from("Simple mode");
    }
    let mut ans = String::new();
    // run the game
    if arg.word.is_some() {
        // word asigned, one round
        if exist(&arg.word.as_ref().unwrap(), unsafe { &FINALSET }) {
            ans = String::from(arg.word.as_ref().unwrap().to_ascii_uppercase());
            answers.push(Some(ans.clone()));
        }
        single_game(
            &ans.to_string(),
            hard,
            &mut guesses,
            &mut results,
            &mut terminal,
            &mut history,
            &mut None,
            &mut information,
            &mut key_board,
        )?;
    } else {
        // random word
        go_on_game(
            &arg,
            &mut answers,
            hard,
            &mut guesses,
            &mut results,
            &mut terminal,
            &mut history,
            &mut None,
            &mut information,
            &mut key_board,
        )?;
    }
    if arg.state.is_some() {
        //save to json
        statesave(&arg.state, &mut answers, &mut guesses, &mut results)?;
    }
    // end of the game, return terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
/*
function: to play a single round of game
input: ans: answer for this round of game
        hard: difficult mode?
        guesses: where stores all the valid guesses the player inputs
        results: where stores the results of every round of games, including win/fail, the times player tried
        terminal: the terminal
        history: the inputs and current state of a round of game
        inputs: last keycode
        information: the message and statistic for the player
        keyboard: the current state of colors of keyboard
output: None
*/
fn single_game<B: Backend>(
    ans: &String,
    hard: bool,
    guesses: &mut Vec<String>,
    results: &mut Vec<Option<(Option<bool>, Option<usize>)>>,
    terminal: &mut Terminal<B>,
    history: &mut History,
    input: &mut Option<char>,
    information: &mut Information,
    keyboard: &mut Vec<Vec<char>>,
) -> io::Result<()> {
    let mut total = 0; //number of guesses
    let mut exit = false;
    loop {
        terminal.draw(|f| ui(f, history, input, information, keyboard))?;
        if crossterm::event::poll(Duration::from_secs(1))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(ch) => {
                        if history.buf.len() != LENGTH - 1 {
                            //buf not full
                            if input.is_some() {
                                // input not the first letter of a word
                                history.buf.push(input.unwrap().to_ascii_uppercase());
                            }
                            *input = Some(ch.to_ascii_uppercase()); // the first letter of a word
                        }
                    }
                    KeyCode::Enter => {
                        if input.is_some() && history.buf.len() == LENGTH - 1 {
                            //input is a char, together with buf to form a guess
                            history.buf.push(input.unwrap().to_ascii_uppercase());
                        }
                        // a guess is formed
                        if history.buf.len() == LENGTH {
                            let guess = history.buf.iter().collect::<String>(); //get guess
                            if exist(&guess, unsafe { &ACCEPTSET }) == false {
                                information.message = String::from("INVALID");
                            } else if hard
                                && total > 0
                                && !diffcult(
                                    &guess,
                                    &guesses[guesses.len() - 1],
                                    &history.colors[total - 1],
                                )
                            {
                                information.message = String::from("INVALID");
                            } else {
                                guesses.push(guess.clone());
                                history
                                    .guesses
                                    .push(guess.clone().chars().collect::<Vec<char>>());
                                total += 1;
                                history.colors.push(color(&ans, &guess));
                                change_keyboard(&history.buf, &color(&ans, &guess), keyboard);
                            } //guess processed
                            history.buf = vec![];
                            *input = None;
                            if guess == *ans {
                                information.message = String::from(format!("Correct! "));
                                results.push(Some((Some(true), Some(total))));
                                break;
                            } else if total == LIMIT {
                                information.message = String::from(format!("Fail at {}. ", ans));
                                results.push(Some((Some(false), Some(LIMIT))));
                                break;
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        *input = history.buf.pop(); // if buf not empty, history will back
                    }
                    KeyCode::Esc => {
                        exit = true; // exit
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
    if exit {
        return Ok(());
    } else {
        // regular game over
        let ((win, lose, rounds), preference) = stats(&guesses, &results);
        information.result = format!(
            "win: {} / lose: {}    average trail when winning: {:.2}",
            win, lose, rounds
        );
        let mut text = String::new();
        for i in 0..preference.len() {
            text += &(format!("{}: {};  ", preference[i].0, preference[i].1));
        }
        information.preference = text;
        loop {
            // draw the final outcome
            terminal.draw(|f| ui(f, history, input, information, keyboard))?;
            if crossterm::event::poll(Duration::from_secs(1))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Esc => {
                            break;
                        }
                        _ => {
                            information.message = String::from("click 'ESC' to exit");
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/*
function: to play sequent rounds of game
input: ans: answer for this round of game
        hard: difficult mode?
        guesses: where stores all the valid guesses the player inputs
        results: where stores the results of every round of games, including win/fail, the times player tried
        terminal: the terminal
        history: the inputs and current state of a round of game
        inputs: last keycode
        information: the message and statistic for the player
        keyboard: the current state of colors of keyboard
output: None
*/
fn go_on_game<B: Backend>(
    arg: &Config,
    answers: &mut Vec<Option<String>>,
    hard: bool,
    guesses: &mut Vec<String>,
    results: &mut Vec<Option<(Option<bool>, Option<usize>)>>,
    terminal: &mut Terminal<B>,
    history: &mut History,
    input: &mut Option<char>,
    information: &mut Information,
    keyboard: &mut Vec<Vec<char>>,
) -> io::Result<()> {
    let list = 0..unsafe { FINALSET.len() };
    let mut list = list.collect::<Vec<usize>>();
    let mut sub = random(&arg.day, &arg.seed, &mut list);
    loop {
        let ans = unsafe { &FINALSET[list[sub]] }
            .to_string()
            .to_ascii_uppercase();
        answers.push(Some(ans.clone()));
        information.message = String::from("New Game On");
        let mut total = 0;
        let mut exit = false;
        loop {
            terminal.draw(|f| ui(f, history, input, information, keyboard))?;
            if crossterm::event::poll(Duration::from_secs(1))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char(ch) => {
                            if history.buf.len() != LENGTH - 1 {
                                if input.is_some() {
                                    history.buf.push(input.unwrap().to_ascii_uppercase());
                                }
                                *input = Some(ch.to_ascii_uppercase());
                            }
                        }
                        KeyCode::Enter => {
                            // basically the same as single
                            if input.is_some() && history.buf.len() == LENGTH - 1 {
                                history.buf.push(input.unwrap().to_ascii_uppercase());
                            }
                            if history.buf.len() == LENGTH {
                                let guess = history.buf.iter().collect::<String>();
                                if exist(&guess, unsafe { &ACCEPTSET }) == false {
                                    information.message = String::from("INVALID");
                                } else if hard
                                    && total > 0
                                    && !diffcult(
                                        &guess,
                                        &guesses[guesses.len() - 1],
                                        &history.colors[total - 1],
                                    )
                                {
                                    information.message = String::from("INVALID");
                                } else {
                                    guesses.push(guess.clone());
                                    history
                                        .guesses
                                        .push(guess.clone().chars().collect::<Vec<char>>());
                                    total += 1;
                                    history.colors.push(color(&ans, &guess));
                                    change_keyboard(&history.buf, &color(&ans, &guess), keyboard);
                                    information.message = String::from("ACCEPTED");
                                }
                                history.buf = vec![];
                                *input = None;
                                if guess == ans {
                                    information.message = String::from(format!(
                                        "Correct! Input 'Y' to play again, click 'ESC' to exit."
                                    ));
                                    results.push(Some((Some(true), Some(total))));
                                    break;
                                } else if total == LIMIT {
                                    information.message = String::from(format!(
                                        "Fail at {}. Input 'Y' to play again, click 'ESC' to exit.",
                                        ans
                                    ));
                                    results.push(Some((Some(false), Some(LIMIT))));
                                    break;
                                }
                            }
                        }
                        KeyCode::Backspace => {
                            *input = history.buf.pop();
                        }
                        KeyCode::Esc => {
                            exit = true;
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
        if exit {
            break;
        } else {
            *keyboard = vec![
                vec!['X', 'X', 'X', 'X', 'X', 'X', 'X', 'X', 'X', 'X'],
                vec!['X', 'X', 'X', 'X', 'X', 'X', 'X', 'X', 'X'],
                vec!['X', 'X', 'X', 'X', 'X', 'X', 'X'],
            ]
            .clone();
            let ((win, lose, rounds), preference) = stats(&guesses, &results);
            information.result = format!(
                "win: {} / lose: {}    average trail when winning: {:.2}",
                win, lose, rounds
            );
            let mut text = String::new();
            for i in 0..preference.len() {
                text += &(format!("{}: {}; ", preference[i].0, preference[i].1));
            }
            information.preference = text;
            loop {
                terminal.draw(|f| ui(f, history, input, information, keyboard))?;
                if crossterm::event::poll(Duration::from_secs(1))? {
                    if let Event::Key(key) = event::read()? {
                        match key.code {
                            KeyCode::Char('y') => {
                                sub = (sub + 1) % unsafe { FINALSET.len() };
                                *history = History {
                                    guesses: vec![],
                                    colors: vec![],
                                    buf: vec![],
                                };
                                information.message = String::from("");
                                information.result = String::from("");
                                information.preference = String::from("");
                                break; // reinitialization
                            }
                            KeyCode::Esc => {
                                exit = true;
                                break;
                            }
                            _ => {
                                information.message =
                                    String::from("Input 'Y' for another game, click 'ESC' to exit");
                            }
                        }
                    }
                }
            }
        }
        if exit {
            break;
        }
    }
    Ok(())
}

/*
function: to draw the situation on the terminal
input: f: the terminal
        history: the inputs and current state of a round of game
        inputs: last keycode
        information: the message and statistic for the player
        keyboard: the current state of colors of keyboard
output: None
*/
fn ui<B: Backend>(
    f: &mut Frame<B>,
    history: &History,
    input: &Option<char>,
    information: &Information,
    key_board: &Vec<Vec<char>>,
) {
    let kboard = vec![
        vec!['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P'],
        vec!['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L'],
        vec!['Z', 'X', 'C', 'V', 'B', 'N', 'M'],
    ];
    let all_chunks = Layout::default()
        .constraints([Constraint::Percentage(67), Constraint::Percentage(33)].as_ref())
        .direction(Direction::Vertical)
        .split(f.size()); //terminal split into up:down1 2:1
    let up_chunks = Layout::default()
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .direction(Direction::Horizontal)
        .split(all_chunks[0]); // up split into left:right 1:1
    let paragraph = Paragraph::new(Span::styled(
        "",
        Style::default().add_modifier(Modifier::BOLD),
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .border_type(BorderType::Rounded)
            .title("History"),
    )
    .alignment(tui::layout::Alignment::Left);
    f.render_widget(paragraph, up_chunks[0]); // upleft history
    let paragraph = Paragraph::new(Span::styled(
        "",
        Style::default().add_modifier(Modifier::BOLD),
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("information")
            .title_alignment(tui::layout::Alignment::Center),
    )
    .alignment(tui::layout::Alignment::Left);
    f.render_widget(paragraph, up_chunks[1]); // upright information
    let paragraph = Paragraph::new("")
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White))
                .border_type(BorderType::Rounded)
                .title("Keyboard")
                .title_alignment(tui::layout::Alignment::Center),
        );
    f.render_widget(paragraph, all_chunks[1]); // down keynoard
    let words = cut(up_chunks[0], 4, Direction::Vertical);
    let words = cut(words, 5, Direction::Horizontal);
    let mut words = Layout::default()
        .constraints([Constraint::Percentage(16); 7].as_ref())
        .direction(Direction::Vertical)
        .split(words); // split history into 6 lines, each for a guess
    for i in 0..LIMIT {
        let paragraph = Paragraph::new(Span::styled(
            "",
            Style::default().add_modifier(Modifier::BOLD),
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("guess ".to_string() + &(i + 1).to_string())
                .title_alignment(tui::layout::Alignment::Center),
        )
        .alignment(tui::layout::Alignment::Center);
        f.render_widget(paragraph, words[i]);

        words[i] = cut(words[i], 3, Direction::Horizontal);
        words[i] = Layout::default()
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Length(2),
                    Constraint::Length(1),
                ]
                .as_ref(),
            )
            .horizontal_margin(6)
            .direction(Direction::Vertical)
            .split(words[i])[1];
    } // beatify each guess
    let informations = cut(up_chunks[1], 5, Direction::Vertical);
    let informations = cut(informations, 5, Direction::Horizontal);
    let informations = Layout::default()
        .constraints([Constraint::Percentage(25); 5].as_ref())
        .direction(Direction::Vertical)
        .split(informations); // split information into 4 lines
    let keyboard = cut(all_chunks[1], 10, Direction::Vertical);
    let keyboard = cut(keyboard, 5, Direction::Horizontal);
    let keyboard = Layout::default()
        .constraints([Constraint::Percentage(33); 4].as_ref())
        .vertical_margin(1)
        .direction(Direction::Vertical)
        .split(keyboard); // split keyboard into 6 lines

    for i in 0..history.guesses.len() {
        let mut letters = Layout::default()
            .constraints([Constraint::Percentage(20); 6].as_ref())
            .direction(Direction::Horizontal)
            .split(words[i]);
        for j in 0..LENGTH {
            letters[j] = cut(letters[j], 5, Direction::Horizontal);
            let paragraph = Paragraph::new(Span::styled(
                history.guesses[i][j].to_string(),
                Style::default().add_modifier(Modifier::BOLD),
            ))
            .block(Block::default().style(Style::default().bg(char2color(history.colors[i][j]))))
            .alignment(Alignment::Center);
            f.render_widget(paragraph, letters[j]);
        }
    } // draw history
    if history.guesses.len() < LIMIT {
        // if game not over
        let mut letters = Layout::default()
            .constraints([Constraint::Percentage(20); 6].as_ref())
            .direction(Direction::Horizontal)
            .split(words[history.guesses.len()]);

        for j in 0..history.buf.len() {
            letters[j] = cut(letters[j], 5, Direction::Horizontal);
            let paragraph = Paragraph::new(Span::styled(
                history.buf[j].to_string(),
                Style::default().add_modifier(Modifier::BOLD),
            ))
            .block(Block::default().style(Style::default().bg(Color::Gray)))
            .alignment(Alignment::Center);
            f.render_widget(paragraph, letters[j]);
        } // draw buff

        if input.is_some() {
            // draw input, if input is not the first letter of a guess
            let j = history.buf.len();
            letters[j] = cut(letters[j], 5, Direction::Horizontal);
            let paragraph = Paragraph::new(Span::styled(
                input.unwrap().to_string(),
                Style::default().add_modifier(Modifier::BOLD),
            ))
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(Color::Blue))
                    .border_type(BorderType::Rounded)
                    .style(Style::default().bg(Color::White)),
            )
            .alignment(Alignment::Center); // blue underline
            f.render_widget(paragraph, letters[j]);
        } else {
            // draw input, if input is the first letter of a guess
            let j = history.buf.len();
            letters[j] = cut(letters[j], 5, Direction::Horizontal);
            let paragraph = Paragraph::new(Span::styled(
                "",
                Style::default().add_modifier(Modifier::BOLD),
            ))
            .block(Block::default().style(Style::default().bg(Color::White)))
            .alignment(Alignment::Center);
            f.render_widget(paragraph, letters[j]);
        }

        for j in history.buf.len() + 1..LENGTH {
            // draw after the input
            letters[j] = cut(letters[j], 5, Direction::Horizontal);
            let paragraph = Paragraph::new(Span::styled(
                "",
                Style::default().add_modifier(Modifier::BOLD),
            ))
            .block(Block::default().style(Style::default().bg(Color::Gray)))
            .alignment(Alignment::Center);
            f.render_widget(paragraph, letters[j]);
        }
    }

    let title = vec!["message", "mode", "statistic", "words preferred"];
    let text = vec![
        information.message.clone(),
        information.mode.clone(),
        information.result.clone(),
        information.preference.clone(),
    ];
    for i in 0..4 {
        let paragraph = Paragraph::new(Span::styled(
            &text[i],
            Style::default().add_modifier(Modifier::BOLD),
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title[i])
                .title_alignment(tui::layout::Alignment::Center),
        )
        .alignment(tui::layout::Alignment::Center);
        f.render_widget(paragraph, informations[i]);
    } // draw the information

    let mut bordercoler = key_board.clone();
    if input.is_some() {
        let (h, v) = char2location(input.unwrap());
        bordercoler[h][v] = 'B';
    } // input not the first letter of a guess, the selected letter on keyboard bordered blue

    let mut line = Layout::default()
        .constraints([Constraint::Percentage(10); 11].as_ref())
        .direction(Direction::Horizontal)
        .split(keyboard[0]);
    for i in 0..10 {
        line[i] = cut(line[i], 5, Direction::Horizontal);
        let paragraph = Paragraph::new(Span::styled(
            (kboard[0][i]).to_string(),
            Style::default().add_modifier(Modifier::BOLD),
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(char2border(bordercoler[0][i])))
                .border_type(BorderType::Rounded)
                .style(Style::default().bg(char2color(key_board[0][i]))),
        )
        .alignment(Alignment::Center);
        f.render_widget(paragraph, line[i]);
    } // draw the first line of the keyboard
    let mut line = Layout::default()
        .constraints([Constraint::Percentage(11); 10].as_ref())
        .horizontal_margin(4)
        .direction(Direction::Horizontal)
        .split(keyboard[1]);
    for i in 0..9 {
        line[i] = cut(line[i], 5, Direction::Horizontal);
        let paragraph = Paragraph::new(Span::styled(
            (kboard[1][i]).to_string(),
            Style::default().add_modifier(Modifier::BOLD),
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(char2border(bordercoler[1][i])))
                .border_type(BorderType::Rounded)
                .style(Style::default().bg(char2color(key_board[1][i]))),
        )
        .alignment(Alignment::Center);
        f.render_widget(paragraph, line[i]);
    } // the second line
    let mut line = Layout::default()
        .constraints([Constraint::Percentage(12); 8].as_ref())
        .horizontal_margin(10)
        .direction(Direction::Horizontal)
        .split(keyboard[2]);
    for i in 0..7 {
        line[i] = cut(line[i], 5, Direction::Horizontal);
        let paragraph = Paragraph::new(Span::styled(
            (kboard[2][i]).to_string(),
            Style::default().add_modifier(Modifier::BOLD),
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(char2border(bordercoler[2][i])))
                .border_type(BorderType::Rounded)
                .style(Style::default().bg(char2color(key_board[2][i]))),
        )
        .alignment(Alignment::Center);
        f.render_widget(paragraph, line[i]);
    } // the third
}
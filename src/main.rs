use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute, queue,
    style::{Color, SetBackgroundColor, SetForegroundColor},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, SetTitle},
    QueueableCommand,
};
use hangman_rust::constants;
use rand::seq::SliceRandom;
use std::{
    fs,
    io::{self, stdout, Write},
};

struct CleanUp;

//? Drop trait is used to run code when a value goes out of scope
impl Drop for CleanUp {
    fn drop(&mut self) {
        execute!(stdout(), LeaveAlternateScreen).unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();
    }
}

fn main() -> io::Result<()> {
    //? Declare an instance of the class
    //? When the main function ends, the instance of the class will be dropped
    let _cleanup = CleanUp;

    execute!(
        //? This can be used to draw stuff in the terminal
        stdout(),
        EnterAlternateScreen,
        SetTitle("Hangman - Coco Computer Style"),
        SetBackgroundColor(Color::DarkGreen),
        SetForegroundColor(Color::Black)
    )?;

    //? Allows the program to read input directly as it is typed
    //? Ignores console commands like Ctrl+C
    crossterm::terminal::enable_raw_mode()?;

    loop {
        let secret_word = select_random_word()?;
        //? Holds the word and guesses
        let state = GameState::new(secret_word);
        let play_again = main_loop(state)?;
        if !play_again {
            break;
        }
    }

    Ok(())
}

struct GameState {
    secret_word: String,
    guessed_letters: Vec<char>,
    incorrect_guesses: u8,
}

impl GameState {
    fn new(secret_word: String) -> Self {
        Self {
            secret_word: secret_word.to_lowercase(),
            guessed_letters: Vec::new(),
            incorrect_guesses: 0,
        }
    }

    fn current_display(&self) -> String {
        self.secret_word
            .chars()
            .map(|c| {
                if self.guessed_letters.contains(&c) {
                    c.to_string() // Convert char to String
                } else {
                    "_".to_string() // Convert underscore to String
                }
            })
            .collect::<Vec<String>>() // Collect into Vec<String>
            .join(" ")
    }
}

fn draw_interface(state: &GameState) -> io::Result<()> {
    let mut stdout = stdout();

    //? Clear screen and reset cursor
    stdout
        .queue(cursor::Hide)?
        .queue(crossterm::terminal::Clear(
            crossterm::terminal::ClearType::All,
        ))?;

    //? Draw hangman
    let stage_index = state.incorrect_guesses.min(6) as usize;
    queue!(stdout, cursor::MoveTo(5, 2))?;
    print!("{}", constants::HANGMAN_STAGES[stage_index]);

    //? Draw word display
    let display_word = state.current_display();
    queue!(stdout, cursor::MoveTo(10, 15))?;
    print!("Word: {}", display_word);

    //? Draw guessed letters
    queue!(stdout, cursor::MoveTo(10, 17))?;
    print!(
        "Guessed: {}",
        state.guessed_letters.iter().collect::<String>()
    );

    //? Clear the buffer
    stdout.flush()?;
    Ok(())
}

fn get_player_guess() -> io::Result<char> {
    //? Loop until a valid alphabetic character is entered
    loop {
        if let Event::Key(key_event) = event::read()? {
            if let KeyCode::Char(c) = key_event.code {
                let c = c.to_ascii_lowercase();
                if c.is_ascii_alphabetic() {
                    return Ok(c);
                }
            }
        }
    }
}

fn main_loop(mut state: GameState) -> io::Result<bool> {
    loop {
        draw_interface(&state)?;

        let won = state
            .secret_word
            .chars()
            .all(|c| state.guessed_letters.contains(&c));

        let lost = state.incorrect_guesses >= 6;

        if won || lost {
            break;
        }

        //? Get player input
        let guess = get_player_guess()?;

        //? Update game state
        if !state.guessed_letters.contains(&guess) {
            state.guessed_letters.push(guess);
            if !state.secret_word.contains(guess) {
                state.incorrect_guesses += 1;
            }
        }
    }

    let won = state
        .secret_word
        .chars()
        .all(|c| state.guessed_letters.contains(&c));
    draw_game_over_screen(won, &state.secret_word)?;
    prompt_play_again()
}

fn draw_game_over_screen(win: bool, secret_word: &str) -> io::Result<()> {
    let mut stdout = stdout();

    stdout
        .queue(cursor::Hide)?
        .queue(crossterm::terminal::Clear(
            crossterm::terminal::ClearType::All,
        ))?;

    let message = if win {
        "YOU WIN! 🎉"
    } else {
        "YOU LOSE! 💀"
    };

    queue!(stdout, cursor::MoveTo(10, 5))?;
    print!("{}", message);

    queue!(stdout, cursor::MoveTo(10, 6))?;
    print!("The word was: {}", secret_word);

    queue!(stdout, cursor::MoveTo(10, 8))?;
    print!("Play again? (Y/N)");

    stdout.flush()?;
    Ok(())
}

fn prompt_play_again() -> io::Result<bool> {
    let mut result = None;

    //? Get valid Y/N input
    while result.is_none() {
        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => result = Some(true),
                KeyCode::Char('n') | KeyCode::Char('N') => result = Some(false),
                _ => continue,
            }
        }
    }

    //? Crossterm-specific input buffer flushing
    while event::poll(std::time::Duration::from_secs(0))? {
        //? Clear any remaining input events
        let _ = event::read()?;
    }

    Ok(result.unwrap())
}

fn select_random_word() -> io::Result<String> {
    let words = fs::read_to_string("words.txt")?;
    let words: Vec<&str> = words.lines().collect();
    let word = words.choose(&mut rand::thread_rng()).unwrap_or(&"hangman");
    Ok(word.to_string())
}

use crossterm::event::{self, Event, KeyCode};
use crossterm::style::{Color, SetBackgroundColor, SetForegroundColor};
use crossterm::{cursor, queue, QueueableCommand};
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, SetTitle},
};
use rand::seq::SliceRandom;
use std::fs;
use std::io::{self, stdout, Write};

fn setup_colors() -> io::Result<()> {
    execute!(
        stdout(),
        SetBackgroundColor(Color::Black),
        SetForegroundColor(Color::DarkGreen)
    )?;
    Ok(())
}

struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        execute!(stdout(), LeaveAlternateScreen).unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();
    }
}

fn main() -> io::Result<()> {
    let _cleanup = CleanUp;

    let mut stdout = stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        SetTitle("Hangman - Coco Computer Style"),
    )?;
    crossterm::terminal::enable_raw_mode()?;
    setup_colors()?;

    let secret_word = select_random_word()?;
    let state = GameState::new(secret_word);

    main_loop(state)?;

    // Wait for keypress before exiting
    println!("\nPress any key to exit...");
    let _ = get_player_guess()?;

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
            .join(" ") // Now join works!
    }
}

const HANGMAN_STAGES: [&str; 7] = [
    r#"
  +---+
  |   |
      |
      |
      |
      |
========="#,
    r#"
  +---+
  |   |
  O   |
      |
      |
      |
========="#,
    r#"
  +---+
  |   |
  O   |
  |   |
      |
      |
========="#,
    r#"
  +---+
  |   |
  O   |
 /|   |
      |
      |
========="#,
    r#"
  +---+
  |   |
  O   |
 /|\  |
      |
      |
========="#,
    r#"
  +---+
  |   |
  O   |
 /|\  |
 /    |
      |
========="#,
    r#"
  +---+
  |   |
  O   |
 /|\  |
 / \  |
      |
========="#,
];

fn draw_interface(state: &GameState) -> io::Result<()> {
    let mut stdout = stdout();

    // Clear screen and reset cursor
    stdout
        .queue(cursor::Hide)?
        .queue(crossterm::terminal::Clear(
            crossterm::terminal::ClearType::All,
        ))?;

    // Draw hangman
    let stage_index = state.incorrect_guesses.min(6) as usize;
    queue!(stdout, cursor::MoveTo(5, 2))?;
    print!("{}", HANGMAN_STAGES[stage_index]);

    // Draw word display
    let display_word = state.current_display();
    queue!(stdout, cursor::MoveTo(10, 15))?;
    print!("Word: {}", display_word);

    // Draw guessed letters
    queue!(stdout, cursor::MoveTo(10, 17))?;
    print!(
        "Guessed: {}",
        state.guessed_letters.iter().collect::<String>()
    );

    stdout.flush()?;
    Ok(())
}

fn get_player_guess() -> io::Result<char> {
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

fn main_loop(mut state: GameState) -> io::Result<()> {
    loop {
        draw_interface(&state)?;

        // Check win/lose conditions
        if state.current_display().replace(" ", "") == state.secret_word {
            draw_interface(&state)?;
            println!("\nYou win!");
            break;
        }

        if state.incorrect_guesses >= 6 {
            draw_interface(&state)?;
            println!("\nYou lose! The word was: {}", state.secret_word);
            break;
        }

        // Get player input
        let guess = get_player_guess()?;

        // Update game state
        if !state.guessed_letters.contains(&guess) {
            state.guessed_letters.push(guess);
            if !state.secret_word.contains(guess) {
                state.incorrect_guesses += 1;
            }
        }
    }
    Ok(())
}

fn select_random_word() -> io::Result<String> {
    let words = fs::read_to_string("words.txt")?;
    let words: Vec<&str> = words.lines().collect();
    let word = words.choose(&mut rand::thread_rng()).unwrap_or(&"hangman");
    Ok(word.to_string())
}

use std::io;

fn main() {
    println!("Welcome to Hangman!");

    let secret_word = "rust";
    let mut guessed_letters = vec!['_'; secret_word.len()];
    let mut attempts = 6;

    while attempts > 0 && guessed_letters.contains(&'_') {
        println!(
            "Current word: {}",
            guessed_letters.iter().collect::<String>()
        );
        println!("Attempts left: {}", attempts);
        println!("Guess a letter:");

        let mut guess = String::new();
        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");

        let guess_char = guess.trim().chars().next().unwrap();

        if secret_word.contains(guess_char) {
            for (i, c) in secret_word.chars().enumerate() {
                if c == guess_char {
                    guessed_letters[i] = guess_char;
                }
            }
        } else {
            attempts -= 1;
            println!("Incorrect guess!");
        }
    }

    if !guessed_letters.contains(&'_') {
        println!("Congratulations! You guessed the word: {}", secret_word);
    } else {
        println!("You ran out of attempts. The word was: {}", secret_word);
    }
}

use std::io::{stderr, stdout, Write};

use crossterm::{
    cursor,
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Color, Print, PrintStyledContent, SetForegroundColor, Stylize},
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    QueueableCommand,
};

/// Prompt the user for text input.
pub(crate) fn prompt() -> String {
    stdout().flush().expect("failed to flush stdout");
    stderr().flush().expect("failed to flush stderr");

    let mut line = String::new();
    while let Event::Key(KeyEvent { code, .. }) = crossterm::event::read().unwrap() {
        match code {
            KeyCode::Enter => {
                break;
            }
            KeyCode::Char(c) => {
                line.push(c);
            }
            _ => {}
        }
    }

    line
}

/// Prompt the user for a single character response. Useful for asking yes or no questions.
///
/// `chars` should be all lowercase characters, with at most 1 uppercase character. The uppercase character is the default answer if no answer is provided.
pub(crate) fn prompt_char(text: &str, chars: &str) -> char {
    loop {
        let _ = stderr().queue(Print(format!("{} [{}] ", text, chars)));
        let _ = stderr().flush();
        let input = prompt();
        if let Ok(c) = prompt_char_impl(input, chars) {
            return c;
        }
    }
}

#[inline(always)]
fn prompt_char_impl<T>(input: T, chars: &str) -> anyhow::Result<char>
where
    T: Into<String>,
{
    let uppers = chars.replace(char::is_lowercase, "");
    if uppers.len() > 1 {
        panic!("Invalid chars for prompt_char. Maximum 1 uppercase letter is allowed.");
    }
    let default_answer: Option<char> = if uppers.len() == 1 {
        Some(uppers.chars().collect::<Vec<char>>()[0].to_ascii_lowercase())
    } else {
        None
    };

    let answer: String = input.into().to_ascii_lowercase();

    if answer.is_empty() {
        if let Some(a) = default_answer {
            return Ok(a);
        } else {
            anyhow::bail!("no valid answer")
        }
    } else if answer.len() > 1 {
        anyhow::bail!("answer too long")
    }

    let answer_char = answer.chars().collect::<Vec<char>>()[0];
    if chars.to_ascii_lowercase().contains(answer_char) {
        return Ok(answer_char);
    }

    anyhow::bail!("no valid answer")
}

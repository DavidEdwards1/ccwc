/// Library crate for ccwc, a wc clone built in Rust.

use std::io::Read;
use std::{fs, io};
use std::error::Error;

use::clap::Parser;

/// A wc clone built in Rust.
#[derive(Parser, Debug)]
pub struct Cli {
    /// Count the number of bytes the input. If `-m` is specified then that
    /// option overrides this one.
    #[arg(short = 'c')]
    byte_count: bool,

    /// Count the number of words in the input. Words are defined as being
    /// separated by whitespace characters.
    #[arg(short = 'w')]
    word_count: bool,

    /// Count the number of lines in the input. Lines are separated by the
    /// newline character `\n`.
    #[arg(short = 'l')]
    line_count: bool,

    /// Count the number of characters in the string. If the current locale does
    /// not support mutlibyte characters then this will be the same as the byte
    /// count. Use of `-m` will override any usage of `-c`. Note that this is
    /// different from `wc` where the last of the two flags specified will be
    /// used. Here `-m` is always preferred to `-c`.
    #[arg(short = 'm')]
    char_count: bool,

    /// If provided this should be the name of a file to read in as input. If
    /// not provided then stdin will be used as the input.
    filename: Option<String>,

}

impl Cli {
    /// Returns true if any command line flag has been set, false otherwise
    fn any_flag_set(&self) -> bool {
        self.byte_count || self.word_count || self.line_count || self.char_count
    }
}

/// An enum that breaks out the options for counting characters or bytes
#[derive(Debug)]

enum CharCount {
    Chars,
    Bytes,
    None,
}

/// A struct that holds the configuration options for the counts
#[derive(Debug)]
struct CountConfig {
    count_chars: CharCount,
    count_words: bool,
    count_lines: bool,
    filename: Option<String>,
}

impl CountConfig {
    /// Create a CountConfig from the given cli options
    pub fn from_cli(cli: &Cli) -> CountConfig {
        return  CountConfig {
            count_chars: if cli.char_count {
                CharCount::Chars
            } else if cli.byte_count || !cli.any_flag_set() {
                CharCount::Bytes
            } else {
                CharCount::None
            },
            count_lines: cli.line_count || !cli.any_flag_set(),
            count_words: cli.word_count || !cli.any_flag_set(),
            filename: cli.filename.clone(),
        }
    }
}

/// A struct to hold the counts of bytes or characters, words, and lines in a file and the filename
/// Each of the counts is an optional usize and the filename is a required string
#[derive(Debug)]
struct Counter {
    config: CountConfig,
    byte_or_char_count: Option<usize>,
    word_count: Option<usize>,
    line_count: Option<usize>,
}

impl Counter {
    /// A function to create a new Counter struct with the given filename
    /// and all counts set to None
    fn new(config: CountConfig) -> Counter {
        Counter {
            config,
            byte_or_char_count: None,
            word_count: None,
            line_count: None,
        }
    }

    /// Actually calculates the counts specified in the config of the Counter.
    /// Mutates the Counter to add the counts to it.
    fn count(mut self, contents: &String) -> Counter {
        match self.config.count_chars {
            CharCount::Chars => self.byte_or_char_count = Some(count_characters(contents)),
            CharCount::Bytes => self.byte_or_char_count = Some(count_bytes(contents)),
            CharCount::None => self.byte_or_char_count = None,
        }

        if self.config.count_lines {
            self.line_count = Some(count_lines(contents));
        }

        if self.config.count_words {
            self.word_count = Some(count_words(contents));
        }

        self
    }

    /// A function to create a formatted output string from the Counter struct
    /// The output string is formatted as follows:
    /// line_count word_count byte_count filename
    /// where each count is right-aligned in a column of width a multiple of 4
    /// and each column is separated by a space
    fn as_string(&self) -> String {
        let mut output = String::new();

        if let Some(line_count) = self.line_count {
            output.push_str(&format_output(line_count.to_string()));
        }
        if let Some(word_count) = self.word_count {
            output.push_str(&format_output(word_count.to_string()));
        }
        if let Some(byte_count) = self.byte_or_char_count {
            output.push_str(&format_output(byte_count.to_string()));
        }

        if let Some(filename) = &self.config.filename {
            output.push_str(&format!(" {}",filename));
        }

        output
    }
}

/// Formats a string so that it is right-aligned in a column of width a multiple of 4
fn format_output(input_string: String) -> String {
    let column_width: usize = 4 *((input_string.len() / 4) + 1);
    format!("{input_string: >column_width$}", column_width=column_width)
}

/// Count the number of bytes in a string
fn count_bytes(input_string: &str) -> usize {
    input_string.len()
}

/// Count the number of characters in a string
fn count_characters(input_string: &str) -> usize {
    input_string.chars().count()
}

/// Count the number of lines in a string
fn count_lines(input_string: &str) -> usize {
    input_string.lines().count()
}

/// Count the number of words in a string
fn count_words(input_string: &str) -> usize {
    input_string.split_whitespace().count()
}

/// The public interface to the library. Takes in a Cli struct and runs the
/// counts specified therein reading from a file or stdin as required.
pub fn run(cli: Cli) -> Result<String, Box<dyn Error>>{
    let mut contents = String::new();

    match &cli.filename {
        Some(filename) => {
            contents = fs::read_to_string(filename)?;
        }
        None => {
            io::stdin().read_to_string(&mut contents)?;
        }
    }

    let count_config = CountConfig::from_cli(&cli);
    let counter = Counter::new(count_config).count(&contents);

    Ok(counter.as_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_byte_count() {
        let config = CountConfig::from_cli(
            &Cli {
                filename: None,
                byte_count: true,
                char_count: false,
                line_count: false,
                word_count: false,
            }
        );
        let result = Counter::new(config).count(&"hello, world".to_owned());

        assert_eq!(
            result.byte_or_char_count.unwrap(),
            12
        );
        assert_eq!(
            result.line_count,
            None
        );
        assert_eq!(
            result.word_count,
            None
        );
    }

    #[test]
    fn test_counter_char_count() {
        let config = CountConfig::from_cli(
            &Cli {
                filename: None,
                byte_count: false,
                char_count: true,
                line_count: false,
                word_count: false,
            }
        );
        let result = Counter::new(config).count(&"hello, world".to_owned());

        assert_eq!(
            result.byte_or_char_count.unwrap(),
            12
        );
        assert_eq!(
            result.line_count,
            None
        );
        assert_eq!(
            result.word_count,
            None
        );
    }

    #[test]
    fn test_counter_line_count() {
        let config = CountConfig::from_cli(
            &Cli {
                filename: None,
                byte_count: false,
                char_count: false,
                line_count: true,
                word_count: false,
            }
        );
        let result = Counter::new(config).count(&"hello, world".to_owned());

        assert_eq!(
            result.byte_or_char_count,
            None
        );
        assert_eq!(
            result.line_count.unwrap(),
            1
        );
        assert_eq!(
            result.word_count,
            None
        );
    }

    #[test]
    fn test_counter_word_count() {
        let config = CountConfig::from_cli(
            &Cli {
                filename: None,
                byte_count: false,
                char_count: false,
                line_count: false,
                word_count: true,
            }
        );
        let result = Counter::new(config).count(&"hello, world".to_owned());

        assert_eq!(
            result.byte_or_char_count,
            None
        );
        assert_eq!(
            result.line_count,
            None
        );
        assert_eq!(
            result.word_count.unwrap(),
            2
        );
    }

    #[test]
    fn test_count_bytes() {
        assert_eq!(count_bytes(""), 0);
        assert_eq!(count_bytes("Hello, world!"), 13);
        assert_eq!(count_bytes("こんにちは"), 15);
    }

    #[test]
    fn test_count_lines() {
        assert_eq!(count_lines(""), 0);
        assert_eq!(count_lines("Hello\nworld"), 2);
        assert_eq!(count_lines("Line 1\nLine 2\nLine 3"), 3);
    }

    #[test]
    fn test_count_words() {
        assert_eq!(count_words(""), 0);
        assert_eq!(count_words("Hello,\nworld!"), 2);
        assert_eq!(count_words("This is a sentence."), 4);
    }
}

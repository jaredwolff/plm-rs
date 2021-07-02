// License: Apache 2.0
// Author: Jared Wolff

use std::io::{BufRead, Write};

pub struct Prompt<R, W> {
    pub reader: R,
    pub writer: W,
}

// Returns a string response. Newline removed.
impl<R, W> Prompt<R, W>
where
    R: BufRead,
    W: Write,
{
    pub fn ask_text_entry(&mut self, question: &str) -> String {
        // Ask the question
        write!(&mut self.writer, "{}", question).expect("Unable to write");
        self.writer.flush().expect("Unable to flush");

        // Parse input
        let mut input = String::new();
        self.reader.read_line(&mut input).unwrap();
        let response = &input[..(input.len() - 1)]; // Drop the newline character
        let response = response.trim();

        // Return a copy
        response.to_string()
    }

    pub fn ask_yes_no_question(&mut self, question: &str) -> bool {
        // Ask the question
        write!(&mut self.writer, "{} (y/n) ", question).expect("Unable to write!");
        self.writer.flush().expect("Unable to flush output!");

        // Parse input
        let mut input = String::new();
        self.reader.read_line(&mut input).unwrap();
        let ch = input.chars().next().unwrap();

        // Return result
        ch == 'y'
    }
}

#[test]
fn test_text_entry_with_whitespace() {
    let input = b"I love cookies!   \n";
    let mut output = Vec::new();

    let answer = {
        let mut prompt = Prompt {
            reader: &input[..],
            writer: &mut output,
        };

        prompt.ask_text_entry("Do you like cookies?")
    };

    let output = String::from_utf8(output).expect("Not UTF-8");

    assert_eq!("Do you like cookies?", output); // output to stdout
    assert_eq!("I love cookies!", answer); // input
}

#[test]
fn test_yes_no_expect_true() {
    let input = b"y\n";
    let mut output = Vec::new();

    let answer = {
        let mut prompt = Prompt {
            reader: &input[..],
            writer: &mut output,
        };

        prompt.ask_yes_no_question("Would you like to do things?")
    };

    let output = String::from_utf8(output).expect("Not UTF-8");

    assert_eq!("Would you like to do things? (y/n) ", output);
    assert_eq!(true, answer);
}

#[test]
fn test_yes_no_expect_false() {
    let input = b"\n";
    let mut output = Vec::new();

    let answer = {
        let mut prompt = Prompt {
            reader: &input[..],
            writer: &mut output,
        };

        prompt.ask_yes_no_question("Would you like to do things?")
    };

    let output = String::from_utf8(output).expect("Not UTF-8");

    assert_eq!("Would you like to do things? (y/n) ", output);
    assert_eq!(false, answer);
}

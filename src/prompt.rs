
use std::io::{stdin,stdout,Write};

pub fn ask_yes_no_question(question: &String) -> bool {

  // Ask the question
  print!("{} (y/n) ", question);
  stdout().flush().expect("Unable to flush output!");

  // Parse input
  let mut input = String::new();
  stdin().read_line(&mut input).unwrap();
  let ch = input.chars().next().unwrap();

  // Return result
  if ch == 'y' {
    true
  } else {
    false
  }

}


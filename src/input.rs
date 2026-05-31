// src/input.rs

/// input module for reading user input

use std::io::{self, Write}; // importing std io module

/// Reads one line of input from the user.
pub fn input() -> String { // function to read user input

    io::stdout().flush().unwrap(); // flush stdout for immediate output

    let mut buf = String::new(); // create a new strng

    io::stdin().read_line(&mut buf).expect("Failed to read input bady :) "); // read line from stdin

    buf.trim().to_string() // trim and rt string
}



///////////////////////////////////////////////
// END OF input.rs
///////////////////////////////////////////////


// Thanks for reading this code! :)
// Made by imu
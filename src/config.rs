// src/config.rs

/// config module for handling user configuration



use rpassword::prompt_password; // importing rpassword

/// asks the user for their API key hidden input for security. this function rt string.
pub fn ask_api_key() -> String {

    // prompt(input) the user for their API key
    prompt_password("Enter API key: ").expect("Failed to read API key")

}



///////////////////////////////////////////////
// End of config.rs
///////////////////////////////////////////////


// rpassword crate documentation:
// is a crate for hidding password input, usefull for securty.
// https://docs.rs/rpassword/latest/rpassword/



// Thanks for reading this code! :)
// Made by imu
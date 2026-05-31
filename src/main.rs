// src/main.rs



// :}
/////////////////////////////////////////////////////////////
// DragonFox IDE
// a simple AI assistant
// under development
// goal is to make a fully functional IDE with AI integration
/////////////////////////////////////////////////////////////

// enjoy :)


// modules importing
mod ai;
mod config;
mod input;



#[tokio::main] // tokio is an async runtime for rust

// main function
async fn main() {
    println!("DragonFox IDE :}}\n");
    println!("Type ur question below");
    println!("This is a simple AI assistant.");
    println!("easy to use");
    println!("Todo: Fully functional IDE with AI integration");

    let api_key = config::ask_api_key(); // ask for api key function from config module
    
    if api_key.is_empty() { // check if api key is empty

        eprintln!("API key cannot be empty."); // eprintln error message printing function

        return;
    }

    println!("\nReady! Type your questions below.\n");

    // loop for continuous input
    loop {
        // print prompt
        print!("You > ");
        let prompt = input::input();

        if prompt.is_empty() {
            continue; // skip empty input
        }

        if let Err(e) = ai::call_ai(&prompt, &api_key).await { // call ai function from ai module

            eprintln!("Error: {}", e); // eprintln error message printing function
        
        }

        println!();
    }
}


// :}

///////////////////////////////////////////////////////////
// end of main.rs
///////////////////////////////////////////////////////////


// Thanks for reading this code! :)
// Made by imu
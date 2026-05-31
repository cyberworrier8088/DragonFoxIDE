// src/ai.rs


// importing reqwest and serde_json
use reqwest::Client; // importing reqwest
use serde_json::{json, Value}; // importing serde_json



// sends a prompt to the AI and prints the response.
pub async fn call_ai(prompt: &str, api_key: &str) -> Result<(), Box<dyn std::error::Error>> {

    let body = json!({ // creating a json object
        "model": "nvidia/nemotron-3-nano-omni-30b-a3b-reasoning:free",
        "messages": [
            { "role": "system", "content": "You are an expert in programming and can help with any questions." }, // system prompt. roll to give context to the AI, instrction.
            { "role": "system", "content": "You are a expert teacher" }, // system prompt
            { "role": "user", "content": prompt.trim() } // user prompt
        ]
    });

    let resp: Value = Client::new() // creating a new client
        .post("https://ai.hackclub.com/proxy/v1/chat/completions") // posting to the API.
        .bearer_auth(api_key) // setting the API key
        .json(&body) // setting the body
        .send()
        .await?
        .json()
        .await?;

    let answer = resp["choices"][0]["message"]["content"] // getting the answer from the response
        .as_str()
        .unwrap_or("No response");

    println!("DragonFoxAI > {}", answer); // printing the answer
    
    Ok(()) // returning Ok
}



///////////////////////////////////////////////
// END OF ai.rs
///////////////////////////////////////////////


// reqwest crate documentation:
// is a crate for making HTTP requests, usefull for API calls.
// https://docs.rs/reqwest/latest/reqwest/

// serde_json crate documentation:
// is a crate for parsing JSON, usefull for API calls.
// https://docs.rs/serde_json/latest/serde_json/



// Thanks for reading this code! :)
// Made by imu

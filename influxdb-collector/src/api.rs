use reqwest::blocking::Client;
use serde_json::Value;

pub fn fetch_sensor_data(api_url: &str) -> Result<Value, String> {
    let client = Client::new();
    
    // Send the request
    let response = client.get(api_url)
        .send()
        .map_err(|e| format!("Failed to fetch from {}: {}", api_url, e))?;
    
    // Check if the response was successful
    if !response.status().is_success() {
        return Err(format!("HTTP error {} from {}", response.status(), api_url));
    }
    
    // Get the response text first to debug what we're receiving
    let response_text = response.text()
        .map_err(|e| format!("Failed to read response body from {}: {}", api_url, e))?;
    
    // Check if the response is empty
    if response_text.trim().is_empty() {
        return Err(format!("Empty response from {}", api_url));
    }
    
    // Try to parse as JSON and provide better error context
    serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse JSON from {}: {}. Response was: '{}'", 
                            api_url, e, response_text.chars().take(200).collect::<String>()))
}
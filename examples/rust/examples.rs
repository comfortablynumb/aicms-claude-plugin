// AICMS Rust Examples
// Demonstrates @ai:* annotations in Rust code

/// @ai:intent Calculate compound interest for a loan or investment
/// @ai:pre principal > 0.0, rate >= 0.0, years >= 0
/// @ai:post result >= principal
/// @ai:example (1000.0, 0.05, 10) -> 1628.89
/// @ai:example (1000.0, 0.0, 5) -> 1000.0
/// @ai:example (500.0, 0.1, 1) -> 550.0
/// @ai:effects pure
/// @ai:complexity O(1)
/// @ai:confidence 0.95
pub fn calculate_compound_interest(principal: f64, rate: f64, years: u32) -> f64 {
    principal * (1.0 + rate).powi(years as i32)
}

/// @ai:intent Find the maximum value in a non-empty slice
/// @ai:pre values.len() > 0
/// @ai:post result is contained in values
/// @ai:post result >= all elements in values
/// @ai:example ([1, 5, 3, 9, 2]) -> 9
/// @ai:example ([42]) -> 42
/// @ai:example ([-5, -2, -8]) -> -2
/// @ai:effects pure
/// @ai:complexity O(n)
/// @ai:edge_cases single element -> returns that element
/// @ai:edge_cases all equal -> returns that value
pub fn find_max(values: &[i32]) -> i32 {
    *values.iter().max().unwrap()
}

/// @ai:intent Send an HTTP POST request with JSON body and return the response
/// @ai:pre url is a valid HTTP/HTTPS URL
/// @ai:pre body is valid JSON
/// @ai:post result contains response status and body
/// @ai:effects network, io
/// @ai:idempotent false
/// @ai:needs_review Verify timeout handling and error cases
/// @ai:assumes Network connectivity is available
/// @ai:confidence 0.75
pub async fn post_json(
    client: &reqwest::Client,
    url: &str,
    body: serde_json::Value,
) -> Result<Response, reqwest::Error> {
    let response = client
        .post(url)
        .json(&body)
        .send()
        .await?;

    Ok(Response {
        status: response.status().as_u16(),
        body: response.text().await?,
    })
}

pub struct Response {
    pub status: u16,
    pub body: String,
}

/// @ai:intent Validate an email address format using basic rules
/// @ai:pre email is not empty
/// @ai:post result == true implies email contains exactly one '@' with text on both sides
/// @ai:example ("user@example.com") -> true
/// @ai:example ("invalid-email") -> false
/// @ai:example ("@example.com") -> false
/// @ai:example ("user@") -> false
/// @ai:example ("") -> false
/// @ai:effects pure
/// @ai:complexity O(n)
/// @ai:confidence 0.85
/// @ai:needs_review Does not validate against RFC 5322, only basic format
pub fn is_valid_email(email: &str) -> bool {
    if email.is_empty() {
        return false;
    }

    let parts: Vec<&str> = email.split('@').collect();

    if parts.len() != 2 {
        return false;
    }

    let local = parts[0];
    let domain = parts[1];

    !local.is_empty() && !domain.is_empty() && domain.contains('.')
}

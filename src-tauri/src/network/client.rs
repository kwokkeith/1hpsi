use ureq;

/// Performs HTTP get request to specified path return response body as string
pub fn get(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let body = ureq::get(url).call()?.body_mut().read_to_string()?;

    Ok(body)
}

use rig::providers::openai::Client;

/// Build an agent for autonomous cargo-declared workflows
pub fn build_agent(api_url: &str, model: &str) -> Client {
    // LM Studio and other OpenAI-compatible endpoints use the same API key format
    // The actual key value is unused for LM Studio but required by the API
    Client::new("unused-api-key")
        .unwrap()
        .with_api_url(api_url)
        .with_model(model)
}

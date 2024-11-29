pub struct Config {}

impl Config {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_anthropic_key(&self) -> Result<String, Box<dyn std::error::Error>> {
        std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| "ANTHROPIC_API_KEY not set".into())
    }
}
use super::OpenAPISpec;
use anthropic::client::Client;
use anthropic::client::ClientBuilder;
use anthropic::types::{ContentBlock, Message, MessagesRequestBuilder, Role};
use std::fs;
use std::path::Path;

pub struct DocsGenerator {
    client: Client,
}

impl DocsGenerator {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: ClientBuilder::default()
                .api_key(api_key.to_string())
                .build()
                .unwrap(),
        }
    }

    pub async fn generate(&self, spec: &OpenAPISpec, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // Create Next.js project structure
        println!("Creating project structure...");
        fs::create_dir_all(output_dir.join("pages"))?;
        fs::create_dir_all(output_dir.join("components"))?;
        fs::create_dir_all(output_dir.join("styles"))?;

        println!("Generating main page...");
        // Generate main documentation content
        self.generate_main_page(spec, output_dir).await?;
        
        println!("Generating endpoints docs...");
        self.generate_endpoints_docs(spec, output_dir).await?;
        
        println!("Generating package.json...");
        self.generate_package_json(output_dir)?;
        
        println!("Generating components...");
        self.generate_components(output_dir)?;

        println!("📚 Documentation site generated at {}", output_dir.display());
        println!("To start the development server:");
        println!("1. cd {}", output_dir.display());
        println!("2. npm install");
        println!("3. npm run dev");

        Ok(())
    }

    async fn generate_main_page(&self, spec: &OpenAPISpec, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let prompt = format!(
            "Create a Next.js index.tsx page that serves as the main documentation page for this API:\n\
            Title: {}\n\
            Version: {}\n\
            Base URL: {}\n\
            Include sections for:\n\
            1. Introduction\n\
            2. Authentication (if applicable)\n\
            3. Getting Started\n\
            4. Quick Examples\n\
            Use modern React and Tailwind CSS. Make it professional and developer-friendly.\n\
            Return only the TypeScript/TSX code without any markdown or explanation.",
            spec.info.title,
            spec.info.version,
            spec.servers.first().map_or("", |s| &s.url)
        );

        // Generate content using Claude
        let response = self.get_ai_response(&prompt).await?;
        
        // Extract just the code from the response
        let code = if response.contains("```") {
            response
                .split("```")
                .nth(1)  // Get the content between first pair of ```
                .unwrap_or(&response)
                .trim()
                .trim_start_matches("tsx")
                .trim()
        } else {
            &response
        };

        fs::write(output_dir.join("pages/index.tsx"), code)?;
        Ok(())
    }

    async fn generate_endpoints_docs(&self, spec: &OpenAPISpec, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // Create endpoints directory first
        let endpoints_dir = output_dir.join("pages").join("endpoints");
        fs::create_dir_all(&endpoints_dir)?;
        println!("Created endpoints directory at: {:?}", endpoints_dir);
    
        for (path, item) in &spec.paths {
            let endpoint_name = path.trim_matches('/').replace('/', "_");
            println!("Generating docs for endpoint: {}", endpoint_name);
            
            let prompt = format!(
                "Create a Next.js page component for this API endpoint:\n\
                Path: {}\n\
                Methods: {:?}\n\
                Create detailed documentation including:\n\
                1. Endpoint description\n\
                2. Request/Response examples\n\
                3. Parameters explanation\n\
                4. Interactive try-it-out section\n\
                Use modern React and Tailwind CSS. Include syntax highlighting for code examples.\n\
                Return only the TypeScript/TSX code without any markdown or explanation.",
                path,
                item
            );
    
            let response = self.get_ai_response(&prompt).await?;
            // Extract just the code if the response contains markdown code blocks
            let code = if response.contains("```") {
                response
                    .split("```")
                    .nth(1)
                    .unwrap_or(&response)
                    .trim()
                    .trim_start_matches("tsx")
                    .trim()
            } else {
                &response
            };
    
            fs::write(
                endpoints_dir.join(format!("{}.tsx", endpoint_name)),
                code
            )?;
            println!("Generated docs for endpoint: {}", endpoint_name);
        }
    
        Ok(())
    }

    fn generate_package_json(&self, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let package_json = r#"{
  "name": "api-docs",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "dev": "next dev",
    "build": "next build",
    "start": "next start"
  },
  "dependencies": {
    "next": "latest",
    "react": "latest",
    "react-dom": "latest",
    "tailwindcss": "latest",
    "prismjs": "latest",
    "@headlessui/react": "latest"
  },
  "devDependencies": {
    "@types/node": "latest",
    "@types/react": "latest",
    "typescript": "latest",
    "autoprefixer": "latest",
    "postcss": "latest"
  }
}"#;
        fs::write(output_dir.join("package.json"), package_json)?;
        Ok(())
    }

    async fn get_ai_response(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let messages = vec![Message {
            role: Role::User,
            content: vec![ContentBlock::Text { text: prompt.to_string() }],
        }];

        let request = MessagesRequestBuilder::default()
            .messages(messages)
            .model("claude-3-sonnet-20240229".to_string())
            .max_tokens(4000_usize)
            .build()?;

        let response = self.client.messages(request).await?;
        
        if let ContentBlock::Text { text } = &response.content[0] {
            Ok(text.to_string())
        } else {
            Err("Unexpected response format".into())
        }
    }

    fn generate_components(&self, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // Create basic shared components
        let components_dir = output_dir.join("components");
        fs::create_dir_all(&components_dir)?;
        
        // Add a basic layout component
        let layout = r#"export default function Layout({ children }) {
    return (
        <div className="min-h-screen bg-gray-50">
            <nav className="bg-white shadow-sm">
                {/* Add navigation content */}
            </nav>
            <main className="container mx-auto px-4 py-8">
                {children}
            </main>
        </div>
    );
}"#;
        fs::write(components_dir.join("Layout.tsx"), layout)?;
        Ok(())
    }
}

//! Domain Forge - AI-powered domain name generation and availability checking
//!
//! A simple and elegant CLI tool for generating creative domain names using AI
//! and checking their availability in real-time.

use domain_forge::{
    domain::DomainChecker,
    llm::DomainGenerator,
    types::{GenerationConfig, LlmConfig, DomainSuggestion, AvailabilityStatus},
    Result,
};
use inquire::MultiSelect;
use rand::Rng;
use std::env;
use std::process;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the library
    if let Err(e) = domain_forge::init() {
        eprintln!("❌ Failed to initialize: {}", e);
        process::exit(1);
    }

    // Get command line arguments
    let args: Vec<String> = env::args().collect();

    // Check for help
    if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
        print_help();
        return Ok(());
    }

    // Determine if user provided a description
    let description = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        String::new()
    };

    // Run the main flow
    if let Err(e) = run_domain_forge(&description).await {
        eprintln!("❌ Error: {}", e);
        process::exit(1);
    }

    Ok(())
}

/// Main domain forge workflow
async fn run_domain_forge(description: &str) -> Result<()> {
    // Show welcome message
    println!("🔥 Domain Forge - AI-powered domain name generation");
    println!("═══════════════════════════════════════════════════");
    println!();

    // Set up LLM generator
    let mut generator = DomainGenerator::new();
    setup_llm_providers(&mut generator)?;

    // Generate domains
    let domains = if description.is_empty() {
        generate_random_domains(&generator).await?
    } else {
        generate_domains_for_description(&generator, description).await?
    };

    if domains.is_empty() {
        println!("❌ No domains were generated. Please check your API configuration.");
        return Ok(());
    }

    // Let user select domains to check
    let selected_domains = select_domains_to_check(&domains)?;

    if selected_domains.is_empty() {
        println!("👋 No domains selected. Goodbye!");
        return Ok(());
    }

    // Check domain availability
    check_domain_availability(&selected_domains).await?;

    Ok(())
}

/// Setup LLM providers from environment variables
fn setup_llm_providers(generator: &mut DomainGenerator) -> Result<()> {
    // Try to add OpenAI provider
    if let Ok(api_key) = env::var("OPENAI_API_KEY") {
        let base_url = env::var("OPENAI_BASE_URL").ok();
        let model = env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4.1-mini".to_string());

        // Debug information
        println!("🔧 Debug: API Key length: {}", api_key.len());
        if let Some(ref url) = base_url {
            println!("🔧 Debug: Base URL: {}", url);
        }
        println!("🔧 Debug: Model: {}", model);

        let config = LlmConfig {
            provider: "openai".to_string(),
            model,
            api_key,
            base_url,
            temperature: 0.7,
        };
        generator.add_provider(&config)?;
        generator.set_default_provider("openai");
        println!("✅ OpenAI provider configured");
    }

    // Try to add Anthropic provider
    if let Ok(api_key) = env::var("ANTHROPIC_API_KEY") {
        let config = LlmConfig {
            provider: "anthropic".to_string(),
            model: env::var("ANTHROPIC_MODEL").unwrap_or_else(|_| "claude-4-sonnet".to_string()),
            api_key,
            base_url: None,
            temperature: 0.7,
        };
        generator.add_provider(&config)?;
        if !generator.has_provider("openai") {
            generator.set_default_provider("anthropic");
        }
        println!("✅ Anthropic provider configured");
    }

    // Try to add Gemini provider
    if let Ok(api_key) = env::var("GEMINI_API_KEY") {
        let config = LlmConfig {
            provider: "gemini".to_string(),
            model: env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-2.5-flash".to_string()),
            api_key,
            base_url: None,
            temperature: 0.7,
        };
        generator.add_provider(&config)?;
        if !generator.has_provider("openai") && !generator.has_provider("anthropic") {
            generator.set_default_provider("gemini");
        }
        println!("✅ Gemini provider configured");
    }

    if !generator.is_ready() {
        return Err(domain_forge::DomainForgeError::config(
            "No LLM providers configured. Please set OPENAI_API_KEY, ANTHROPIC_API_KEY, or GEMINI_API_KEY environment variable.".to_string()
        ));
    }

    Ok(())
}

/// Generate random domains when no description is provided
async fn generate_random_domains(generator: &DomainGenerator) -> Result<Vec<DomainSuggestion>> {
    let random_prompts = vec![
        "innovative tech startup",
        "creative digital agency",
        "modern e-commerce platform",
        "AI-powered productivity tool",
        "sustainable lifestyle brand",
        "cutting-edge software solution",
        "next-generation mobile app",
        "revolutionary fintech service",
    ];

    let mut rng = rand::thread_rng();
    let prompt = random_prompts[rng.gen_range(0..random_prompts.len())];
    println!("🎲 Generating random domains for: \"{}\"", prompt);

    let config = GenerationConfig {
        description: prompt.to_string(),
        count: 8,
        style: domain_forge::types::GenerationStyle::Creative,
        tlds: vec!["com".to_string(), "io".to_string(), "ai".to_string(), "app".to_string()],
        temperature: 0.8,
        ..Default::default()
    };

    println!("🤖 Generating domains with AI...");
    generator.generate_with_fallback(&config).await
}

/// Generate domains based on user description
async fn generate_domains_for_description(generator: &DomainGenerator, description: &str) -> Result<Vec<DomainSuggestion>> {
    println!("🎯 Generating domains for: \"{}\"", description);

    let config = GenerationConfig {
        description: description.to_string(),
        count: 8,
        style: domain_forge::types::GenerationStyle::Creative,
        tlds: vec!["com".to_string(), "org".to_string(), "io".to_string(), "ai".to_string()],
        temperature: 0.7,
        ..Default::default()
    };

    println!("🤖 Generating domains with AI...");
    generator.generate_with_fallback(&config).await
}

/// Let user select domains to check using beautiful multi-select interface
fn select_domains_to_check(domains: &[DomainSuggestion]) -> Result<Vec<DomainSuggestion>> {
    println!();
    println!("🎨 Generated Domains:");
    println!("═══════════════════");

    // Display domains with details
    for (i, domain) in domains.iter().enumerate() {
        println!("{}. {} (confidence: {:.0}%)",
            i + 1,
            domain.full_domain,
            domain.confidence * 100.0
        );
        if let Some(reasoning) = &domain.reasoning {
            println!("   💭 {}", reasoning);
        }
        println!();
    }

    // Create options for multi-select
    let options: Vec<String> = domains.iter().map(|d| {
        format!("{} ({}%)", d.full_domain, (d.confidence * 100.0) as u8)
    }).collect();

    // Add special options
    let mut all_options = options.clone();
    all_options.insert(0, "🔄 Generate new domains".to_string());
    all_options.push("✅ Select all domains".to_string());

    let selected = MultiSelect::new("Select domains to check availability:", all_options)
        .with_help_message("Use ↑↓ to navigate, Space to select, Enter to confirm")
        .prompt()
        .map_err(|e| domain_forge::DomainForgeError::cli(format!("Selection cancelled: {}", e)))?;

    // Handle special selections
    if selected.contains(&"🔄 Generate new domains".to_string()) {
        return Err(domain_forge::DomainForgeError::cli("Regeneration requested".to_string()));
    }

    if selected.contains(&"✅ Select all domains".to_string()) {
        return Ok(domains.to_vec());
    }

    // Map selected options back to domains
    let mut selected_domains = Vec::new();
    for selection in selected {
        if let Some(index) = options.iter().position(|opt| opt == &selection) {
            selected_domains.push(domains[index].clone());
        }
    }

    Ok(selected_domains)
}

/// Check domain availability and display results beautifully
async fn check_domain_availability(domains: &[DomainSuggestion]) -> Result<()> {
    println!();
    println!("🔍 Checking domain availability...");
    println!("═══════════════════════════════════");

    let checker = DomainChecker::new();
    let domain_names: Vec<String> = domains.iter().map(|d| d.get_full_domain()).collect();

    let check_start = std::time::Instant::now();
    let results = checker.check_domains(&domain_names).await?;
    let check_duration = check_start.elapsed();

    println!();
    println!("📊 Results:");
    println!("═══════════");

    let mut available_count = 0;
    let mut taken_count = 0;

    for (domain, result) in domains.iter().zip(results.iter()) {
        match result.status {
            AvailabilityStatus::Available => {
                println!("✅ {} - AVAILABLE", domain.get_full_domain());
                available_count += 1;
            }
            AvailabilityStatus::Taken => {
                println!("❌ {} - TAKEN", domain.get_full_domain());
                if let Some(registrar) = &result.registrar {
                    println!("   📝 Registrar: {}", registrar);
                }
                taken_count += 1;
            }
            AvailabilityStatus::Unknown => {
                println!("❓ {} - UNKNOWN", domain.get_full_domain());
            }
            AvailabilityStatus::Error => {
                println!("⚠️  {} - ERROR", domain.get_full_domain());
                if let Some(error) = &result.error_message {
                    println!("   🔍 {}", error);
                }
            }
        }

        if let Some(reasoning) = &domain.reasoning {
            println!("   💭 {}", reasoning);
        }
        println!();
    }

    // Performance summary
    let metrics = checker.get_metrics_snapshot();
    
    println!("📈 Summary:");
    println!("   Available: {}", available_count);
    println!("   Taken: {}", taken_count);
    println!("   Total checked: {}", domains.len());
    println!("   ⏱️  Total time: {:.2}s", check_duration.as_secs_f32());
    if metrics.domains_checked > 0 {
        println!("   📊 Average check time: {:.1}ms", metrics.avg_check_time_ms());
    }

    if available_count > 0 {
        println!();
        println!("🎉 Great! You have {} available domain(s) to choose from!", available_count);
    }

    Ok(())
}

/// Print help information
fn print_help() {
    println!("🔥 Domain Forge - AI-powered domain name generation");
    println!("═══════════════════════════════════════════════════");
    println!();
    println!("USAGE:");
    println!("    domain-forge [DESCRIPTION]");
    println!();
    println!("EXAMPLES:");
    println!("    domain-forge                           # Generate random domains");
    println!("    domain-forge \"AI productivity app\"     # Generate for description");
    println!("    domain-forge \"fintech startup\"        # Generate for startup idea");
    println!();
    println!("ENVIRONMENT VARIABLES:");
    println!("    OPENAI_API_KEY     OpenAI API key");
    println!("    ANTHROPIC_API_KEY  Anthropic API key");
    println!("    GEMINI_API_KEY     Google Gemini API key");
    println!();
    println!("    OPENAI_MODEL       OpenAI model (default: gpt-4.1-mini)");
    println!("    ANTHROPIC_MODEL    Anthropic model (default: claude-4-sonnet)");
    println!("    GEMINI_MODEL       Gemini model (default: gemini-2.5-flash)");
    println!();
    println!("FEATURES:");
    println!("    • AI-powered domain generation using OpenAI, Anthropic, Gemini, or Ollama");
    println!("    • Beautiful interactive multi-select interface");
    println!("    • Real-time domain availability checking");
    println!("    • Support for multiple TLDs (.com, .org, .io, .ai)");
    println!();
    println!("Made with ❤️ and 🦀 Rust");
}


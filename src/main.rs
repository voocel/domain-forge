//! Domain Forge - AI-powered domain name generation and availability checking
//!
//! A simple and elegant CLI tool for generating creative domain names using AI
//! and checking their availability in real-time.

use domain_forge::{
    domain::DomainChecker,
    llm::DomainGenerator,
    types::{GenerationConfig, LlmConfig, DomainSuggestion, AvailabilityStatus, DomainSession, DomainResult},
    Result,
};
use indicatif::{ProgressBar, ProgressStyle};
use inquire::Select;
use rand::Rng;
use std::env;
use std::process;
use std::io;
use std::time::Duration;

#[derive(Debug, Clone)]
enum MenuOption {
    GenerateMore,
    ShowAvailable,
    SaveToFile,
    Quit,
}

impl std::fmt::Display for MenuOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MenuOption::GenerateMore => write!(f, "🔄 Generate more domains"),
            MenuOption::ShowAvailable => write!(f, "📋 Show available domains only"),
            MenuOption::SaveToFile => write!(f, "💾 Download results to file"),
            MenuOption::Quit => write!(f, "🚪 Quit"),
        }
    }
}

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

    // Initialize session state
    let mut session = DomainSession::new();
    let final_description = if description.is_empty() {
        get_random_description()
    } else {
        description.to_string()
    };

    // Main generation loop
    loop {
        // Generate domains for this round
        let round_start = std::time::Instant::now();
        let domains = generate_domains_for_round(&generator, &final_description, &session).await?;
        
        if domains.is_empty() {
            println!("❌ No domains were generated. Please check your API configuration.");
            break;
        }

        // Check domain availability with beautiful progress
        let checker = DomainChecker::new();
        let domain_names: Vec<String> = domains.iter().map(|d| d.get_full_domain()).collect();

        let check_pb = ProgressBar::new_spinner();
        check_pb.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["🔍", "🔎", "🕵️", "🔍", "🔎", "🕵️"])
                .template("{spinner:.green} {msg}")
                .unwrap()
        );
        check_pb.enable_steady_tick(Duration::from_millis(100));
        check_pb.set_message(format!("🔍 Checking {} domains for availability...", domain_names.len()));

        let results = checker.check_domains(&domain_names).await?;
        check_pb.finish_with_message("✅ Domain availability check complete!");
        let round_time = round_start.elapsed();

        // Update session with results
        session.add_round_results(&domains, &results, round_time);

        // Display beautiful results
        render_results_panel(&session, &domains, &results, round_time);

        // Show menu and get user choice
        match show_menu_and_get_choice()? {
            MenuOption::GenerateMore => {
                // Generate more domains - continue to next round
                continue;
            }
            MenuOption::ShowAvailable => {
                // Show available domains only
                show_available_domains_only(&session);
                // Show menu again after displaying available domains
                match show_menu_and_get_choice()? {
                    MenuOption::GenerateMore => continue,
                    MenuOption::SaveToFile => {
                        if let Err(e) = save_results_to_file(&session, &final_description) {
                            eprintln!("❌ Failed to save file: {}", e);
                        }
                        break;
                    }
                    _ => break,
                }
            }
            MenuOption::SaveToFile => {
                // Download results to file
                if let Err(e) = save_results_to_file(&session, &final_description) {
                    eprintln!("❌ Failed to save file: {}", e);
                }
                break;
            }
            MenuOption::Quit => {
                // Quit
                break;
            }
        }
    }

    // Final summary
    if !session.available_domains.is_empty() {
        println!();
        println!("🎉 Session Complete! Found {} available domains in {} rounds.", 
            session.available_domains.len(), session.round_count);
    } else {
        println!();
        println!("👋 Session ended. No available domains found.");
    }

    Ok(())
}

/// Get a random description for when no user input is provided
fn get_random_description() -> String {
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
    prompt.to_string()
}

/// Create a beautiful progress bar for AI generation
fn create_ai_progress_bar() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["🤖", "🧠", "💭", "✨", "🎯", "🔮", "⚡", "🚀"])
            .template("{spinner:.blue} {msg}")
            .unwrap()
    );
    pb.enable_steady_tick(Duration::from_millis(120));
    pb
}



/// Generate domains for a single round, considering previous session state
async fn generate_domains_for_round(generator: &DomainGenerator, description: &str, session: &DomainSession) -> Result<Vec<DomainSuggestion>> {
    // Let LLM handle everything - it's smart enough to understand user intent
    let tlds = vec!["com".to_string(), "org".to_string(), "io".to_string(), "ai".to_string(), "tech".to_string(), "dev".to_string(), "app".to_string()];

    let config = GenerationConfig {
        description: description.to_string(),
        count: 20,
        style: domain_forge::types::GenerationStyle::Creative,
        tlds,
        temperature: 0.7,
        avoid_names: session.get_taken_domain_names(), // Smart avoidance!
        ..Default::default()
    };

    // Show beautiful progress for AI generation
    let pb = create_ai_progress_bar();
    if session.round_count == 0 {
        pb.set_message("🎨 AI is crafting creative domain names...");
    } else {
        pb.set_message(format!("🎨 Generating {} more domains (avoiding {} taken ones)...",
            config.count, session.taken_domains.len()));
    }

    let result = generator.generate_with_fallback(&config).await;
    pb.finish_with_message("✅ Domain generation complete!");

    result
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

// ===== Beautiful Terminal UI Functions =====

/// Render a beautiful results panel for the current round
fn render_results_panel(session: &DomainSession, round_domains: &[DomainSuggestion], round_results: &[DomainResult], round_time: std::time::Duration) {
    let round_available: Vec<&DomainSuggestion> = round_domains.iter().zip(round_results.iter())
        .filter(|(_, result)| result.status == AvailabilityStatus::Available)
        .map(|(domain, _)| domain)
        .collect();
    
    let round_taken: Vec<&DomainSuggestion> = round_domains.iter().zip(round_results.iter())
        .filter(|(_, result)| result.status == AvailabilityStatus::Taken)
        .map(|(domain, _)| domain)
        .collect();

    println!();
    println!("╭─ Round {} Results ─────────────────────────────────────╮", session.round_count);
    println!("│                                                       │");
    
    if session.round_count > 1 {
        println!("│  🎯 Generated {} new domains (avoided {} taken ones)   │", 
            round_domains.len(), session.taken_domains.len() - round_taken.len());
        println!("│                                                       │");
    }
    
    // Show available domains for this round
    println!("│  🎉 Available Domains This Round ({:<2})                 │", round_available.len());
    println!("│  ┌─────────────────────────────────────────────────┐  │");
    
    if round_available.is_empty() {
        println!("│  │  (none found this round)                    │  │");
    } else {
        for chunk in round_available.chunks(3) {
            print!("│  │  ");
            for domain in chunk {
                print!("✅ {:<12}", domain.get_full_domain());
            }
            // Fill remaining space
            for _ in chunk.len()..3 {
                print!("             ");
            }
            println!(" │  │");
        }
    }
    
    println!("│  └─────────────────────────────────────────────────┘  │");
    println!("│                                                       │");
    
    // Show taken domains for this round (very important!)
    if !round_taken.is_empty() {
        println!("│  ⚪ Taken Domains This Round ({:<2})                   │", round_taken.len());
        println!("│  ┌─────────────────────────────────────────────────┐  │");
        for chunk in round_taken.chunks(3) {
            print!("│  │  ");
            for domain in chunk {
                print!("⚪ {:<12}", domain.get_full_domain());
            }
            for _ in chunk.len()..3 {
                print!("             ");
            }
            println!(" │  │");
        }
        println!("│  └─────────────────────────────────────────────────┘  │");
        println!("│                                                       │");
    }
    
    // Show total available if multi-round
    if session.round_count > 1 && !session.available_domains.is_empty() {
        println!("│  🏆 Total Available Domains ({:<2})                    │", session.available_domains.len());
        println!("│  ┌─────────────────────────────────────────────────┐  │");
        for chunk in session.available_domains.chunks(3) {
            print!("│  │  ");
            for domain in chunk {
                print!("✅ {:<12}", domain.get_full_domain());
            }
            for _ in chunk.len()..3 {
                print!("             ");
            }
            println!(" │  │");
        }
        println!("│  └─────────────────────────────────────────────────┘  │");
        println!("│                                                       │");
    }
    
    // Stats
    if session.round_count == 1 {
        println!("│  📊 Stats: {} available • {} taken • {:.1}s           │", 
            round_available.len(), 
            round_taken.len(),
            round_time.as_secs_f32());
    } else {
        println!("│  📊 Total: {} available • {} taken • {:.1}s total      │", 
            session.available_domains.len(),
            session.taken_domains.len(),
            session.total_time.as_secs_f32());
    }
    
    println!("╰───────────────────────────────────────────────────────╯");
}

/// Show interactive menu and get user choice
fn show_menu_and_get_choice() -> Result<MenuOption> {
    println!();
    
    let options = vec![
        MenuOption::GenerateMore,
        MenuOption::ShowAvailable,
        MenuOption::SaveToFile,
        MenuOption::Quit,
    ];
    
    let selection = Select::new("What would you like to do next?", options)
        .with_help_message("Use ↑↓ arrow keys to navigate, Enter to select")
        .prompt()
        .map_err(|e| domain_forge::DomainForgeError::cli(format!("Menu selection cancelled: {}", e)))?;
    
    Ok(selection)
}

/// Show only available domains in a clean format
fn show_available_domains_only(session: &DomainSession) {
    println!();
    println!("╭─ Available Domains Summary ───────────────────────────╮");
    println!("│                                                       │");
    
    if session.available_domains.is_empty() {
        println!("│  😔 No available domains found yet.                  │");
        println!("│      Try generating more domains!                    │");
    } else {
        println!("│  🎉 Found {} Available Domains:                      │", session.available_domains.len());
        println!("│  ┌─────────────────────────────────────────────────┐  │");
        
        for chunk in session.available_domains.chunks(3) {
            print!("│  │  ");
            for domain in chunk {
                print!("✅ {:<12}", domain.get_full_domain());
            }
            for _ in chunk.len()..3 {
                print!("             ");
            }
            println!(" │  │");
        }
        
        println!("│  └─────────────────────────────────────────────────┘  │");
    }
    
    println!("│                                                       │");
    println!("│  📊 {} rounds • {} total checked • {:.1}s total        │",
        session.round_count,
        session.total_domains_checked(),
        session.total_time.as_secs_f32());
    println!("╰───────────────────────────────────────────────────────╯");
}

/// Save results to a file
fn save_results_to_file(session: &DomainSession, description: &str) -> io::Result<()> {
    use std::fs;
    
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("domains_{}.txt", timestamp);
    
    let mut content = String::new();
    content.push_str(&format!("Domain Forge Results\n"));
    content.push_str(&format!("Generated: {}\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    content.push_str(&format!("Search: {}\n", description));
    content.push_str(&format!("Rounds: {}\n", session.round_count));
    content.push_str(&format!("Total Time: {:.1}s\n", session.total_time.as_secs_f32()));
    content.push_str(&format!("Total Checked: {}\n\n", session.total_domains_checked()));
    
    content.push_str(&format!("=== AVAILABLE DOMAINS ({}) ===\n", session.available_domains.len()));
    if session.available_domains.is_empty() {
        content.push_str("None found.\n");
    } else {
        for domain in &session.available_domains {
            content.push_str(&format!("{}\n", domain.get_full_domain()));
        }
    }
    
    content.push_str(&format!("\n=== TAKEN DOMAINS ({}) ===\n", session.taken_domains.len()));
    for domain in &session.taken_domains {
        content.push_str(&format!("{}\n", domain));
    }
    
    if !session.error_domains.is_empty() {
        content.push_str(&format!("\n=== ERRORS ({}) ===\n", session.error_domains.len()));
        for (domain, error) in &session.error_domains {
            content.push_str(&format!("{}: {}\n", domain, error));
        }
    }
    
    fs::write(&filename, content)?;
    
    println!();
    println!("╭─ File Saved ──────────────────────────────────────────╮");
    println!("│                                                       │");
    println!("│  💾 Results saved to: {:<28}│", filename);
    println!("│                                                       │");
    println!("│  📊 {} available domains                              │", session.available_domains.len());
    println!("│  📊 {} taken domains                                  │", session.taken_domains.len());
    println!("│                                                       │");
    println!("╰───────────────────────────────────────────────────────╯");
    
    Ok(())
}

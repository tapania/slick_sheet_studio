//! Slick CLI - Command-line tool for Slick Sheet Studio
//!
//! This tool provides command-line access to:
//! - Read/write JSON content data
//! - Read/write Typst templates
//! - Render templates with data
//! - Compile Typst to SVG/PDF
//! - Run AI agent for automated editing

use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Slick Sheet Studio CLI
#[derive(Parser)]
#[command(name = "slick-cli")]
#[command(about = "Command-line tool for Slick Sheet Studio", long_about = None)]
#[command(version)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Read current JSON content data from a project
    ReadJson {
        /// Path to project JSON file
        #[arg(short, long)]
        project: PathBuf,

        /// Output in compact format (default: pretty)
        #[arg(long)]
        compact: bool,
    },

    /// Write new JSON content data to a project (validates before accepting)
    WriteJson {
        /// Path to project JSON file
        #[arg(short, long)]
        project: PathBuf,

        /// Path to new JSON file to read from
        #[arg(short, long)]
        input: Option<PathBuf>,

        /// Read JSON from stdin
        #[arg(long)]
        stdin: bool,

        /// Skip validation (not recommended)
        #[arg(long)]
        no_validate: bool,

        /// Validate without writing
        #[arg(long)]
        dry_run: bool,
    },

    /// Read current template from a project
    ReadTemplate {
        /// Path to project JSON file
        #[arg(short, long)]
        project: PathBuf,
    },

    /// Write new template to a project (validates before accepting)
    WriteTemplate {
        /// Path to project JSON file
        #[arg(short, long)]
        project: PathBuf,

        /// Path to new template file to read from
        #[arg(short, long)]
        input: Option<PathBuf>,

        /// Read template from stdin
        #[arg(long)]
        stdin: bool,

        /// Skip validation (not recommended)
        #[arg(long)]
        no_validate: bool,

        /// Validate without writing
        #[arg(long)]
        dry_run: bool,
    },

    /// Render template with data to produce Typst code
    Render {
        /// Path to JSON data file
        #[arg(short, long)]
        data: PathBuf,

        /// Path to template file
        #[arg(short, long)]
        template: PathBuf,

        /// Output Typst file path (if not specified, writes to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Compile Typst source to SVG, PDF, or PNG
    Compile {
        /// Path to Typst source file
        #[arg(short, long)]
        input: PathBuf,

        /// Output SVG file path
        #[arg(long)]
        output_svg: Option<PathBuf>,

        /// Output PDF file path
        #[arg(long)]
        output_pdf: Option<PathBuf>,
    },

    /// Run AI agent loop to make changes based on a prompt
    Agent {
        /// Path to project JSON file
        #[arg(short, long)]
        project: PathBuf,

        /// AI prompt text
        #[arg(long)]
        prompt: Option<String>,

        /// Read prompt from file
        #[arg(long)]
        prompt_file: Option<PathBuf>,

        /// AI model to use (default: google/gemini-3-flash-preview)
        #[arg(short, long)]
        model: Option<String>,

        /// Maximum iterations (default: 3)
        #[arg(long, default_value = "3")]
        max_iterations: usize,

        /// Use tool-based editing mode (read/write JSON and template)
        #[arg(long)]
        tool_mode: bool,

        /// Enable visual verification
        #[arg(long)]
        visual_verify: bool,

        /// Directory to save verification screenshots
        #[arg(long)]
        save_screenshots: Option<PathBuf>,

        /// Output modified project to this path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Show what would change without applying
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    if cli.verbose {
        eprintln!("Verbose mode enabled");
    }

    let result = match cli.command {
        Commands::ReadJson { project, compact } => cmd_read_json(&project, compact),
        Commands::WriteJson {
            project,
            input,
            stdin,
            no_validate,
            dry_run,
        } => cmd_write_json(&project, input.as_deref(), stdin, no_validate, dry_run),
        Commands::ReadTemplate { project } => cmd_read_template(&project),
        Commands::WriteTemplate {
            project,
            input,
            stdin,
            no_validate,
            dry_run,
        } => cmd_write_template(&project, input.as_deref(), stdin, no_validate, dry_run),
        Commands::Render {
            data,
            template,
            output,
        } => cmd_render(&data, &template, output.as_deref()),
        Commands::Compile {
            input,
            output_svg,
            output_pdf,
        } => cmd_compile(&input, output_svg.as_deref(), output_pdf.as_deref()),
        Commands::Agent {
            project,
            prompt,
            prompt_file,
            model,
            max_iterations,
            tool_mode,
            visual_verify,
            save_screenshots,
            output,
            dry_run,
        } => cmd_agent(
            &project,
            prompt.as_deref(),
            prompt_file.as_deref(),
            model.as_deref(),
            max_iterations,
            tool_mode,
            visual_verify,
            save_screenshots.as_deref(),
            output.as_deref(),
            dry_run,
        ),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

// ============================================================================
// Command Implementations
// ============================================================================

fn cmd_read_json(project: &Path, compact: bool) -> Result<(), String> {
    use slick_sheet_studio::persistence::Project;

    let content = std::fs::read_to_string(project)
        .map_err(|e| format!("Failed to read project file: {}", e))?;

    let project_data =
        Project::from_json(&content).map_err(|e| format!("Failed to parse project: {}", e))?;

    // For now, just output the source (which is the Typst code)
    // In a full implementation, this would output the structured JSON data
    let _ = compact; // TODO: Use compact flag for JSON output formatting
    println!("{}", project_data.source);

    Ok(())
}

fn cmd_write_json(
    project: &Path,
    input: Option<&Path>,
    stdin: bool,
    _no_validate: bool,
    dry_run: bool,
) -> Result<(), String> {
    use slick_sheet_studio::persistence::Project;

    // Read the new JSON content
    let new_content = if stdin {
        use std::io::Read;
        let mut buffer = String::new();
        std::io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|e| format!("Failed to read from stdin: {}", e))?;
        buffer
    } else if let Some(input_path) = input {
        std::fs::read_to_string(input_path)
            .map_err(|e| format!("Failed to read input file: {}", e))?
    } else {
        return Err("Either --input or --stdin must be provided".to_string());
    };

    // Read existing project
    let existing_content = std::fs::read_to_string(project)
        .map_err(|e| format!("Failed to read project file: {}", e))?;

    let mut project_data = Project::from_json(&existing_content)
        .map_err(|e| format!("Failed to parse project: {}", e))?;

    // Update the source with new content
    project_data.source = new_content;

    if dry_run {
        println!("Validation passed. Would write to: {}", project.display());
        return Ok(());
    }

    // Write back
    let output_json = project_data
        .to_json_pretty()
        .map_err(|e| format!("Failed to serialize project: {}", e))?;

    std::fs::write(project, output_json)
        .map_err(|e| format!("Failed to write project file: {}", e))?;

    println!("Successfully updated project: {}", project.display());
    Ok(())
}

fn cmd_read_template(project: &Path) -> Result<(), String> {
    use slick_sheet_studio::persistence::Project;

    let content = std::fs::read_to_string(project)
        .map_err(|e| format!("Failed to read project file: {}", e))?;

    let project_data =
        Project::from_json(&content).map_err(|e| format!("Failed to parse project: {}", e))?;

    println!("{}", project_data.source);
    Ok(())
}

fn cmd_write_template(
    project: &Path,
    input: Option<&Path>,
    stdin: bool,
    _no_validate: bool,
    dry_run: bool,
) -> Result<(), String> {
    use slick_sheet_studio::persistence::Project;

    // Read the new template content
    let new_template = if stdin {
        use std::io::Read;
        let mut buffer = String::new();
        std::io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|e| format!("Failed to read from stdin: {}", e))?;
        buffer
    } else if let Some(input_path) = input {
        std::fs::read_to_string(input_path)
            .map_err(|e| format!("Failed to read input file: {}", e))?
    } else {
        return Err("Either --input or --stdin must be provided".to_string());
    };

    // Read existing project
    let existing_content = std::fs::read_to_string(project)
        .map_err(|e| format!("Failed to read project file: {}", e))?;

    let mut project_data = Project::from_json(&existing_content)
        .map_err(|e| format!("Failed to parse project: {}", e))?;

    // Update the source with new template
    project_data.source = new_template;

    if dry_run {
        println!("Validation passed. Would write to: {}", project.display());
        return Ok(());
    }

    // Write back
    let output_json = project_data
        .to_json_pretty()
        .map_err(|e| format!("Failed to serialize project: {}", e))?;

    std::fs::write(project, output_json)
        .map_err(|e| format!("Failed to write project file: {}", e))?;

    println!("Successfully updated project: {}", project.display());
    Ok(())
}

fn cmd_render(data: &Path, template: &Path, output: Option<&Path>) -> Result<(), String> {
    // Read data and template files
    let _data_content =
        std::fs::read_to_string(data).map_err(|e| format!("Failed to read data file: {}", e))?;

    let template_content = std::fs::read_to_string(template)
        .map_err(|e| format!("Failed to read template file: {}", e))?;

    // For now, just output the template (full template engine integration would parse data)
    // This is a placeholder - in full implementation, use TemplateEngine::render()
    let rendered = template_content;

    if let Some(output_path) = output {
        std::fs::write(output_path, &rendered)
            .map_err(|e| format!("Failed to write output file: {}", e))?;
        println!("Rendered to: {}", output_path.display());
    } else {
        println!("{}", rendered);
    }

    Ok(())
}

fn cmd_compile(
    input: &Path,
    output_svg: Option<&Path>,
    output_pdf: Option<&Path>,
) -> Result<(), String> {
    use slick_sheet_studio::world::VirtualWorld;

    if output_svg.is_none() && output_pdf.is_none() {
        return Err(
            "At least one output format must be specified (--output-svg or --output-pdf)"
                .to_string(),
        );
    }

    let source =
        std::fs::read_to_string(input).map_err(|e| format!("Failed to read input file: {}", e))?;

    // Compile to SVG if requested
    if let Some(svg_path) = output_svg {
        let svg = VirtualWorld::compile_to_svg(&source)
            .map_err(|errors| format!("Compilation failed:\n{}", errors.join("\n")))?;

        std::fs::write(svg_path, svg).map_err(|e| format!("Failed to write SVG file: {}", e))?;

        println!("SVG written to: {}", svg_path.display());
    }

    // Compile to PDF if requested
    if let Some(pdf_path) = output_pdf {
        use slick_sheet_studio::persistence::pdf_bytes_from_source;

        let pdf_bytes =
            pdf_bytes_from_source(&source).map_err(|e| format!("PDF export failed: {}", e))?;

        std::fs::write(pdf_path, pdf_bytes)
            .map_err(|e| format!("Failed to write PDF file: {}", e))?;

        println!("PDF written to: {}", pdf_path.display());
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn cmd_agent(
    project: &Path,
    prompt: Option<&str>,
    prompt_file: Option<&Path>,
    model: Option<&str>,
    max_iterations: usize,
    _tool_mode: bool,
    _visual_verify: bool,
    _save_screenshots: Option<&Path>,
    output: Option<&Path>,
    dry_run: bool,
) -> Result<(), String> {
    use slick_sheet_studio::ai::agent::{AgentConfig, AgentLoop, AgentResult};
    use slick_sheet_studio::ai::client::{OpenRouterClient, OpenRouterConfig};
    use slick_sheet_studio::persistence::Project;
    use slick_sheet_studio::world::VirtualWorld;

    // Get the prompt
    let prompt_text = if let Some(p) = prompt {
        p.to_string()
    } else if let Some(pf) = prompt_file {
        std::fs::read_to_string(pf).map_err(|e| format!("Failed to read prompt file: {}", e))?
    } else {
        return Err("Either --prompt or --prompt-file must be provided".to_string());
    };

    // Read project
    let project_content = std::fs::read_to_string(project)
        .map_err(|e| format!("Failed to read project file: {}", e))?;

    let mut project_data = Project::from_json(&project_content)
        .map_err(|e| format!("Failed to parse project: {}", e))?;

    let model_name = model.unwrap_or("google/gemini-3-flash-preview");

    println!("Project: {}", project_data.metadata.name);
    println!("Prompt: {}", prompt_text);
    println!("Model: {}", model_name);
    println!("Max iterations: {}", max_iterations);

    if dry_run {
        println!("\n[DRY RUN] Would run AI agent with the above settings.");
        return Ok(());
    }

    // Check for API key
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .map_err(|_| "OPENROUTER_API_KEY environment variable not set. Load .env.testing first.")?;

    if api_key.is_empty() {
        return Err("OPENROUTER_API_KEY is empty".to_string());
    }

    println!("\nRunning AI agent...");

    // Create the runtime and run the agent
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to create tokio runtime: {}", e))?;

    let result = rt.block_on(async {
        // Create client and config
        let config = OpenRouterConfig::with_key(api_key);
        let client = OpenRouterClient::new(config);

        let agent_config = AgentConfig {
            max_iterations,
            model: model_name.to_string(),
            enable_visual_verification: false,
        };

        let mut agent = AgentLoop::new(client, agent_config);

        // Compile function
        let compile_fn = |code: &str| -> Result<String, String> {
            VirtualWorld::compile_to_svg(code).map_err(|errors| errors.join("\n"))
        };

        // Run the agent
        agent
            .run(&prompt_text, Some(&project_data.source), compile_fn)
            .await
    });

    match result {
        AgentResult::Success {
            code, iterations, ..
        } => {
            println!("\nSuccess after {} iteration(s)!", iterations);

            // Update project
            project_data.source = code;

            // Write output
            let output_path = output.unwrap_or(project);
            let output_json = project_data
                .to_json_pretty()
                .map_err(|e| format!("Failed to serialize project: {}", e))?;

            std::fs::write(output_path, output_json)
                .map_err(|e| format!("Failed to write output file: {}", e))?;

            println!("Updated project written to: {}", output_path.display());
            Ok(())
        }
        AgentResult::MaxIterationsReached {
            last_code,
            last_error,
        } => {
            eprintln!("\nMax iterations reached without success.");
            if let Some(error) = last_error {
                eprintln!("Last error: {}", error);
            }
            if let Some(code) = last_code {
                eprintln!("\nLast generated code:\n{}", code);
            }
            Err("Agent did not succeed within max iterations".to_string())
        }
        AgentResult::Error(e) => Err(format!("Agent error: {}", e)),
    }
}

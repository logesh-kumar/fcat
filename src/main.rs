use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use ignore::Walk;
use indicatif::{ProgressBar, ProgressStyle};
use glob::Pattern;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory to search for TypeScript files
    #[arg(default_value = ".")]
    path: String,

    /// Output filename (without extension)
    #[arg(short, long, default_value = "concatenated")]
    output: String,

    /// Estimate token count in output
    #[arg(long)]
    estimate_tokens: bool,

    /// Don't open output directory when done
    #[arg(long)]
    no_open: bool,

    /// Patterns to exclude (e.g., "**/*.test.ts")
    #[arg(short, long)]
    exclude: Vec<String>,

    /// Strip extra whitespace from output
    #[arg(long)]
    strip_spaces: bool,
}

#[derive(Debug)]
struct TypeScriptFile {
    path: PathBuf,
    content: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Create tmp directory if it doesn't exist
    let output_dir = PathBuf::from("tmp");
    fs::create_dir_all(&output_dir)?;
    
    let output_path = output_dir.join(format!("{}.txt", args.output));
    
    println!("{}", "🔍 Searching for TypeScript files...".blue());
    
    // Collect all TypeScript files
    let files = collect_typescript_files(&args.path, &args.exclude)?;
    
    if files.is_empty() {
        anyhow::bail!("No TypeScript files found in the specified path");
    }
    
    println!("{}", format!("Found {} files", files.len()).green());
    
    // Setup progress bar
    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?
            .progress_chars("#>-"),
    );
    
    // Process files
    let mut total_chars = 0;
    let mut output_file = File::create(&output_path)?;
    
    for file in files {
        let separator = "\n\n// ===========================================\n";
        let header = format!("// File: {}\n// ===========================================\n\n",
            file.path.display());
        
        let content = if args.strip_spaces {
            file.content
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| {
                    let trimmed = line.trim();
                    if trimmed.starts_with("//") {
                        trimmed.to_string()
                    } else {
                        // Count leading spaces for indentation
                        let indent_level = line.chars().take_while(|c| c.is_whitespace()).count();
                        let indent = " ".repeat(indent_level);
                        // Format the rest of the line
                        format!("{}{}", indent, trimmed.split_whitespace().collect::<Vec<_>>().join(" "))
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            file.content
        };
        
        write!(output_file, "{}{}{}", separator, header, content)?;
        total_chars += content.len();
        
        pb.inc(1);
    }
    
    pb.finish_with_message("Done!");
    
    // Calculate and show token estimate if requested
    if args.estimate_tokens {
        let estimated_tokens = total_chars / 4;
        println!("{}", format!("\nEstimated tokens: {}", estimated_tokens).magenta());
    }
    
    println!("{}", format!("\n✅ Successfully processed files").green());
    println!("{}", format!("📁 Output saved to: {}", output_path.display()).blue());
    
    // Open output directory if requested
    if !args.no_open {
        if let Err(e) = open::that(output_dir) {
            eprintln!("Failed to open output directory: {}", e);
        }
    }
    
    Ok(())
}

fn collect_typescript_files(root: &str, exclude_patterns: &[String]) -> Result<Vec<TypeScriptFile>> {
    let mut files = Vec::new();
    let walker = Walk::new(root);
    
    for entry in walker.filter_map(Result::ok) {
        let path = entry.path();
        
        // Skip if path matches any exclude pattern
        if should_exclude(path, exclude_patterns) {
            continue;
        }
        
        if is_typescript_file(path) {
            let content = fs::read_to_string(path)
                .with_context(|| format!("Failed to read file: {}", path.display()))?;
                
            files.push(TypeScriptFile {
                path: path.to_path_buf(),
                content,
            });
        }
    }
    
    Ok(files)
}

fn is_typescript_file(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        let ext = extension.to_string_lossy().to_lowercase();
        ext == "ts" || ext == "tsx"
    } else {
        false
    }
}

fn should_exclude(path: &Path, exclude_patterns: &[String]) -> bool {
    // Always exclude node_modules and .d.ts files
    if path.to_string_lossy().contains("node_modules") {
        return true;
    }
    
    if path.to_string_lossy().ends_with(".d.ts") {
        return true;
    }
    
    // Check custom exclude patterns
    exclude_patterns.iter().any(|pattern| {
        let matcher = Pattern::new(pattern).unwrap_or_else(|_| {
            eprintln!("Warning: Invalid exclude pattern: {}", pattern);
            Pattern::new("").unwrap()
        });
        matcher.matches_path(path)
    })
}
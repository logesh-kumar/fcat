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
    /// Directory to search for files
    #[arg(default_value = ".")]
    path: String,

    /// Output filename (without extension)
    #[arg(short, long, default_value = "concatenated")]
    output: String,

    /// File extensions to include (e.g., "ts,tsx,js,jsx")
    #[arg(short, long, default_value = "ts,tsx")]
    extensions: String,

    /// Estimate token count in output
    #[arg(long)]
    estimate_tokens: bool,

    /// Don't open output directory when done
    #[arg(long)]
    no_open: bool,

    /// Patterns to exclude (e.g., "**/*.test.ts")
    #[arg(short, long)]
    exclude: Vec<String>,

    /// Include node_modules directory (overrides default ignore)
    #[arg(long)]
    include_node_modules: bool,

    /// Include all default ignored directories
    #[arg(long)]
    no_default_ignores: bool,

    /// Strip extra whitespace from output
    #[arg(long)]
    strip_spaces: bool,

    /// Include files without extensions
    #[arg(long)]
    include_no_ext: bool,
}

#[derive(Debug)]
struct SourceFile {
    path: PathBuf,
    content: String,
    extension: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Parse extensions into a HashSet for efficient lookup
    let extensions: Vec<String> = args.extensions
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .collect();
        
    println!("{}", format!("🔍 Searching for files with extensions: {}", 
        extensions.join(", ")).blue());
    
    // Create tmp directory if it doesn't exist
    let output_dir = PathBuf::from("tmp");
    fs::create_dir_all(&output_dir)?;
    
    let output_path = output_dir.join(format!("{}.txt", args.output));
    let md_output_path = output_dir.join(format!("{}.md", args.output));
    
    // Collect all matching files
    let files = collect_files(&args.path, &extensions, &args.exclude, args.include_no_ext, &args)?;
    
    if files.is_empty() {
        anyhow::bail!("No matching files found in the specified path");
    }
    
    println!("{}", format!("Found {} files", files.len()).green());
    
    // Setup progress bar
    let pb = ProgressBar::new((files.len() * 2) as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?
            .progress_chars("#>-"),
    );
    
    // Process files for txt output
    let mut total_chars = 0;
    let mut output_file = File::create(&output_path)?;
    let mut md_output = File::create(&md_output_path)?;
    
    // Write MD header with included extensions
    writeln!(md_output, "# Combined Files Structure")?;
    writeln!(md_output, "\nIncluded extensions: {}\n", extensions.join(", "))?;
    
    for file in &files {
        // Write to txt file
        let separator = "\n\n// ===========================================\n";
        let header = format!("// File: {} ({})\n// ===========================================\n\n",
            file.path.display(),
            file.extension.as_deref().unwrap_or("no extension"));
        
        let content = if args.strip_spaces {
            file.content
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| {
                    let trimmed = line.trim();
                    if trimmed.starts_with("//") || trimmed.starts_with("#") {
                        trimmed.to_string()
                    } else {
                        let indent_level = line.chars().take_while(|c| c.is_whitespace()).count();
                        let indent = " ".repeat(indent_level);
                        format!("{}{}", indent, trimmed.split_whitespace().collect::<Vec<_>>().join(" "))
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            file.content.clone()
        };
        
        write!(output_file, "{}{}{}", separator, header, content)?;
        total_chars += content.len();
        
        // Enhanced MD output with file extension
        writeln!(md_output, "## {} ({})", 
            file.path.display(),
            file.extension.as_deref().unwrap_or("no extension"))?;
        
        pb.inc(2);
    }
    
    pb.finish_with_message("Done!");
    
    // Calculate and show token estimate if requested
    if args.estimate_tokens {
        let estimated_tokens = total_chars / 4;
        println!("{}", format!("\nEstimated tokens: {}", estimated_tokens).magenta());
    }
    
    println!("{}", format!("\n✅ Successfully processed files").green());
    println!("{}", format!("📁 Output saved to: {}", output_path.display()).blue());
    println!("{}", format!("📝 Markdown saved to: {}", md_output_path.display()).blue());
    
    // Open output directory if requested
    if !args.no_open {
        if let Err(e) = open::that(output_dir) {
            eprintln!("Failed to open output directory: {}", e);
        }
    }
    
    Ok(())
}

fn collect_files(
    root: &str, 
    extensions: &[String], 
    exclude_patterns: &[String],
    include_no_ext: bool,
    args: &Args
) -> Result<Vec<SourceFile>> {
    let mut files = Vec::new();
    let walker = Walk::new(root);
    
    for entry in walker.filter_map(Result::ok) {
        let path = entry.path();
        
        // Skip if path matches any exclude pattern
        if should_exclude(path, exclude_patterns, args) {
            continue;
        }
        
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            if extensions.contains(&ext) {
                let content = fs::read_to_string(path)
                    .with_context(|| format!("Failed to read file: {}", path.display()))?;
                    
                files.push(SourceFile {
                    path: path.to_path_buf(),
                    content,
                    extension: Some(ext),
                });
            }
        } else if include_no_ext {
            // Include files without extension if flag is set
            let content = fs::read_to_string(path)
                .with_context(|| format!("Failed to read file: {}", path.display()))?;
                
            files.push(SourceFile {
                path: path.to_path_buf(),
                content,
                extension: None,
            });
        }
    }
    
    Ok(files)
}

fn should_exclude(path: &Path, exclude_patterns: &[String], args: &Args) -> bool {
    // Default ignore patterns unless disabled
    if !args.no_default_ignores {
        let default_ignores = [
            "node_modules",
            ".git",
            "target",
            "dist",
            "build",
            ".cache",
            ".temp",
            "tmp",
        ];
        
        // Skip node_modules unless explicitly included
        if !args.include_node_modules && path.to_string_lossy().contains("node_modules") {
            return true;
        }
        
        // Check other default ignores
        for pattern in default_ignores.iter() {
            if path.to_string_lossy().contains(pattern) {
                return true;
            }
        }
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
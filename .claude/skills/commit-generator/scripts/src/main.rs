use anyhow::{Context, Result};
use clap::Parser;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Parser)]
#[command(name = "commit-analyzer")]
#[command(about = "Analyze git changes and suggest commit messages for Backbone Framework")]
struct Args {
    /// Repository path (default: current directory)
    #[arg(short, long)]
    repo_path: Option<String>,

    /// Verbose output with detailed analysis
    #[arg(short, long)]
    verbose: bool,

    /// Only show suggested commit message
    #[arg(short, long)]
    quiet: bool,
}

#[derive(Debug, Clone)]
pub struct FileCategory {
    name: String,
    pattern: Regex,
}

impl FileCategory {
    fn new(name: &str, pattern: &str) -> Result<Self> {
        Ok(Self {
            name: name.to_string(),
            pattern: Regex::new(pattern)?,
        })
    }
}

pub struct CommitAnalyzer {
    repo_path: String,
    categories: Vec<FileCategory>,
}

impl CommitAnalyzer {
    pub fn new(repo_path: Option<String>) -> Self {
        let repo_path = repo_path.unwrap_or_else(|| ".".to_string());

        let categories = vec![
            FileCategory::new("schema", r"libs/modules/([^/]+)/schema/.*\.yaml$").unwrap(),
            FileCategory::new("generated", r"src/generated/.*\.rs$").unwrap(),
            FileCategory::new("domain", r"src/domain/.*\.rs$").unwrap(),
            FileCategory::new("application", r"src/application/.*\.rs$").unwrap(),
            FileCategory::new("infrastructure", r"src/infrastructure/.*\.rs$").unwrap(),
            FileCategory::new("presentation", r"src/presentation/.*\.rs$").unwrap(),
            FileCategory::new("tests", r"(tests/.*\.rs$|.*_test\.rs$)").unwrap(),
            FileCategory::new("docs", r".*\.md$").unwrap(),
            FileCategory::new("config", r".*\.(yml|yaml|toml|env|json)$").unwrap(),
            FileCategory::new("proto", r".*\.proto$").unwrap(),
        ];

        Self { repo_path, categories }
    }

    fn run_git_command(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.repo_path)
            .output()
            .with_context(|| format!("Failed to run git command: git {}", args.join(" ")))?;

        if !output.status.success() {
            anyhow::bail!("Git command failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub fn get_changed_files(&self) -> Result<Vec<String>> {
        let output = self.run_git_command(&["diff", "--cached", "--name-only"])?;

        if output.is_empty() {
            Ok(vec![])
        } else {
            Ok(output.split('\n').map(|s| s.to_string()).collect())
        }
    }

    pub fn get_file_stats(&self) -> Result<String> {
        self.run_git_command(&["diff", "--cached", "--stat"])
    }

    pub fn categorize_files(&self, files: &[String]) -> HashMap<String, Vec<String>> {
        let mut categories: HashMap<String, Vec<String>> = HashMap::new();

        for file in files {
            let mut categorized = false;
            for category in &self.categories {
                if category.pattern.is_match(file) {
                    categories.entry(category.name.clone())
                        .or_insert_with(Vec::new)
                        .push(file.clone());
                    categorized = true;
                    break;
                }
            }

            if !categorized {
                categories.entry("other".to_string())
                    .or_insert_with(Vec::new)
                    .push(file.clone());
            }
        }

        categories
    }

    pub fn detect_module(&self, file_path: &str) -> String {
        // Extract module from libs/modules/{module}/
        if let Some(captures) = Regex::new(r"libs/modules/([^/]+)/").unwrap().captures(file_path) {
            return captures.get(1).unwrap().as_str().to_string();
        }

        // Extract app from apps/{app}/
        if let Some(captures) = Regex::new(r"apps/([^/]+)/").unwrap().captures(file_path) {
            return captures.get(1).unwrap().as_str().to_string();
        }

        // Check for framework-wide changes
        if file_path.starts_with("libs/") || file_path.starts_with("framework-cli") {
            return "framework".to_string();
        }

        "backbone".to_string()
    }

    pub fn analyze_change_type(&self, categories: &HashMap<String, Vec<String>>) -> &str {
        if categories.contains_key("schema") {
            "feat"
        } else if categories.contains_key("domain") || categories.contains_key("application") {
            "feat"
        } else if categories.contains_key("infrastructure") || categories.contains_key("presentation") {
            "feat"
        } else if categories.contains_key("tests") {
            "test"
        } else if categories.contains_key("docs") {
            "docs"
        } else if categories.contains_key("config") {
            "chore"
        } else if categories.contains_key("generated") {
            "feat"  // Usually part of schema changes
        } else {
            "feat"
        }
    }

    pub fn suggest_scope(&self, files: &[String], categories: &HashMap<String, Vec<String>>) -> String {
        // Count modules
        let mut module_counts: HashMap<String, usize> = HashMap::new();
        for file in files {
            let module = self.detect_module(file);
            *module_counts.entry(module).or_insert(0) += 1;
        }

        if !module_counts.is_empty() {
            // Return module with most changes
            return module_counts.into_iter()
                .max_by_key(|(_, count)| *count)
                .map(|(module, _)| module)
                .unwrap_or_else(|| "framework".to_string());
        }

        // Fallback to category-based scopes
        if categories.contains_key("schema") {
            "schema".to_string()
        } else if categories.contains_key("docs") {
            "docs".to_string()
        } else if categories.contains_key("tests") {
            "test".to_string()
        } else if categories.contains_key("config") {
            "config".to_string()
        } else {
            "framework".to_string()
        }
    }

    pub fn generate_description(&self, categories: &HashMap<String, Vec<String>>, change_type: &str) -> String {
        if categories.contains_key("schema") {
            let schema_files = categories.get("schema").unwrap();
            if schema_files.len() == 1 {
                let file = &schema_files[0];
                if file.contains("user.model.yaml") {
                    return "Update user entity schema".to_string();
                } else if file.contains("profile") || file.contains("avatar") {
                    return "Add user profile features".to_string();
                } else if file.contains("role") || file.contains("permission") {
                    return "Update role and permission structure".to_string();
                }
            } else {
                return "Update multiple schema definitions".to_string();
            }
        }

        if categories.contains_key("domain") {
            return "Update domain logic".to_string();
        }

        if categories.contains_key("application") {
            return "Update application services".to_string();
        }

        if categories.contains_key("infrastructure") {
            return "Update infrastructure layer".to_string();
        }

        if categories.contains_key("presentation") {
            return "Update presentation layer".to_string();
        }

        if categories.contains_key("tests") {
            return "Add or update tests".to_string();
        }

        if categories.contains_key("docs") {
            return "Update documentation".to_string();
        }

        if categories.contains_key("config") {
            return "Update configuration".to_string();
        }

        // Fallback based on change type
        match change_type {
            "feat" => "Add new functionality".to_string(),
            "fix" => "Fix reported issues".to_string(),
            "refactor" => "Refactor code structure".to_string(),
            "perf" => "Optimize performance".to_string(),
            "test" => "Add or update tests".to_string(),
            "docs" => "Update documentation".to_string(),
            "chore" => "Update project configuration".to_string(),
            _ => "Update project files".to_string(),
        }
    }

    pub fn suggest_commit_message(&self) -> Result<String> {
        let files = self.get_changed_files()?;

        if files.is_empty() {
            return Ok("No staged changes found".to_string());
        }

        let categories = self.categorize_files(&files);
        let change_type = self.analyze_change_type(&categories);
        let scope = self.suggest_scope(&files, &categories);
        let description = self.generate_description(&categories, change_type);

        Ok(format!("{}({}): {}", change_type, scope, description))
    }

    pub fn print_analysis(&self) -> Result<()> {
        let files = self.get_changed_files()?;

        if files.is_empty() {
            println!("No staged changes found");
            return Ok(());
        }

        println!("=== Commit Analysis ===");
        println!("Files changed: {}", files.len());

        let categories = self.categorize_files(&files);
        for (category, file_list) in &categories {
            if !file_list.is_empty() {
                println!("\n{}:", category.to_uppercase());
                for file_path in file_list {
                    println!("  - {}", file_path);
                }
            }
        }

        println!("\n=== Suggested Commit Message ===");
        println!("{}", self.suggest_commit_message()?);

        println!("\n=== File Statistics ===");
        println!("{}", self.get_file_stats()?);

        Ok(())
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let analyzer = CommitAnalyzer::new(args.repo_path);

    if args.quiet {
        println!("{}", analyzer.suggest_commit_message()?);
    } else if args.verbose {
        analyzer.print_analysis()?;
    } else {
        println!("{}", analyzer.suggest_commit_message()?);
    }

    Ok(())
}
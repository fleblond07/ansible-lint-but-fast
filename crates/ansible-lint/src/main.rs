use std::path::PathBuf;
use std::process;

use anyhow::Result;
use clap::Parser;

use ansible_lint_core::{
    config::Config,
    fix::fix_file,
    formatter::get_formatter,
    registry::{Profile, RuleRegistry},
    rules::all_rules,
    runner::{count_errors, LintRunner},
};

#[derive(Parser)]
#[command(
    name = "ansible-rust-lint",
    version = env!("CARGO_PKG_VERSION"),
    about = "A linting tool for Ansible playbooks, roles, and collections",
    long_about = None,
)]
struct Cli {
    /// Files or directories to lint (default: current directory)
    #[arg(value_name = "FILE_OR_DIR")]
    paths: Vec<PathBuf>,

    /// Output format
    #[arg(short = 'f', long = "format", value_name = "FORMAT", default_value = "brief")]
    format: String,

    /// Config file path
    #[arg(short = 'c', long = "config", value_name = "FILE")]
    config_file: Option<PathBuf>,

    /// Rules/tags to skip (comma-separated or repeatable)
    #[arg(short = 'x', long = "skip-list", value_name = "RULES", value_delimiter = ',')]
    skip_list: Vec<String>,

    /// Rules/tags to treat as warnings
    #[arg(short = 'w', long = "warn-list", value_name = "RULES", value_delimiter = ',')]
    warn_list: Vec<String>,

    /// Rules/tags to enable (opt-in)
    #[arg(long = "enable-list", value_name = "RULES", value_delimiter = ',')]
    enable_list: Vec<String>,

    /// Lint profile to use
    #[arg(long = "profile", value_name = "PROFILE")]
    profile: Option<Profile>,

    /// Project root directory
    #[arg(long = "project-dir", value_name = "DIR")]
    project_dir: Option<PathBuf>,

    /// Exclude paths (repeatable)
    #[arg(long = "exclude", value_name = "PATH")]
    exclude: Vec<String>,

    /// Disable colour output
    #[arg(long = "no-color")]
    no_color: bool,

    /// Treat warnings as errors
    #[arg(long = "strict")]
    strict: bool,

    /// Offline mode
    #[arg(long = "offline")]
    offline: bool,

    /// List all rules and exit
    #[arg(short = 'L', long = "list-rules")]
    list_rules: bool,

    /// List available profiles and exit
    #[arg(short = 'P', long = "list-profiles")]
    list_profiles: bool,

    /// List all tags and exit
    #[arg(short = 'T', long = "list-tags")]
    list_tags: bool,

    /// Generate a .ansible-lint-ignore file from current violations
    #[arg(long = "generate-ignore")]
    generate_ignore: bool,

    /// Auto-fix violations (comma-separated rule list, or empty for all fixable rules)
    #[arg(long = "fix", value_name = "RULES", value_delimiter = ',', num_args = 0..)]
    fix: Option<Vec<String>>,

    /// Increase verbosity (repeatable)
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    verbose: u8,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("ansible-lint: {e}");
        process::exit(2);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    let registry = RuleRegistry::new(all_rules());

    // Handle info-only flags.
    if cli.list_rules {
        print_rules(&registry, cli.profile.unwrap_or_default());
        return Ok(());
    }
    if cli.list_profiles {
        print_profiles();
        return Ok(());
    }
    if cli.list_tags {
        print_tags(&registry);
        return Ok(());
    }

    // Load config.
    let project_dir = cli.project_dir
        .clone()
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));

    let mut config = if let Some(ref cfg_path) = cli.config_file {
        Config::from_file(cfg_path)?
    } else {
        let (cfg, found) = Config::load(&project_dir)?;
        if cli.verbose > 0 {
            if let Some(p) = found {
                eprintln!("INFO: Using config file: {}", p.display());
            }
        }
        cfg
    };

    config.merge_cli(
        cli.profile,
        cli.skip_list,
        cli.warn_list,
        cli.enable_list,
        cli.exclude,
        cli.offline,
        cli.strict,
    );

    let input_paths = if cli.paths.is_empty() {
        vec![project_dir.clone()]
    } else {
        cli.paths
    };

    let runner = LintRunner::new(&config, &registry, project_dir);
    let results = runner.run(&input_paths)?;

    // Handle --generate-ignore.
    if cli.generate_ignore {
        for m in &results {
            println!("{} {}", m.filename.display(), m.rule_id);
        }
        return Ok(());
    }

    // Handle --fix.
    if let Some(ref fix_rules) = cli.fix {
        let unique_files: std::collections::HashSet<&std::path::PathBuf> =
            results.iter().map(|m| &m.filename).collect();
        let mut total_fixes = 0;
        for file_path in &unique_files {
            match fix_file(file_path, &results, fix_rules) {
                Ok(n) => {
                    total_fixes += n;
                    if n > 0 && cli.verbose > 0 {
                        eprintln!("Fixed {n} issue(s) in {}", file_path.display());
                    }
                }
                Err(e) => eprintln!("Warning: could not fix {}: {e}", file_path.display()),
            }
        }
        eprintln!("Applied {total_fixes} fix(es).");
        return Ok(());
    }

    // Format and print results.
    let use_color = !cli.no_color && atty_stdout();
    let formatter = get_formatter(&cli.format);
    let output = formatter.format(&results, use_color);
    if !output.is_empty() {
        println!("{output}");
    }

    // Stats summary.
    let error_count = count_errors(&results, config.strict);
    let warning_count = results.iter().filter(|m| {
        m.severity == ansible_lint_core::rule::Severity::Warning
    }).count();

    if cli.verbose > 0 || !results.is_empty() {
        eprintln!("Finished with {error_count} failure(s), {warning_count} warning(s)");
    }

    if error_count > 0 {
        process::exit(1);
    }

    Ok(())
}

fn print_rules(registry: &RuleRegistry, _profile: Profile) {
    println!("{:<30} {:<12} DESCRIPTION", "ID", "SEVERITY");
    println!("{}", "-".repeat(80));
    for rule in registry.all_rules() {
        println!("{:<30} {:<12} {}", rule.id(), rule.severity().to_string(), rule.description());
    }
}

fn print_profiles() {
    use ansible_lint_core::registry::Profile::*;
    for profile in [Min, Basic, Moderate, Safety, Shared, Production] {
        println!("{:<12} {}", profile.to_string(), profile.description());
    }
}

fn print_tags(registry: &RuleRegistry) {
    for tag in registry.all_tags() {
        println!("{tag}");
    }
}

fn atty_stdout() -> bool {
    // Simple check: if TERM is set and not "dumb", assume color is supported.
    std::env::var("NO_COLOR").is_err()
        && std::env::var("TERM").is_ok_and(|t| t != "dumb")
}

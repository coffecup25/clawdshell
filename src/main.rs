use std::io::IsTerminal;
use crossterm::terminal;
use std::process;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let debug = env::var("CLAWDSHELL_DEBUG").is_ok();

    // Load config
    let mut config = match clawdshell::config::Config::load() {
        Ok(c) => c,
        Err(e) => {
            if debug { eprintln!("[clawdshell] config error: {}", e); }
            clawdshell::config::Config::default()
        }
    };

    // --- Pre-parse: intercept -c, -l, and unrecognized flags ---
    if args.len() > 1 {
        // Handle -c "command" [args...] — POSIX shell compatibility
        if args[1] == "-c" {
            if args.len() < 3 {
                eprintln!("clawdshell: -c requires a command argument");
                process::exit(1);
            }
            let shell = get_fallback_shell(&config);
            let extra_args: Vec<String> = args[3..].to_vec();
            match clawdshell::shell::run_command_via_shell(&shell, &args[2], &extra_args) {
                Ok(status) => process::exit(status.code().unwrap_or(1)),
                Err(e) => { eprintln!("clawdshell: {}", e); process::exit(1); }
            }
        }

        match args[1].as_str() {
            "-l" => { /* Login shell flag, no-op — fall through to normal startup */ }
            "--install" => {
                clawdshell::install::install(&mut config);
                return;
            }
            "--uninstall" => {
                clawdshell::install::uninstall(&config);
                return;
            }
            "--set-tool" => {
                if args.len() < 3 {
                    eprintln!("clawdshell: --set-tool requires a name");
                    process::exit(1);
                }
                config.defaults.tool = args[2].clone();
                if let Err(e) = config.save_to(&clawdshell::config::Config::config_path()) {
                    eprintln!("clawdshell: {}", e);
                    process::exit(1);
                }
                println!("Default tool set to: {}", args[2]);
                return;
            }
            "--companion" => {
                let companion = load_or_create_companion(&mut config);
                print!("{}", clawdshell::companion::card::render_card(&companion));
                return;
            }
            "--version" => {
                println!("clawdshell {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            "--help" | "-h" => {
                print_help();
                return;
            }
            "--" => { /* Tool args follow */ }
            arg if arg.starts_with('-') && !arg.starts_with("--") => {
                // Unrecognized short flag — forward to fallback shell
                let shell = get_fallback_shell(&config);
                match clawdshell::shell::forward_to_fallback(&shell, &args[1..]) {
                    Ok(status) => process::exit(status.code().unwrap_or(1)),
                    Err(e) => { eprintln!("clawdshell: {}", e); process::exit(1); }
                }
            }
            _ => {}
        }
    }

    // --- Normal startup flow ---
    let tool_args: Vec<String> = if let Some(pos) = args.iter().position(|a| a == "--") {
        args[pos + 1..].to_vec()
    } else {
        vec![]
    };

    let first_launch = config.companion.seed.is_none();
    let companion = load_or_create_companion(&mut config);

    // Non-TTY: skip tool, go to fallback
    if !std::io::stdin().is_terminal() {
        let shell = get_fallback_shell(&config);
        let _ = clawdshell::shell::spawn_fallback_shell(&shell);
        return;
    }

    let shell = get_fallback_shell(&config);

    // Inherit the user's shell environment (PATH, etc.) so we can find tools
    // that are installed via nvm, cargo, homebrew, etc.
    clawdshell::detect::inherit_shell_environment(&shell);

    let tool_name = config.defaults.tool.clone();
    let tool_config = config.tools.get(&tool_name);
    let command_override = tool_config.and_then(|tc| tc.command.as_deref());
    let tool_path = clawdshell::detect::resolve_tool_binary(&tool_name, command_override);

    // Show greeting
    if config.companion.enabled {
        let width = terminal::size().map(|(w, _)| w).unwrap_or(80);
        if first_launch {
            println!("A companion has appeared!");
            let _ = clawdshell::companion::animate::play_idle(&companion, 15);
            println!("Meet {}!\n", companion.name);
        }
        print!("{}", clawdshell::greeting::render_greeting(&tool_name, &shell, &companion, width));
        // Clear screen immediately so the tool gets a clean terminal
        print!("\x1b[2J\x1b[H");
        let _ = std::io::Write::flush(&mut std::io::stdout());
    }

    clawdshell::shell::setup_signal_forwarding();

    match tool_path {
        Some(path) => {
            let mut all_args: Vec<String> = tool_config.map(|tc| tc.args.clone()).unwrap_or_default();
            all_args.extend(tool_args);
            match clawdshell::shell::spawn_tool(&path.to_string_lossy(), &all_args) {
                Ok(_) => {
                    if config.companion.enabled {
                        let width = terminal::size().map(|(w, _)| w).unwrap_or(80);
                        if width >= 100 {
                            let lines = clawdshell::companion::render::render_sprite(&companion, 0);
                            let info = [format!("dropping to {}...", shell), format!("type '{}' to come back", tool_name)];
                            let empty = String::new();
                            println!();
                            for (i, line) in lines.iter().enumerate() {
                                println!("{}   {}", line, if i < info.len() { &info[i] } else { &empty });
                            }
                        } else {
                            println!("\n{} dropping to {}...", clawdshell::companion::render::render_face(&companion), shell);
                        }
                    }
                }
                Err(e) => eprintln!("clawdshell: failed to launch {}: {}", tool_name, e),
            }
        }
        None => {
            if config.companion.enabled {
                let width = terminal::size().map(|(w, _)| w).unwrap_or(80);
                if width >= 100 {
                    let mut err_companion = companion.clone();
                    err_companion.eye = "×";
                    let lines = clawdshell::companion::render::render_sprite(&err_companion, 0);
                    let info = [format!("'{}' not found in PATH", tool_name), format!("falling back to {}", shell)];
                    let empty = String::new();
                    for (i, line) in lines.iter().enumerate() {
                        eprintln!("{}   {}", line, if i < info.len() { &info[i] } else { &empty });
                    }
                } else {
                    eprintln!("{} '{}' not found in PATH, falling back to {}", clawdshell::companion::render::render_face(&companion), tool_name, shell);
                }
            } else {
                eprintln!("clawdshell: '{}' not found, falling back to {}", tool_name, shell);
            }
        }
    }

    let _ = clawdshell::shell::spawn_fallback_shell(&shell);
}

fn get_fallback_shell(config: &clawdshell::config::Config) -> String {
    config.defaults.fallback_shell.clone()
        .unwrap_or_else(clawdshell::detect::detect_fallback_shell)
}

fn load_or_create_companion(config: &mut clawdshell::config::Config) -> clawdshell::companion::Companion {
    let seed = match &config.companion.seed {
        Some(s) => s.clone(),
        None => {
            let mut buf = [0u8; 8];
            getrandom::getrandom(&mut buf).expect("Failed to generate random seed");
            let seed = format!("{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}", buf[0], buf[1], buf[2], buf[3], buf[4], buf[5]);
            config.companion.seed = Some(seed.clone());
            let _ = config.save_to(&clawdshell::config::Config::config_path());
            seed
        }
    };
    clawdshell::companion::generate(&seed)
}

fn print_help() {
    println!("clawdshell {} — a login shell that launches AI coding tools", env!("CARGO_PKG_VERSION"));
    println!("You weren't using your terminal anyways.\n");
    print!("{}", include_str!("help_extra.txt"));
}

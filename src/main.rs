#![allow(unused_variables, unused_mut, dead_code)]
use crate::gui_main::{run_gui_loop, sekai_initialize};
use crate::rns::create_dmk_sekai;
use crate::utils::globals::set_world_dir;
use crate::utils::{debug_mode, find_root, log};
use clap::{Parser, Subcommand};
use deemak::*;
use raylib::ffi::{SetConfigFlags, SetTargetFPS};
use raylib::prelude::get_monitor_width;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct DeemakArgs {
    /// Path to the Deemak Encrypted Sekai (World) file or directory
    #[arg()]
    sekai: std::path::PathBuf,

    /// Enable debug mode for more verbose logging
    #[arg(long, default_value_t = false)]
    debug: bool,

    /// Run the application in web mode (requires a web server)
    #[arg(long, default_value_t = false)]
    web: bool,

    /// Subcommand to execute
    #[command(subcommand)]
    encrypt: Option<DeemakEncrypt>,
}

#[derive(Subcommand, Debug)]
enum DeemakEncrypt {
    /// Create new Deemak Encrypted file from your existing Sekai directory
    Create {
        /// Password for encryption
        #[arg(short, long)]
        password: Option<String>,

        /// Output directory (defaults to current directory)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
    },

    /// Restore Sekai from a Deemak Encrypted file
    Restore {
        /// Output directory (defaults to current directory)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
    },
}

fn main() {
    let args = DeemakArgs::parse();
    // Set Debug Mode if given
    if args.debug {
        DEBUG_MODE.set(true).expect("DEBUG_MODE already set");
        unsafe {
            std::env::set_var("RUST_BACKTRACE", "1");
        }
    }

    log::log_info("Application", "Starting DEEMAK Shell");

    // get absolute path to the sekai directory
    let sekai_path = if args.sekai.is_absolute() {
        args.sekai.clone()
    } else {
        std::env::current_dir()
            .expect("Failed to get current directory")
            .join(&args.sekai)
    };

    if !sekai_path.exists() {
        log::log_error(
            "SEKAI",
            &format!("Sekai directory does not exist: {}", sekai_path.display()),
        );
        eprintln!(
            "Error: Sekai directory does not exist: {}",
            sekai_path.display()
        );
        std::process::exit(1);
    }
    log::log_info(
        "SEKAI",
        &format!("Sekai directory provided: {sekai_path:?}"),
    );

    log::log_info("Application", "Creating Deemak Encrypted Sekai file");
    if let Some(encrypt_cmd) = args.encrypt {
        match encrypt_cmd {
            DeemakEncrypt::Create {
                password: _,
                output,
            } => {
                // Get input password securely
                let password = match dialoguer::Password::new()
                    .with_prompt("Enter password for your Sekai's Deemak encryption")
                    .with_confirmation("Confirm password", "Passwords don't match!")
                    .interact()
                {
                    Ok(pwd) => pwd,
                    Err(e) => {
                        log::log_error("SEKAI", &format!("Password input failed: {e}"));
                        eprintln!("Error: Password input failed: {e}");
                        std::process::exit(1);
                    }
                };
                let output_path = output.unwrap_or_else(|| {
                    std::env::current_dir()
                        .expect("Failed to get current directory")
                        .join("deemak_sekai.dmk")
                });

                // Handle encryption
                match create_dmk_sekai::deemak_encrypt_sekai(&sekai_path, &output_path, &password) {
                    Ok(_) => {
                        log::log_info(
                            "SEKAI",
                            &format!(
                                "Successfully created Deemak file at: {}",
                                sekai_path.display()
                            ),
                        );
                    }
                    Err(e) => {
                        log::log_error("SEKAI", &format!("Encryption failed: {e}"));
                        eprintln!("Error: Encryption failed: {e}");
                        std::process::exit(1);
                    }
                }
            }
            DeemakEncrypt::Restore { output } => {
                log::log_info("SEKAI", "Restoring Sekai from Deemak Encrypted file");
                let output_path = output.unwrap_or_else(|| {
                    std::env::current_dir()
                        .expect("Failed to get current directory")
                        .join("default.deemak")
                });

                match create_dmk_sekai::original_from_encrypted_sekai(&sekai_path, &output_path) {
                    Ok(restored_path) => {
                        log::log_info(
                            "SEKAI",
                            &format!(
                                "Successfully restored Sekai to: {}",
                                restored_path.display()
                            ),
                        );
                    }
                    Err(e) => {
                        log::log_error("SEKAI", &format!("Restoration failed: {e}"));
                        eprintln!("Error: Restoration failed: {e}");
                        std::process::exit(1);
                    }
                }
            }
        }
    }

    // Check for HOME directory validity and set global SEKAI_DIR accordingly
    match find_root::check_home(&sekai_path) {
        Ok(Some(sekai_dir)) => {
            log::log_info(
                "SEKAI",
                &format!("Found root directory for Sekai: {}", sekai_dir.display()),
            );
            // Set the global Sekai directory
            set_world_dir(sekai_dir);
        }
        Ok(None) => {
            log::log_error(
                "SEKAI",
                "Failed to find root directory for Sekai. No HOME location found. Exiting.",
            );
            eprintln!("Error: Failed to find root directory for Sekai. Exiting.");
            std::process::exit(1);
        }
        Err(e) => {
            log::log_error(
                "SEKAI",
                &format!("Process failed while finding Sekai HOME. Error: {e}. Exiting."),
            );
            eprintln!("Process failed while finding Sekai HOME. Error: {e}. Exiting.");
            std::process::exit(1);
        }
    }

    // NOTE: All Directory operations and variables settings should be done before this point.
    //
    // We have 2 modes, the web and the raylib gui. The web argument runs it on the web, else
    // raylib gui is set by default.
    //
    // NOTE: #############    WEB USAGE    #############
    //
    // Initialize the server if --web argument is provided
    if args.web {
        // TODO: Remove the extra sekai_no_hajimari call, it will be shifted to the server module
        // later on.
        sekai_initialize(&sekai_path);
        log::log_info("Application", "Running in web mode");
        // server::launch_web(sekai_dir.clone().unwrap());
        let _ = deemak::server::server();
        return;
    }

    // NOTE: #############    RAYLIB GUI USAGE    #############
    //
    // Initialize Raylib window
    unsafe {
        SetConfigFlags(4);
        SetTargetFPS(60);
    }
    let loglevel = if !debug_mode() {
        raylib::consts::TraceLogLevel::LOG_ERROR
    } else {
        raylib::consts::TraceLogLevel::LOG_ALL
    };

    let (mut rl, thread) = raylib::init()
        .log_level(loglevel)
        .size(800, 600)
        .title("DEEMAK Shell")
        .build();
    let font_size = get_monitor_width(0) as f32 / 73.5;
    rl.set_trace_log(loglevel);
    // Disable escape key exit to prevent accidental application closure
    unsafe {
        raylib::ffi::SetExitKey(0i32);
    }
    log::log_info("Application", "DEEMAK initialized successfully");

    // Show login screen before menu
    if !deemak::login::show_login(&mut rl, &thread, font_size) {
        log::log_info("Application", "Login aborted by user.");
        return; // Exit if window closed during login
    }

    // Run the GUI loop
    run_gui_loop(&mut rl, &thread, font_size);
}

use std::process;
use tiktok_crosspost::config_wizard;

fn main() {
    println!("Starting TikTok Cross-posting Configuration Wizard...");

    match config_wizard::run_wizard() {
        Ok(config) => {
            // Ask if user wants to export environment variables
            let export_env = dialoguer::Confirm::new()
                .with_prompt("Do you want to export the configuration as environment variables to a .env file?")
                .default(true)
                .interact()
                .unwrap_or(false);

            if export_env {
                let env_path = dialoguer::Input::<String>::new()
                    .with_prompt("Where do you want to save the .env file?")
                    .default(".env".to_string())
                    .interact()
                    .unwrap_or(".env".to_string());

                match config_wizard::generate_env_file(&config, &env_path) {
                    Ok(_) => println!("Environment variables exported to {}", env_path),
                    Err(e) => {
                        eprintln!("Failed to export environment variables: {}", e);
                        process::exit(1);
                    }
                }
            }

            println!("Configuration completed successfully!");
        }
        Err(e) => {
            eprintln!("Error during configuration: {}", e);
            process::exit(1);
        }
    }
}

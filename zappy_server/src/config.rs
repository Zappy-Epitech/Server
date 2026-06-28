//! Command-line configuration.
//!
//! Defines [`ServerConfig`], the `clap`-derived struct that maps CLI arguments
//! (port, map size, team names, clients-per-team, frequency, `--bonus`) into the
//! parameters used to build the [`World`](crate::game::world::World) and run the
//! server, along with [`ServerConfig::validate`] for sanity-checking them.

use clap::Parser;

/// Configuration structure for the Zappy server.
/// 
/// It contains all the necessary parameters to initialize and run the server,
/// parsed from command-line arguments using `clap`.
#[derive(Parser, Debug, Clone)]
#[command(name = "zappy_server")]
#[command(about = "Zappy server", long_about = None)]
pub struct ServerConfig {
    /// Port number on which the server will listen.
    #[arg(short = 'p', default_value_t = 4242)]
    pub port: u16,

    /// Width of the world map (Trantor).
    #[arg(short = 'x', default_value_t = 10)]
    pub width: u32,

    /// Height of the world map (Trantor).
    #[arg(short = 'y', default_value_t = 10)]
    pub height: u32,

    /// List of team names.
    #[arg(short = 'n', num_args = 1.., default_values = ["Team1", "Team2"])]
    pub teams: Vec<String>,

    /// Maximum number of authorized clients per team at start.
    #[arg(short = 'c', default_value_t = 3)]
    pub clients_nb: usize,

    /// Frequency reciprocal for action execution time (default is 100).
    #[arg(short = 'f', default_value_t = 100)]
    pub freq: u32,

    /// Enable the Ratatui TUI dashboard (bonus feature).
    #[arg(short = 'b', long = "bonus")]
    pub bonus: bool,
}

impl ServerConfig {
    /// Validates the parsed arguments, printing every offending parameter and
    /// exiting the process with status code `84` if any are invalid (zero map
    /// dimensions, zero clients/frequency, or no team names).
    pub fn validate(&self) {
        let mut errors = Vec::new();
        if self.width == 0 {
            errors.push("Width (-x) must be greater than 0.");
        }
        if self.height == 0 {
            errors.push("Height (-y) must be greater than 0.");
        }
        if self.clients_nb == 0 {
            errors.push("Number of authorized clients per team (-c) must be greater than 0.");
        }
        if self.freq == 0 {
            errors.push("Frequency (-f) must be greater than 0.");
        }
        if self.teams.is_empty() {
            errors.push("You must provide at least one team name (-n).");
        }

        if !errors.is_empty() {
            println!("Invalid parameters:");
            for err in errors {
                println!("  - {}", err);
            }
            std::process::exit(84);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_parsing() {
        let args = vec![
            "zappy_server",
            "-p", "4242",
            "-x", "10",
            "-y", "20",
            "-n", "team1", "team2",
            "-c", "5",
            "-f", "50"
        ];
        let config = ServerConfig::parse_from(args);

        assert_eq!(config.port, 4242);
        assert_eq!(config.width, 10);
        assert_eq!(config.height, 20);
        assert_eq!(config.teams, vec!["team1".to_string(), "team2".to_string()]);
        assert_eq!(config.clients_nb, 5);
        assert_eq!(config.freq, 50);
    }

    #[test]
    fn test_server_config_default_freq() {
        let args = vec![
            "zappy_server",
            "-p", "4242",
            "-x", "10",
            "-y", "10",
            "-n", "team1",
            "-c", "2"
        ];
        let config = ServerConfig::parse_from(args);

        assert_eq!(config.freq, 100);
    }
}

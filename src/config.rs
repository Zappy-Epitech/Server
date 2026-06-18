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
    #[arg(short = 'p')]
    pub port: u16,

    /// Width of the world map (Trantor).
    #[arg(short = 'x')]
    pub width: u32,

    /// Height of the world map (Trantor).
    #[arg(short = 'y')]
    pub height: u32,

    /// List of team names.
    #[arg(short = 'n', num_args = 1..)]
    pub teams: Vec<String>,

    /// Maximum number of authorized clients per team at start.
    #[arg(short = 'c')]
    pub clients_nb: usize,

    /// Frequency reciprocal for action execution time (default is 100).
    #[arg(short = 'f', default_value_t = 100)]
    pub freq: u32,

    /// Enable the Ratatui TUI dashboard (bonus feature).
    #[arg(short = 'b', long = "bonus")]
    pub bonus: bool,
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

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "zappy_server")]
#[command(about = "Zappy server", long_about = None)]
pub struct ServerConfig {
    #[arg(short = 'p')]
    pub port: u16,

    #[arg(short = 'x')]
    pub width: u32,

    #[arg(short = 'y')]
    pub height: u32,

    #[arg(short = 'n', num_args = 1..)]
    pub teams: Vec<String>,

    #[arg(short = 'c')]
    pub clients_nb: usize,

    #[arg(short = 'f', default_value_t = 100)]
    pub freq: u32,
}

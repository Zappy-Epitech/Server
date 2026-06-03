use mio::net::TcpStream;

/// Represents the various states a client can be in.
#[derive(Debug, PartialEq)]
pub enum ClientState {
    /// Initial state, waiting for the client to send a team name.
    Authenticating,
    /// Client is an AI (drone) currently in the game.
    InGame,
    /// Client is a graphical interface.
    Graphic,
}

/// Represents a connected client and its network buffers.
pub struct Client {
    /// The non-blocking TCP stream associated with the client.
    pub stream: TcpStream,
    /// Input buffer for accumulating raw bytes received from the network.
    pub buffer_in: Vec<u8>,
    /// Output buffer for storing raw bytes to be sent to the network.
    pub buffer_out: Vec<u8>,
    /// Current state of the client in the authentication/game process.
    pub state: ClientState,
    /// The name of the team the client belongs to (None for GRAPHIC or during auth).
    pub team_name: Option<String>,
}

impl Client {
    /// Creates a new Client instance with the given TCP stream.
    /// 
    /// Initial state is `Authenticating`.
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            buffer_in: Vec::new(),
            buffer_out: Vec::new(),
            state: ClientState::Authenticating,
            team_name: None,
        }
    }
}

use mio::net::TcpStream;

pub enum ClientState {
    Authenticating,
    InGame,
    Graphic,
}

pub struct Client {
    pub stream: TcpStream,
    pub buffer_in: Vec<u8>,
    pub buffer_out: Vec<u8>,
    pub state: ClientState,
    pub team_name: Option<String>,
}

impl Client {
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

use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;


#[derive(PartialEq, Eq)]
enum State {
    WaitingPlayers,
}
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
enum Receiver {
    All,
    AllBut,
    Only,
}

struct Player {
    address: std::net::SocketAddr,
    name: String,
    money: u32,
}

impl Player {
    pub fn new(address: std::net::SocketAddr, name: String) -> Self {
        Player {
            address,
            name,
            money: 100,
        }
    }
}

struct Game {
    players: Vec<Player>,
    state: State,
    test: i32,
}

impl Game {
    fn new() -> Self {
        Game {
            players: Vec::new(),
            state: State::WaitingPlayers,
            test: 0,
        }
    }

    fn get_list(&self)->String {
        let mut s = String::from("table:");
        for p in &self.players {
            s.push_str(&p.name);
            s.push('|');
        }
        s.push('\n');
        s
    }
}

#[tokio::main]
async fn main() {
    // Bind the listener to the address
    let listener = TcpListener::bind("127.0.0.1:7777").await.unwrap();
    let (tx, mut _rx) = broadcast::channel(16);
    let game = Arc::new(Mutex::new(Game::new()));
    loop {
        // The second item contains the IP and port of the new connection.
        let (mut socket, addr) = listener.accept().await.unwrap();
        let tx = tx.clone();
        let mut rx = tx.subscribe();
        let game = Arc::clone(&game);
        tokio::spawn(async move {
            println!("spawning connection...");
            let (reader, mut writer) = socket.split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        if result.unwrap() == 0 {
                            print!("client disconnected");
                            tx.send(("bye".to_string(),addr,Receiver::AllBut)).unwrap();
                            break;
                        }
                        let (cmd, op) = match line.find(':') {
                            None => continue,
                            Some(idx) => (&line[..idx], line[idx + 1..].trim()),
                        };
                        match cmd {
                            "join"=>{
                                println!("process join {}",op);
                                let mut tgame = game.lock().unwrap();
                                if tgame.players.len()<8 && tgame.state == State::WaitingPlayers {
                                    let p = Player::new(addr,op.to_string());
                                    tgame.players.push(p);
                                    tx.send((tgame.get_list(),addr,Receiver::Only)).unwrap();
                                    tx.send((format!("join:{op}\n").to_string(),addr,Receiver::AllBut)).unwrap();
                                } else {
                                    tx.send(("full\n".to_string(),addr,Receiver::Only)).unwrap();
                                }
                            },
                            "start"=>{
                                println!("process start {}",op);
                            }
                            _=>{
                                println!("invalid command");
                            }
                        }
                        //tx.send((line.clone(),addr)).unwrap();
                    }
                    result = rx.recv()=>{
                        let (msg,sender,mode) = result.unwrap();

                        if (addr!=sender && mode==Receiver::AllBut) || (addr==sender && mode==Receiver::Only ) {
                            writer.write_all(msg.as_bytes()).await.unwrap();
                        }
                    }

                }
            }
        });
    }
}
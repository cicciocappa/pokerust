// altra logica:
// enter: connessione al server
// sit: si siede al tavolo

use pokerust::poker::{prepare, Command, NewPlayerInfo, Operation, Player};

use std::collections::HashMap;
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

// struct Player {
//     address: std::net::SocketAddr,
//     name: String,
//     money: u32,
// }

// impl Player {
//     pub fn new(address: std::net::SocketAddr, name: String) -> Self {
//         Player {
//             address,
//             name,
//             money: 100,
//         }
//     }
// }

struct Game {
    players: Vec<Option<Player>>,
    state: State,
    names: HashMap<std::net::SocketAddr, String>,
}

impl Game {
    fn new() -> Self {
        Game {
            players: vec![None, None, None, None, None, None, None, None],
            state: State::WaitingPlayers,
            names: HashMap::new(),
        }
    }
}

#[tokio::main]
async fn main() {
    println!("Starting server...");
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
            let peer = socket.peer_addr().unwrap();
            println!("spawning connection... {:?}", socket.peer_addr());
            let (reader, mut writer) = socket.split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        if result.unwrap() == 0 {
                            println!("client disconnected");
                            let mut tgame = game.lock().unwrap();
                            let leaving = tgame.names.get(&peer).unwrap();
                            println!("si tratta di {leaving}");
                            let who = tgame.players.iter().position(|m|match m {
                                None=>false,
                                Some(n)=>&n.name==leaving,
                            });
                            println!("{:?}",who);
                            if let Some(pos) = who {
                                println!("era seduto in posizione {pos}");
                                let msg = prepare(Operation::Leave, pos.to_string());
                                tx.send((msg,addr,Receiver::AllBut)).unwrap();
                                tgame.players[pos]=None;
                                

                            } 
                            tgame.names.remove(&peer);
                            break;
                        }
                        println!("{line}");
                        let cmd:Command = serde_json::from_str(&line).unwrap();
                        match cmd.op {
                            Operation::Enter =>{
                                println!("process enter {}",cmd.para);
                                let mut tgame = game.lock().unwrap();
                                let free = tgame.players.iter().any(|x| x.is_none());
                                if free && tgame.state == State::WaitingPlayers {
                                    tgame.names.insert(peer,cmd.para);
                                    let msg = prepare(Operation::List, serde_json::to_string(&tgame.players).unwrap());
                                    tx.send((msg,addr,Receiver::Only)).unwrap();

                                } else {
                                    let msg = prepare(Operation::Full, String::new());
                                    tx.send((msg,addr,Receiver::Only)).unwrap();
                                }
                            },
                            Operation::Sit => {

                                let mut tgame = game.lock().unwrap();
                                println!("si siede {:?} nella posizione {} ",peer,cmd.para);
                                let pos = cmd.para.parse::<usize>().unwrap();
                                println!("il cui nome ?? {}",tgame.names.get(&peer).unwrap());
                                let p = Player::new(addr,tgame.names.get(&peer).unwrap().clone());
                                //let cmd = Command::new(Operation::Join, serde_json::to_string(&p).unwrap());
                                //let mut msg = serde_json::to_string(&cmd).unwrap();
                                //msg.push('\n');
                                let info = NewPlayerInfo {
                                    position:pos,
                                    name: tgame.names.get(&peer).unwrap().clone(),

                                };
                                let msg = prepare(Operation::Sit, serde_json::to_string(&info).unwrap());
                                tx.send((msg,addr,Receiver::All)).unwrap();
                                tgame.players[pos]=Some(p);
                                //let cmd = Command::new(Operation::List,serde_json::to_string(&tgame.players).unwrap());
                                //let mut msg = serde_json::to_string(&cmd).unwrap();
                                //msg.push('\n');


                            }
                            Operation::Start =>{
                                println!("process start {}",cmd.para);
                            }
                            _=>{
                                println!("invalid command");
                            }
                        }
                        //tx.send((line.clone(),addr)).unwrap();
                    }
                    result = rx.recv()=>{
                        let (msg,sender,mode) = result.unwrap();
                        println!("scrivo {}",msg);
                        if (addr!=sender && mode==Receiver::AllBut) || (addr==sender && mode==Receiver::Only ) || mode==Receiver::All {
                            writer.write_all(msg.as_bytes()).await.unwrap();
                        }
                    }

                }
            }
        });
    }
}

// altra logica:
// enter: connessione al server
// sit: si siede al tavolo


use pokerust::poker::{Player, Command, Operation};

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
   
}

impl Game {
    fn new() -> Self {
        Game {
            players: vec![None,None,None,None,None,None,None,None],
            state: State::WaitingPlayers,
           
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
            println!("spawning connection...");
            let (reader, mut writer) = socket.split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        if result.unwrap() == 0 {
                            print!("client disconnected");
                            let mut tgame = game.lock().unwrap();
                            tgame.players.remove(0);
                            tx.send(("bye".to_string(),addr,Receiver::AllBut)).unwrap();
                            break;
                        }
                        /*
                        let (cmd, op) = match line.find(':') {
                            None => continue,
                            Some(idx) => (&line[..idx], line[idx + 1..].trim()),
                        };
                        */
                        let cmd:Command = serde_json::from_str(&line).unwrap();
                        match cmd.op {
                            Operation::Enter =>{
                                println!("process enter {}",cmd.para);
                                let tgame = game.lock().unwrap();
                                if tgame.players.len()<8 && tgame.state == State::WaitingPlayers {
                                    let msg = prepare(Operation::List, serde_json::to_string(&tgame.players).unwrap());
                                    tx.send((msg,addr,Receiver::Only)).unwrap();

                                    
                                } else {
                                    tx.send(("full\n".to_string(),addr,Receiver::Only)).unwrap();
                                }
                            },
                            Operation::Sit => {
                                let mut tgame = game.lock().unwrap();
                                let p = Player::new(addr,cmd.para);
                                //let cmd = Command::new(Operation::Join, serde_json::to_string(&p).unwrap());
                                //let mut msg = serde_json::to_string(&cmd).unwrap();
                                //msg.push('\n');
                                let msg = prepare(Operation::Sit, serde_json::to_string(&p).unwrap());
                                tx.send((msg,addr,Receiver::AllBut)).unwrap();
                                let pos = 0;
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

                        if (addr!=sender && mode==Receiver::AllBut) || (addr==sender && mode==Receiver::Only ) {
                            writer.write_all(msg.as_bytes()).await.unwrap();
                        }
                    }

                }
            }
        });
    }
}

fn prepare(op:Operation, para:String)->String {
    let cmd = Command::new(op,para);
    let mut msg = serde_json::to_string(&cmd).unwrap();
    msg.push('\n');
    msg
}
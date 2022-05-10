use pokerust::poker::{prepare, Command, NewPlayerInfo, Operation, Player};

use fltk::{
    app,
    button::Button,
    enums::{Color, FrameType},
    frame::Frame,
    group::Group,
    image::SvgImage,
    input,
    prelude::*,
    window::Window,
};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};

use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Copy, Clone)]
enum Message {
    OpenJoinDialog,
    CloseJoinDialog,
    TryEnter,
    //EnterOk,
    Sit(usize),
    Leave(usize),
    GotList,
    NewPlayer,
}

enum State {
    Disconnected,
    Waiting,
    Playing,
}

struct GameInfo {
    players: Vec<Option<Player>>,
    self_position: usize,
    new_player: NewPlayerInfo,
}

struct PokerClient {
    app: app::App,
    main_win: Window,
    player_labels: Vec<Group>,
    join_win: Window,
    join_btn: Button,
    receiver: app::Receiver<Message>,
    sender: app::Sender<Message>,
    state: State,
    server_input: input::Input,
    name_input: input::Input,
    join_ok: Button,
    join_info: Frame,
    stream: Option<TcpStream>,
}

impl PokerClient {
    fn new() -> Self {
        const POSIZIONI: [(i32, i32); 8] = [
            (570, 710),
            (1000, 700),
            (1140, 360),
            (1000, 20),
            (570, 10),
            (140, 20),
            (0, 360),
            (140, 700),
        ];
        let app = app::App::default();
        app::set_visible_focus(false);
        let (s, receiver) = app::channel();
        let mut player_labels = Vec::new();
        let mut main_win = Window::default().with_size(1600, 800).with_label("Poker");
        let mut frame = Frame::default().with_size(1280, 768).with_pos(0, 0);
        let mut table = SvgImage::load("assets/table.svg").unwrap();
        table.scale(1280, 768, true, true);
        frame.set_image(Some(table));
        let mut join_btn = Button::default()
            .with_label("JOIN")
            .with_size(80, 80)
            .center_of(&main_win);
        join_btn.set_frame(FrameType::PlasticRoundDownBox);
        for i in 0..8 {
            let mut gp0 = Group::new(POSIZIONI[i].0, POSIZIONI[i].1, 140, 48, "");
            let mut lp0 = Frame::new(POSIZIONI[i].0, POSIZIONI[i].1 + 2, 140, 24, "");
            //lp0.set_frame(FrameType::FlatBox);
            lp0.set_label_color(Color::Cyan);
            //lp0.set_color(Color::Black);
            let mut lp1 = Frame::new(POSIZIONI[i].0, POSIZIONI[i].1 + 22, 140, 24, "");
            lp1.set_label_color(Color::Yellow);
            let mut bt0 = Button::new(POSIZIONI[i].0 + 50, POSIZIONI[i].1 + 12, 40, 24, "SIT");
            bt0.set_callback(move |_| s.send(Message::Sit(i)));
            bt0.hide();
            gp0.end();
            gp0.set_frame(FrameType::EmbossedBox);
            gp0.set_color(Color::Black);
            player_labels.push(gp0);
        }
        main_win.end();
        main_win.show();

        let mut join_win = Window::default()
            .with_size(300, 140)
            .with_label("Join server");
        let _f1 = Frame::default()
            .with_label("Host:")
            .with_size(100, 24)
            .with_pos(10, 30);
        let _f2 = Frame::default()
            .with_label("Name:")
            .with_size(100, 24)
            .with_pos(10, 60);
        let mut join_info = Frame::default()
            //.with_label("Enter server and name")
            .with_size(300, 24)
            .with_pos(0, 0);
        join_info.set_frame(FrameType::FlatBox);
        join_info.set_label_color(Color::White);
        //join_info.set_color(Color::Blue);
        let mut name_input = input::Input::default().with_size(120, 24).with_pos(120, 60);
        name_input.set_maximum_size(14);
        let mut server_input = input::Input::default().with_size(120, 24).with_pos(120, 30);
        server_input.set_maximum_size(128);
        let mut join_ok = Button::default()
            .with_size(80, 24)
            .with_label("Ok")
            .with_pos(70, 100);
        let mut join_cancel = Button::default()
            .with_size(80, 24)
            .with_label("Cancel")
            .with_pos(160, 100);
        join_win.end();
        join_win.make_modal(true);

        join_btn.emit(s, Message::OpenJoinDialog);
        join_ok.emit(s, Message::TryEnter);
        join_cancel.emit(s, Message::CloseJoinDialog);

        PokerClient {
            app,
            state: State::Disconnected,
            main_win,
            player_labels,
            join_win,
            join_btn,
            receiver,
            sender: s,
            server_input,
            name_input,
            join_ok,
            join_info,
            stream: None,
        }
    }

    pub fn run(mut self) {
        let game_info = GameInfo {
            players: Vec::new(),
            self_position: 0,
            new_player: NewPlayerInfo {
                position: 0,
                name: String::new(),
            },
        };
        let game_info = Arc::new(Mutex::new(game_info));
        while self.app.wait() {
            if let Some(msg) = self.receiver.recv() {
                match msg {
                    Message::GotList => {
                        let game_info = game_info.lock().unwrap();
                        println!("got players list lunga {}", game_info.players.len());
                        //println!("io sono in posizione {}", game_info.self_position);
                        for i in 0..8 {
                            match &game_info.players[i] {
                                Some(p) => {
                                    self.player_labels[i].child(0).unwrap().set_label(&p.name);
                                    let mut s = p.money.to_string();
                                    s.push('$');
                                    self.player_labels[i].child(1).unwrap().set_label(&s);
                                }
                                None => {
                                    self.player_labels[i].child(2).unwrap().show();
                                }
                            };
                        }
                    }
                    

                    Message::Sit(p) => {
                        println!("giocatore prova a sedersi in posizione {p}");
                        for i in 0..8 {
                            self.player_labels[i].child(2).unwrap().hide();
                        }
                        let msg = prepare(Operation::Sit, p.to_string());
                        //match self.stream {
                        //    Some(ref mut s) => {s.write(&msg.into_bytes()).unwrap();}
                        //    None => {}
                        // }
                        //self.stream.as_mut().write(&msg.into_bytes()).unwrap();
                        self.stream
                            .as_mut()
                            .unwrap()
                            .write(&msg.into_bytes())
                            .unwrap();
                    }

                    Message::Leave(p) => {
                        self.player_labels[p].child(0).unwrap().set_label("");
                        self.player_labels[p].child(1).unwrap().set_label("");
                    }

                    Message::NewPlayer => {
                        let tgame = game_info.lock().unwrap();
                        let p = tgame.new_player.position;
                        self.player_labels[p].child(2).unwrap().hide();
                        self.player_labels[p]
                            .child(0)
                            .unwrap()
                            .set_label(&tgame.new_player.name);
                        self.player_labels[p].child(1).unwrap().set_label("100$");
                        // aggiorniamo anche l'elenco locale di giocatori?
                    }

                    Message::OpenJoinDialog => {
                        self.join_info.set_label("Enter host and player name");
                        self.join_info.set_color(Color::Blue);
                        self.join_win.show();
                    }
                    Message::CloseJoinDialog => {
                        self.join_win.hide();
                    }
                    Message::TryEnter => {
                        self.join_ok.deactivate();
                        let server = if self.server_input.value().len() > 2 {
                            self.server_input.value()
                        } else {
                            "127.0.0.1".to_string()
                        };
                        let name = if self.name_input.value().len() > 1 {
                            self.name_input.value()
                        } else {
                            "Player".to_string()
                        };
                        let address = format!("{server}:7777").to_socket_addrs();
                        if address.is_ok() {
                            let mut address = address.unwrap();
                            let mut stream = TcpStream::connect_timeout(
                                &address.next().unwrap(),
                                std::time::Duration::from_secs(1),
                            );
                            if stream.is_ok() {
                                let mut stream = stream.unwrap();
                                //self.join_info.set_label("Connessione ok");
                                let msg = prepare(Operation::Enter, name);
                                //let cmd = Command::new(Operation::Enter, name);
                                //let mut msg = serde_json::to_string(&cmd).unwrap();
                                //msg.push('\n');
                                stream.write(&msg.into_bytes()).unwrap();
                                let rstream = stream.try_clone().unwrap();
                                let game_info = Arc::clone(&game_info);
                                thread::spawn(move || reader(self.sender, rstream, game_info));
                                self.stream = Some(stream);
                                self.join_win.hide();
                                self.join_btn.hide();
                            } else {
                                self.join_info.set_label("Can't connect to host");
                                self.join_info.set_color(Color::Red);
                                self.join_ok.activate();
                            }
                        } else {
                            self.join_info.set_label("Host unknow");
                            self.join_info.set_color(Color::Red);
                            self.join_ok.activate();
                        }

                        // controllare la lunghezza massima di name;
                    }
                    //Message::EnterOk => {
                    //    println!("join ok");
                    //}
                }
            }
        }
    }
}

fn main() {
    let a = PokerClient::new();
    a.run();
}

fn reader(s: app::Sender<Message>, mut reader: TcpStream, game_info: Arc<Mutex<GameInfo>>) {
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    loop {
        let len = reader.read_line(&mut line).unwrap();
        if len == 0 {
            println!("disconnesso");
            break;
        } else {
            println!("from server: {}", line);
            // process line
            //let (cmd, op) = match line.find(':') {
            //    None => continue,
            //    Some(idx) => (&line[..idx], line[idx + 1..].trim()),
            //};

            let cmd: Command = serde_json::from_str(&line).unwrap();

            match cmd.op {
                Operation::Sit => {
                    let mut tgame = game_info.lock().unwrap();
                    let p: NewPlayerInfo = serde_json::from_str(&cmd.para).unwrap();
                    println!("aggiungo {} in pos {}", p.name, p.position);
                    tgame.new_player = p;
                    s.send(Message::NewPlayer);
                }
                Operation::List => {
                    let presenti: Vec<Option<Player>> = serde_json::from_str(&cmd.para).unwrap();
                    let mut tgame = game_info.lock().unwrap();
                    tgame.self_position = presenti.len() - 1;
                    tgame.players = presenti;
                    println!("tavolo: {:?}", tgame.players.len());
                    s.send(Message::GotList);
                }
                Operation::Leave => {
                    let pos = cmd.para.parse::<usize>().unwrap();
                    println!("processo addio {}",cmd.para);
                    s.send(Message::Leave(pos));
                    // probabilmente nel client non mi serve aggiornare la lista giocatori, mi basta cancellare le info
                    //let mut tgame = game_info.lock().unwrap();

                }
                _ => (),
            }
            line.clear();
        }
    }
}

use fltk::{
    app,
    button::Button,
    enums::{Color, FrameType},
    frame::Frame,
    image::SvgImage,
    input,
    prelude::*,
    window::Window,
};
use std::io::{BufReader, BufRead, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::str::from_utf8;
use std::thread;

#[derive(Copy, Clone)]
enum Message {
    OpenJoinDialog,
    CloseJoinDialog,
    TryJoin,
    JoinOk,
    CommandJoin,
}

enum State {
    Disconnected,
    Waiting,
    Playing,
}

struct PlayerInfo {
    player_pos: usize,
    player_name: String,
    cash: u32,
}

struct PokerClient {
    app: app::App,
    main_win: Window,
    player_labels: Vec<Frame>,
    join_win: Window,
    join_btn: Button,
    receiver: app::Receiver<Message>,
    sender: app::Sender<Message>,
    state: State,
    server_input: input::Input,
    name_input: input::Input,
    join_ok: Button,
    join_info: Frame,
    player_info: Option<PlayerInfo>,
    stream: Option<TcpStream>,
}

impl PokerClient {
    fn new() -> Self {
        let app = app::App::default();
        app::set_visible_focus(false);
        let (s, receiver) = app::channel();
         
        let mut main_win = Window::default().with_size(1280, 768).with_label("Poker");
        let mut frame = Frame::default().with_size(1280, 768).with_pos(0, 0);
        let mut table = SvgImage::load("assets/table.svg").unwrap();
        table.scale(1280, 768, true, true);
        frame.set_image(Some(table));
        let mut join_btn = Button::default()
            .with_label("JOIN")
            .with_size(80, 80)
            .center_of(&main_win);
        join_btn.set_frame(FrameType::PlasticRoundDownBox);
        let mut lp0 = Frame::new(100, 100, 120, 24, "WMWMWMWMWMWMWM0");
        lp0.set_frame(FrameType::FlatBox);
        lp0.set_color(Color::Yellow);
        let mut lp1 = Frame::new(1180, 100, 80, 24, "empty");
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
        join_ok.emit(s, Message::TryJoin);
        join_cancel.emit(s, Message::CloseJoinDialog);

        let player_labels=vec![lp0,lp1];

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
            player_info: None,
            stream: None,
        }
    }

    pub fn run(mut self) {
        while self.app.wait() {
            if let Some(msg) = self.receiver.recv() {
                match msg {
                    Message::CommandJoin => {
                        println!("join nuovo player");
                    }

                    Message::OpenJoinDialog => {
                        self.join_info.set_label("Enter host and player name");
                        self.join_info.set_color(Color::Blue);
                        self.join_win.show();
                    }
                    Message::CloseJoinDialog => {
                        self.join_win.hide();
                    }
                    Message::TryJoin => {
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
                                let msg = format!("join:{}\r\n",name);
                                stream.write(&msg.into_bytes()).unwrap();
                                let rstream = stream.try_clone().unwrap();
                                thread::spawn(move || reader(self.sender, rstream));
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
                    Message::JoinOk => {
                        println!("join ok");
                    }
                }
            }
        }
    }
}

fn main() {
    let a = PokerClient::new();
    a.run();
}

fn reader(s: app::Sender<Message>, mut reader: TcpStream) {
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    loop {

        let len = reader.read_line(&mut line).unwrap();
        if len==0 {
            println!("disconnesso");
            break;

        } else {
            println!("got: {}", line);
            // process line
            let (cmd, op) = match line.find(':') {
                None => continue,
                Some(idx) => (&line[..idx], line[idx + 1..].trim()),
            };

            match cmd {
                "join"=>{
                    println!("aggiungo {}",op);
                    s.send(Message::CommandJoin);

                },
                "table"=>{
                    let mut presenti:Vec<&str> = op.split('|').collect();
                    presenti.pop();
                    println!("tavolo: {:?}",presenti);
                }
                _=>()
            }
            line.clear();
        }

       
    }
}
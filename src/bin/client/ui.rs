use super::{State,PokerClient,Message};
use fltk::{
    
    app::{self,Screen},
    button::Button,
    enums::{Color, FrameType},
    frame::Frame,
    group::Group,
    image::SvgImage,
    input,
    text,
    prelude::*,
    window::Window,
};

pub fn setup() -> PokerClient {
    let wa = Screen::all_screens()[0].set_scale(1.0);
    const POSIZIONI: [(i32, i32); 8] = [
        (570+16, 710+16),
        (1000+16, 700+16),
        (1140+16, 360+16),
        (1000+16, 20+16),
        (570+16, 10+16),
        (140+16, 20+16),
        (0+16, 360+16),
        (140+16, 700+16),
    ];
    let app = app::App::default();
    app::set_visible_focus(false);
    let (s, receiver) = app::channel();
    let mut player_labels = Vec::new();
    let mut main_win = Window::default().with_size(1600, 800).with_label("Poker");
    main_win.set_color(Color::Black);
    let mut frame = Frame::default().with_size(1280, 768).with_pos(16, 16);
    let mut table = SvgImage::load("assets/table.svg").unwrap();
    let logo = SvgImage::load("assets/logo.svg").unwrap();
 
    table.scale(1280, 768, true, true);
    frame.set_image(Some(table));
    let mut join_btn = Button::default()
        .with_label("JOIN")
        .with_size(80, 80)
        .center_of(&frame);
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
        gp0.set_color(Color::by_index(36));
        player_labels.push(gp0);
    }
    let mut title = Frame::default().with_size(280,48).with_pos(1304,16);
    title.set_image(Some(logo));
    let mut txt = text::TextDisplay::default().with_size(280, 512).with_pos(1304,80);
    let mut buf = text::TextBuffer::default();
    txt.set_buffer(buf.clone());

    main_win.end();
    main_win.show();
    buf.set_text("Hello!");
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
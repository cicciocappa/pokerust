use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fmt;

use Seed::*;

struct Player {
    name: String,
    money: u32,
    connected: bool,
}

struct Table {
    players: Vec<Player>,
    pot: u32,
    dealer: usize,
    hands: u32,
    state: State,
}

#[derive(Debug, Copy, Clone)]
enum Seed {
    Hearts,
    Spades,
    Clubs,
    Diamonds,
}

enum State {
    Waiting,
    Deal,
    Bet,
    Flop,
    Turn,
    River,
    PostHand,
}

#[derive(Debug, Copy, Clone)]
pub struct Card {
    seed: Seed,
    value: u8,
}

impl Card {
    fn new() -> Self {
        Self {
            value: 0,
            seed: Hearts,
        }
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let face = match self.value {
            0..=8 => (self.value + 2).to_string(),
            9 => "J".to_string(),
            10 => "K".to_string(),
            11 => "Q".to_string(),
            12 => "A".to_string(),
            _ => "?".to_string(),
        };
        let seed = match self.seed {
            Hearts => '\u{2665}',
            Diamonds => '\u{2666}',
            Clubs => '\u{2663}',
            Spades => '\u{2660}',
        };
        write!(f, "[{} {}]", face, seed)
    }
}

#[derive(Debug)]
pub struct Deck {
    cards: [Card; 52],
    position: usize,
}

impl Deck {
    pub fn new() -> Self {
        let mut cards = [Card::new(); 52];
        for i in 0..52 {
            cards[i] = Card {
                value: (i % 13) as u8,
                seed: match i {
                    0..=12 => Hearts,
                    13..=25 => Clubs,
                    26..=38 => Diamonds,
                    _ => Spades,
                },
            }
        }
        Self { cards, position: 0 }
    }
    pub fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.cards.shuffle(&mut rng);
        self.position = 0;
    }
}

impl Iterator for Deck {
    type Item = Card;
    fn next(&mut self) -> Option<Card> {
        if self.position > 51 {
            return None;
        }
        let result = self.cards[self.position];
        self.position += 1;
        Some(result)
    }
}
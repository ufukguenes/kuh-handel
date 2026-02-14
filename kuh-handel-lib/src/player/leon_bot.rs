use std::collections::HashMap;

use crate::{
    Value,
    animals::Animal,
    messages::{
        actions::{
            AuctionDecision, Bidding, InitialTrade, NoAction, PlayerTurnDecision, SendMoney,
            TradeOpponentDecision,
        },
        game_updates::{AuctionRound, GameUpdate, TradeOffer},
    },
    player::{base_player::PlayerId, player_actions::PlayerActions, wallet::Wallet},
};

struct BotPlayer {
    card_count: u16,
    estimated_wallet: Wallet,
    owned_animals: HashMap<Animal, usize>,
    real_points: u16,
    estimated_points: u16,
}

struct LeonBot {
    // add other necessary things that have to be stored
    opponents: HashMap<PlayerId, BotPlayer>,
}

impl LeonBot {
    pub fn new() -> Self {
        Self {
            opponents: HashMap::default(),
        }
    }

    pub fn add_player(&mut self, id: PlayerId, initial_wallet: Wallet) {
        if self
            .opponents
            .insert(
                id,
                BotPlayer {
                    card_count: 0,
                    estimated_wallet: initial_wallet,
                    owned_animals: HashMap::default(),
                    real_points: 0,
                    estimated_points: 0,
                },
            )
            .is_some()
        {
            panic!("two players with the same name detected");
        }
    }
}

impl PlayerActions for LeonBot {
    fn _draw_or_trade(&mut self) -> PlayerTurnDecision {
        // depending on the state of the current game, which player own whihc animal
        // decide whether to use trade against a player or to draw a new card
        todo!()
    }

    fn _trade(&mut self) -> InitialTrade {
        // using the same logic as in _draw_or_trade to devide which trade can be done next
        todo!()
    }

    fn _provide_bidding(&mut self, state: AuctionRound) -> Bidding {
        // easily define a metric until which point I want to bid
        // also think about stealing an animal such that another player will not get the animal
        // simple idea: calculate the possible points per player (metric that is based on the points
        // that would be currently possible, when having full sets, but then kind of reduce the
        // points per set by number of animals that are currently available)
        // on each bid calculate that number, or simply store it, and calculate on change
        // but then always calculate a percentage that is then used as probability on whether to bid
        // or not, and then bid if the maximum is not already reached -> but also consider the
        // number of cards in this calculation
        let animal = state.animal;
        let host = state.host;
        let bids = state.bids;
        todo!()
    }

    fn _buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision {
        // check animal, who owns it and depending on whether it is affordable, decide to either buy
        // or sell
        let animal = state.animal;
        let host = state.host;
        let bids = state.bids;
        todo!()
    }
    fn _send_money_to_player(&mut self, player: &PlayerId, amount: Value) -> SendMoney {
        // calculate minimal amount of cards that are enough to pay
        // but consider to use the next higher card, if that would lead to a loss of too many other
        // card that i own in the current moment
        todo!()
    }

    fn _respond_to_trade(&mut self, offer: TradeOffer) -> TradeOpponentDecision {
        // here also weight whether it is nice to accept the trade or not
        // if not maybe return
        let challenger = offer.challenger;
        let animal = offer.animal;
        let animal_count = offer.animal_count;
        let card_offer = offer.challenger_card_offer;

        todo!()
    }

    fn _receive_game_update(&mut self, update: GameUpdate) -> NoAction {
        // easily add the information to my own state representation
        match update {
            GameUpdate::Auction(auction_kind) => (),
            GameUpdate::Trade {
                challenger,
                opponent,
                animal,
                animal_count,
                receiver,
                money_trade,
            } => (),
            GameUpdate::Start {
                wallet,
                players_in_turn_order,
                animals,
            } => (),
            GameUpdate::End { ranking } => (),
            GameUpdate::ExposePlayer { player, wallet } => (),
            GameUpdate::Inflation(_) => (),
        }
        todo!()
    }
}

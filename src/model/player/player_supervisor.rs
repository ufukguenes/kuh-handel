use rand::seq::IndexedRandom;

use crate::messages::actions::*;
use crate::messages::game_updates::*;
use crate::model::animals::Animal;
use crate::model::money::money::Money;
use crate::model::money::wallet::Wallet;
use crate::model::player;
use crate::model::player::{
    base_player::Player, player_actions::base_player_actions::PlayerActions,
};

pub struct PlayerSupervisor {
    player: Player,
}

/// todo, what to do when invalid decision?
///  maybe notify the bot and just pick a random valid action
/// 
/// 

impl PlayerSupervisor {
    pub fn can_afford(wallet: Wallet, payment_amount: Vec<Money>) {
        wallet.
    }
}

impl PlayerActions for PlayerSupervisor {
    fn _draw_or_trade(&mut self) -> PlayerTurnDecision {
        let decision = self.player.player_actions()._draw_or_trade();
        match decision {
            PlayerTurnDecision::Draw => decision,
            PlayerTurnDecision::Trade(initial_trade) => {
                let num_trade_animal = self
                    .player
                    .owned_animals()
                    .iter()
                    .filter(|animal| animal.value() == initial_trade.animal.value)
                    .count();
                let has_enough_animals = num_trade_animal >= initial_trade.animal_count as usize;
                let opponent_not_self = *self.player.id() != initial_trade.opponent;
                let has_enough_money = self.player.wallet().can_afford(initial_trade.amount);
                match has_enough_money {
                    (true, None) => todo!(),
                    (true, Some(_)) => todo!(),
                    (false, None) => todo()!,
                    (false, Some(_)) => panic!("This is can not happen"),
                }


                decision
            }
        }
    }

    fn _trade(&mut self) -> InitialTrade {
        self.player.player_actions()._trade()
    }

    fn _provide_bidding(&mut self, state: AuctionRound) -> Bidding {
        self.player.player_actions()._provide_bidding(state)
    }

    fn _buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision {
        self.player.player_actions()._buy_or_sell(state)
    }

    fn _send_money_to_player(
        &mut self,
        player: &super::base_player::PlayerId,
        amount: crate::model::money::value::Value,
    ) -> SendMoney {
        self.player
            .player_actions()
            ._send_money_to_player(player, amount)
    }

    fn _receive_from_player(
        &mut self,
        player: &super::base_player::PlayerId,
        money: Vec<crate::model::money::money::Money>,
    ) -> NoAction {
        self.player
            .player_actions()
            ._receive_from_player(player, money)
    }

    fn _respond_to_trade(&mut self, offer: TradeOffer) -> TradeOpponentDecision {
        self.player.player_actions()._respond_to_trade(offer)
    }

    fn _receive_game_update(&mut self, update: GameUpdate) -> NoAction {
        self.player.player_actions()._receive_game_update(update)
    }
}

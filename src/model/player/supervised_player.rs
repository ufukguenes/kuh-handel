use std::rc::Rc;

use crate::messages::actions::*;
use crate::messages::game_updates::*;
use crate::model::money::money::Money;
use crate::model::money::value::Value;
use crate::model::money::wallet::Affordability::*;
use crate::model::player::{
    base_player::{Player, PlayerId},
    player_actions::base_player_actions::PlayerActions,
};

/// This changes an action based on the deepest nested thing that breaks the action
/// example:
/// 1. player is called to do draw_or_trade
/// 2. if player wants to trade, player picks it opponent, animals to trade, money to trade
/// 3. if for example the opponent exists but it doesn't have the animals,
/// the alternative would not be to switch from trade to draw, but to find a fitting alternative trade
pub struct SupervisedPlayer {
    player: Player,
    opponents: Rc<Vec<Player>>,
}

/// todo, what to do when invalid decision?
///  maybe notify the bot and just pick a random valid action
///
/// todo: should the player actions be not be &mut self, but only &self, except for game update,
/// as you might for example only withdraw the amount of money that you actually payed, and not if you only thought you payed?

impl SupervisedPlayer {
    fn rectify_money_combination(&self, combination: &Vec<Money>) -> Vec<Money> {
        match self.player.wallet().can_afford(combination) {
            Exact => combination.clone(),
            Alternative(alternative) => alternative,
            CannotAfford => self.player.wallet().to_vec(),
        }
    }

    fn rectify_initial_trade(&self, trade: &InitialTrade) -> InitialTrade {
        let trade_animal = trade.animal;
        let animal_count: usize = trade.animal_count.clone() as usize;

        let self_has_enough_animals = match self.player.owned_animals().get(&trade_animal) {
            Some(&count) => count >= animal_count,
            None => false,
        };

        if !self_has_enough_animals {
            todo!()
        }

        let opponent = self
            .opponents
            .iter()
            .find(|player| *player.id() == trade.opponent);

        match opponent {
            Some(opponent) => {
                let opponent_has_enough_animals = match opponent.owned_animals().get(&trade_animal)
                {
                    Some(&count) => count >= animal_count,
                    None => false,
                };
                if !opponent_has_enough_animals {
                    todo!()
                }
            }
            None => todo!(),
        }

        let mut new_trade = trade.clone();
        let new_combination = self.rectify_money_combination(&trade.amount);
        new_trade.amount = new_combination;

        new_trade
    }

    fn rectify_payment(&self, send_money: &SendMoney) -> SendMoney {
        match send_money {
            SendMoney::WasBluff => return send_money.clone(),
            SendMoney::Amount(amount) => {
                let has_enough_money = self.player.wallet().can_afford(amount);
                match has_enough_money {
                    Exact => return send_money.clone(),
                    Alternative(alternative_payment) => {
                        return SendMoney::Amount(alternative_payment);
                    }

                    CannotAfford => return SendMoney::WasBluff,
                }
            }
        }
    }
}

impl PlayerActions for SupervisedPlayer {
    fn _draw_or_trade(&mut self) -> PlayerTurnDecision {
        let decision: PlayerTurnDecision = self.player.player_actions()._draw_or_trade();
        match decision {
            PlayerTurnDecision::Draw => decision,
            PlayerTurnDecision::Trade(initial_trade) => {
                PlayerTurnDecision::Trade(self.rectify_initial_trade(&initial_trade))
            }
        }

        //todo check if the player even has that money to make the hidden offer
    }

    fn _trade(&mut self) -> InitialTrade {
        let decision: InitialTrade = self.player.player_actions()._trade();
        self.rectify_initial_trade(&decision)
    }

    fn _provide_bidding(&mut self, state: AuctionRound) -> Bidding {
        self.player.player_actions()._provide_bidding(state)
    }

    fn _buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision {
        self.player.player_actions()._buy_or_sell(state)
    }

    fn _send_money_to_player(&mut self, player: &PlayerId, amount: Value) -> SendMoney {
        let decision = self
            .player
            .player_actions()
            ._send_money_to_player(player, amount);
        self.rectify_payment(&decision)
    }

    fn _respond_to_trade(&mut self, offer: TradeOffer) -> TradeOpponentDecision {
        let decision = self.player.player_actions()._respond_to_trade(offer);
        match decision {
            TradeOpponentDecision::Accept => decision,
            TradeOpponentDecision::CounterOffer(amount) => {
                TradeOpponentDecision::CounterOffer(self.rectify_money_combination(&amount))
            }
        }
    }

    fn _receive_game_update(&mut self, update: GameUpdate) -> NoAction {
        // GameUpdate::Start is handled by the game logic when initializing a new player, because then the opponents can be Rc
        match update.clone() {
            GameUpdate::Auction {
                rounds,
                from,
                to,
                money_transfer,
            } => match money_transfer {
                // check if what animal, not necessary to check if host, because is checked with from to
                MoneyTransfer::Private { amount } => {
                    if self.player.id() == &from {
                        self.player.wallet_mut().withdraw(&amount);
                    } else if self.player.id() == &to {
                        self.player.wallet_mut().withdraw(&amount);
                    }
                }
                _ => {}
            },
            GameUpdate::Trade {
                challenger,
                opponent,
                animal,
                animal_count,
                receiver,
                money_trade,
            } => {
                let animal_count: usize = animal_count.clone() as usize;
                let player_id = self.player.id().clone();
                if (player_id == challenger || player_id == opponent) && player_id == receiver {
                    self.player.add_animals(&animal, animal_count);
                } else if (player_id == challenger || player_id == opponent)
                    && player_id != receiver
                {
                    self.player.remove_animals(&animal, animal_count);
                }
                match money_trade {
                    MoneyTrade::Private {
                        challenger_card_offer,
                        opponent_card_offer,
                    } => {
                        if player_id == challenger {
                            self.player.wallet_mut().withdraw(&challenger_card_offer);
                            opponent_card_offer
                                .map(|amount| self.player.wallet_mut().deposit(&amount));
                        } else {
                            opponent_card_offer
                                .map(|amount| self.player.wallet_mut().withdraw(&amount));
                            self.player.wallet_mut().deposit(&challenger_card_offer);
                        }
                    }
                    _ => {}
                }
            }

            _ => {}
        }

        self.player.player_actions()._receive_game_update(update)
    }
}

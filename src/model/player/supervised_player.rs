use std::cell::Ref;
use std::cell::RefCell;

use std::rc::Rc;

use crate::messages::actions::*;
use crate::messages::game_updates::*;
use crate::messages::message_protocol::StateMessage;
use crate::model::animals::Animal;
use crate::model::money::money::Money;
use crate::model::money::value::Value;
use crate::model::money::wallet::Affordability::*;
use crate::model::money::wallet::Wallet;
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
    pub player: Rc<RefCell<Player>>,
    opponents: Vec<Rc<RefCell<Player>>>,
    limit_bidding_until_next_auction: bool,
}

/// todo, what to do when invalid decision?
///  maybe notify the bot and just pick a random valid action
///
/// todo: should the player actions be not be &mut self, but only &self, except for game update,
/// as you might for example only withdraw the amount of money that you actually payed, and not if you only thought you payed?

impl SupervisedPlayer {
    pub fn new(player: Rc<RefCell<Player>>, opponents: Vec<Rc<RefCell<Player>>>) -> Self {
        SupervisedPlayer {
            player: player,
            opponents: opponents,
            limit_bidding_until_next_auction: false,
        }
    }

    pub fn clone_wallet(&self) -> Wallet {
        self.player.borrow().wallet().clone()
    }

    pub fn id(&self) -> PlayerId {
        self.player.borrow().id().clone()
    }

    pub fn can_trade_against(&self, opponent: Rc<RefCell<Player>>) -> Option<InitialTrade> {
        self.player.borrow().can_trade_against(opponent)
    }
    pub fn can_trade(&self) -> Option<InitialTrade> {
        self.player.borrow().can_trade(&self.opponents)
    }

    fn rectify_money_combination(&self, combination: &Vec<Money>) -> Vec<Money> {
        match self.player.borrow_mut().wallet().can_afford(combination) {
            Exact => combination.clone(),
            Alternative(alternative) => alternative,
            CannotAfford => self.player.borrow_mut().wallet().to_vec(),
        }
    }

    fn rectify_initial_trade(&self, trade: &InitialTrade) -> InitialTrade {
        let new_amount = self.rectify_money_combination(&trade.amount);

        let trade_animal = trade.animal;
        let animal_count: usize = trade.animal_count.clone() as usize;

        let animal_trade_count = match self.player.borrow().owned_animals().get(&trade_animal) {
            Some(&count) => {
                if count >= animal_count {
                    count
                } else if count > 0 {
                    1
                } else {
                    panic!(
                        "animal was in hashmap, but amount was set to zero, this should/ can not happen"
                    )
                }
            }
            None => return self.can_trade().unwrap(), // player does not have animal
        };

        let opponent: Option<&Rc<RefCell<Player>>> = self
            .opponents
            .iter()
            .find(|player| player.borrow().id() == &trade.opponent);

        let opponent = match opponent {
            Some(opponent) => opponent,
            None => return self.can_trade().unwrap(), // opponent does not exist
        };

        let opponent_animal_count = match opponent.borrow().owned_animals().get(&trade_animal) {
            Some(&count) => {
                if count >= animal_count {
                    count
                } else if count > 0 {
                    1
                } else {
                    panic!(
                        "animal was in hashmap, but amount was set to zero, this should/ can not happen"
                    )
                }
            }
            None => return self.can_trade().unwrap(), // opponent does not have animal
        };
        // is never 0

        let mut new_trade = trade.clone();
        new_trade.opponent = opponent.borrow().id().clone();
        new_trade.animal = trade_animal;
        new_trade.amount = new_amount;
        new_trade.animal_count = std::cmp::min(opponent_animal_count, animal_trade_count);
        new_trade
    }

    fn rectify_payment(&self, send_money: &SendMoney) -> SendMoney {
        match send_money {
            SendMoney::WasBluff => return send_money.clone(),
            SendMoney::Amount(amount) => {
                let has_enough_money = self.player.borrow().wallet().can_afford(amount);
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

    pub fn map_to_action_inner<T: FromActionMessage>(&mut self, state_msg: StateMessage) -> T {
        let action_msg = self.map_to_action(state_msg);
        T::extract(action_msg)
    }
}

impl PlayerActions for SupervisedPlayer {
    fn _draw_or_trade(&mut self) -> PlayerTurnDecision {
        let decision: PlayerTurnDecision =
            self.player.borrow_mut().player_actions()._draw_or_trade();
        match decision {
            PlayerTurnDecision::Draw => return decision,
            PlayerTurnDecision::Trade(initial_trade) => {
                if self.can_trade().is_some() {
                    return PlayerTurnDecision::Trade(self.rectify_initial_trade(&initial_trade));
                }
                return PlayerTurnDecision::Draw;
            }
        }
    }

    fn _trade(&mut self) -> InitialTrade {
        let decision: InitialTrade = self.player.borrow_mut().player_actions()._trade();
        self.rectify_initial_trade(&decision)
    }

    fn _provide_bidding(&mut self, state: AuctionRound) -> Bidding {
        let decision = self
            .player
            .borrow_mut()
            .player_actions()
            ._provide_bidding(state);

        if self.limit_bidding_until_next_auction {
            if let Some(limit) = self.player.borrow().wallet().total_money() {
                match decision {
                    Bidding::Pass => {
                        return decision;
                    }
                    Bidding::Bid(value) => {
                        if value > limit {
                            return Bidding::Bid(limit);
                        } else {
                            return decision;
                        };
                    }
                }
            }
            return Bidding::Pass;
        } else {
            return decision;
        }
    }

    fn _buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision {
        self.player
            .borrow_mut()
            .player_actions()
            ._buy_or_sell(state)
    }

    fn _send_money_to_player(&mut self, player: &PlayerId, amount: Value) -> SendMoney {
        let decision = self
            .player
            .borrow_mut()
            .player_actions()
            ._send_money_to_player(player, amount);
        self.rectify_payment(&decision)
    }

    fn _respond_to_trade(&mut self, offer: TradeOffer) -> TradeOpponentDecision {
        let decision = self
            .player
            .borrow_mut()
            .player_actions()
            ._respond_to_trade(offer);
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
            GameUpdate::Auction(auction_kind) => {
                self.limit_bidding_until_next_auction = false;
                match auction_kind {
                    AuctionKind::NoBiddings { host_id, animal } => {
                        if &host_id == self.player.borrow().id() {
                            self.player.borrow_mut().add_animals(&animal, 1);
                        }
                    }
                    AuctionKind::NormalAuction {
                        rounds,
                        from,
                        to,
                        money_transfer,
                    } => {
                        match money_transfer {
                            // check if what animal, not necessary to check if host, because is checked with from to
                            MoneyTransfer::Private { amount } => {
                                let mut player = self.player.borrow_mut();
                                if player.id() == &from {
                                    player.wallet_mut().withdraw(&amount);
                                    player.add_animals(&rounds.animal, 1);
                                } else if player.id() == &to {
                                    player.wallet_mut().deposit(&amount);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            GameUpdate::Trade {
                challenger,
                opponent,
                animal,
                animal_count,
                receiver,
                money_trade,
            } => {
                let animal_count: usize = animal_count.clone();
                let mut player = self.player.borrow_mut();
                let player_id = player.id().clone();
                if (player_id == challenger || player_id == opponent) && player_id == receiver {
                    player.add_animals(&animal, animal_count);
                } else if (player_id == challenger || player_id == opponent)
                    && player_id != receiver
                {
                    player.remove_animals(&animal, animal_count);
                }
                match money_trade {
                    MoneyTrade::Private {
                        challenger_card_offer,
                        opponent_card_offer,
                    } => {
                        if player_id == challenger {
                            player.wallet_mut().withdraw(&challenger_card_offer);
                            opponent_card_offer.map(|amount| player.wallet_mut().deposit(&amount));
                        } else {
                            opponent_card_offer.map(|amount| player.wallet_mut().withdraw(&amount));
                            player.wallet_mut().deposit(&challenger_card_offer);
                        }
                    }
                    _ => {}
                }
            }

            GameUpdate::ExposePlayer { player, wallet } => {
                if &player == self.player.borrow().id() {
                    self.limit_bidding_until_next_auction = true;
                }
            }

            _ => {}
        }

        self.player
            .borrow_mut()
            .player_actions()
            ._receive_game_update(update)
    }
}

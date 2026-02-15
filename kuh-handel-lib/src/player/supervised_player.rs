use std::cell::RefCell;

use std::collections::BTreeMap;
use std::rc::Rc;

use crate::animals::Animal;
use crate::messages::actions::*;
use crate::messages::game_updates::*;
use crate::messages::message_protocol::StateMessage;
use crate::player::{
    base_player::{Player, PlayerId},
    player_actions::PlayerActions,
    wallet::{Affordability::*, Wallet},
};
use crate::{Money, Value};

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
    raise_faulty_action_warning: bool,
}

// todo tell the bot if action was changed

impl SupervisedPlayer {
    pub fn new(
        player: Rc<RefCell<Player>>,
        opponents: Vec<Rc<RefCell<Player>>>,
        raise_faulty_action_warning: bool,
    ) -> Self {
        SupervisedPlayer {
            player: player,
            opponents: opponents,
            limit_bidding_until_next_auction: false,
            raise_faulty_action_warning: raise_faulty_action_warning,
        }
    }

    pub fn clone_wallet(&self) -> Wallet {
        self.player.borrow().wallet().clone()
    }

    pub fn id(&self) -> PlayerId {
        self.player.borrow().id().clone()
    }

    pub fn can_trade(&self) -> Option<InitialTrade> {
        self.player.borrow().can_trade(&self.opponents)
    }
    pub fn clone_owned_animals(&self) -> BTreeMap<Animal, usize> {
        self.player.borrow().owned_animals().clone()
    }

    pub fn calculate_points(&self) -> Points {
        self.player.borrow().calculate_points()
    }

    fn rectify_money_combination(&self, combination: &Vec<Money>) -> Vec<Money> {
        match self.player.borrow().wallet().can_afford(combination) {
            Exact(exact_amount) => exact_amount,
            Alternative(alternative) => alternative,
            CannotAfford() => self.player.borrow().wallet().to_vec(),
        }
    }

    fn rectify_initial_trade(&self, trade: &InitialTrade) -> InitialTrade {
        if self.id() == trade.opponent {
            return self.can_trade().unwrap(); // todo remove unwrap if return is changed to option
        }

        let new_amount = self.rectify_money_combination(&trade.amount);

        let trade_animal = trade.animal;
        let animal_count: usize = trade.animal_count.clone() as usize;

        let animal_trade_count = match self.player.borrow().owned_animals().get(&trade_animal) {
            Some(&count) => {
                if count > 0 {
                    count
                } else {
                    panic!(
                        "animal was in BTreeMap, but amount was set to zero, this should/ can not happen"
                    )
                }
            }
            None => return self.can_trade().unwrap(), // player does not have animal, todo remove unwrap if return is changed to option
        };

        let opponent: Option<&Rc<RefCell<Player>>> = self
            .opponents
            .iter()
            .find(|player| player.borrow().id() == &trade.opponent);

        let opponent = match opponent {
            Some(opponent) => opponent,
            None => return self.can_trade().unwrap(), // opponent does not exist, todo remove unwrap if return is changed to option
        };

        let opponent_animal_count = match opponent.borrow().owned_animals().get(&trade_animal) {
            Some(&count) => {
                if count > 0 {
                    count
                } else {
                    panic!(
                        "animal was in BTreeMap, but amount was set to zero, this should/ can not happen"
                    )
                }
            }
            None => return self.can_trade().unwrap(), // opponent does not have animal, todo remove unwrap if return is changed to option
        };
        // is never 0

        let mut new_trade = trade.clone();
        new_trade.opponent = opponent.borrow().id().clone();
        new_trade.animal = trade_animal;
        new_trade.amount = new_amount;
        new_trade.animal_count = std::cmp::min(opponent_animal_count, animal_trade_count);
        new_trade
    }

    fn rectify_payment(&self, send_money: &SendMoney, value_amount: Value) -> SendMoney {
        let mut has_enough_money = self
            .player
            .borrow()
            .wallet()
            .can_afford(&vec![value_amount]);

        match send_money {
            SendMoney::Amount(bill_combination_amount) => {
                let total_payed: Value = bill_combination_amount.iter().map(|money| money).sum();

                if total_payed >= value_amount {
                    has_enough_money = self
                        .player
                        .borrow()
                        .wallet()
                        .can_afford(bill_combination_amount);
                }
            }
            SendMoney::WasBluff() => (),
        }

        match has_enough_money {
            Exact(exact_payment) => return SendMoney::Amount(exact_payment),
            Alternative(alternative_payment) => {
                return SendMoney::Amount(alternative_payment);
            }

            CannotAfford() => return SendMoney::WasBluff(),
        }
    }

    pub fn map_to_action_inner<T: FromActionMessage>(&mut self, state_msg: StateMessage) -> T {
        let action_msg = self.map_to_action(state_msg);
        T::extract(action_msg).unwrap()
    }

    pub fn raise_warning<T: PartialEq>(&self, original_action: &T, rectified_action: &T) {
        if self.raise_faulty_action_warning && original_action == rectified_action {
            eprintln!("rectified action of:{}", self.id())
        }
    }
}

impl PlayerActions for SupervisedPlayer {
    fn _draw_or_trade(&mut self) -> PlayerTurnDecision {
        let decision: PlayerTurnDecision =
            self.player.borrow_mut().player_actions()._draw_or_trade();

        let rectified_decision = match decision {
            PlayerTurnDecision::Draw() => decision.clone(),
            PlayerTurnDecision::Trade(initial_trade) => {
                if self.can_trade().is_some() {
                    PlayerTurnDecision::Trade(self.rectify_initial_trade(&initial_trade));
                }
                return PlayerTurnDecision::Draw();
            }
        };

        self.raise_warning(&decision, &rectified_decision);
        return rectified_decision;
    }

    fn _trade(&mut self) -> InitialTrade {
        let decision: InitialTrade = self.player.borrow_mut().player_actions()._trade();
        let rectified_decision = self.rectify_initial_trade(&decision);

        self.raise_warning(&decision, &rectified_decision);
        return rectified_decision;
    }

    fn _provide_bidding(&mut self, state: AuctionRound) -> Bidding {
        let decision = self
            .player
            .borrow_mut()
            .player_actions()
            ._provide_bidding(state.clone());

        let current_max_bid_value = match state.bids.iter().max_by_key(|(_, bid)| bid) {
            Some((_, max_bid)) => match max_bid {
                Bidding::Pass() => 0,
                Bidding::Bid(max_bid_value) => *max_bid_value,
            },
            None => 0,
        };

        let rectified_bidding = if self.limit_bidding_until_next_auction {
            let limit = self.player.borrow().wallet().total_money();
            match decision {
                Bidding::Pass() => decision.clone(),
                Bidding::Bid(value) => {
                    if value > limit {
                        Bidding::Bid(limit)
                    } else {
                        decision.clone()
                    }
                }
            }
        } else {
            decision.clone()
        };

        let rectified_decision = match rectified_bidding {
            Bidding::Pass() => Bidding::Pass(),
            Bidding::Bid(value) => {
                if value <= current_max_bid_value {
                    Bidding::Pass()
                } else {
                    Bidding::Bid(value)
                }
            }
        };
        self.raise_warning(&decision, &rectified_decision);
        return rectified_decision;
    }

    fn _buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision {
        let decision = self
            .player
            .borrow_mut()
            .player_actions()
            ._buy_or_sell(state);

        let rectified_decision = if self.limit_bidding_until_next_auction {
            return AuctionDecision::Sell();
        } else {
            decision.clone()
        };

        self.raise_warning(&decision, &rectified_decision);
        return rectified_decision;
    }

    fn _send_money_to_player(&mut self, player: &PlayerId, amount: Value) -> SendMoney {
        let decision = self
            .player
            .borrow_mut()
            .player_actions()
            ._send_money_to_player(player, amount);

        let rectified_decision = self.rectify_payment(&decision, amount);

        self.raise_warning(&decision, &rectified_decision);
        return rectified_decision;
    }

    fn _respond_to_trade(&mut self, offer: TradeOffer) -> TradeOpponentDecision {
        let decision = self
            .player
            .borrow_mut()
            .player_actions()
            ._respond_to_trade(offer);
        let rectified_decision = match &decision {
            TradeOpponentDecision::Accept() => decision.clone(),
            TradeOpponentDecision::CounterOffer(amount) => {
                TradeOpponentDecision::CounterOffer(self.rectify_money_combination(&amount))
            }
        };

        self.raise_warning(&decision, &rectified_decision);
        return rectified_decision;
    }

    fn _receive_game_update(&mut self, update: GameUpdate) -> NoAction {
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
                    } => match money_transfer {
                        MoneyTransfer::Private { amount } => {
                            let mut player = self.player.borrow_mut();
                            if player.id() == &from {
                                let _ = player.wallet_mut().withdraw(&amount);
                                player.add_animals(&rounds.animal, 1);
                            } else if player.id() == &to {
                                player.wallet_mut().deposit(&amount);
                            }
                        }
                        MoneyTransfer::Public {
                            card_amount: _,
                            min_value: _,
                        } => {}
                    },
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
                if player_id == challenger || player_id == opponent {
                    if player_id == receiver {
                        player.add_animals(&animal, animal_count);
                    } else {
                        let _ = player.remove_animals(&animal, animal_count);
                    }
                }
                match money_trade {
                    MoneyTrade::Private {
                        challenger_card_offer,
                        opponent_card_offer,
                    } => {
                        if player_id == challenger {
                            let _ = player.wallet_mut().withdraw(&challenger_card_offer);
                            opponent_card_offer.map(|amount| player.wallet_mut().deposit(&amount));
                        } else {
                            opponent_card_offer.map(|amount| player.wallet_mut().withdraw(&amount));
                            player.wallet_mut().deposit(&challenger_card_offer);
                        }
                    }
                    MoneyTrade::Public {
                        challenger_card_offer: _,
                        opponent_card_offer: _,
                    } => {}
                }
            }
            GameUpdate::ExposePlayer { player, wallet: _ } => {
                if &player == self.player.borrow().id() {
                    self.limit_bidding_until_next_auction = true;
                }
            }
            GameUpdate::Inflation(inflation) => {
                self.player.borrow_mut().wallet_mut().add_money(inflation);
            }
            GameUpdate::Start {
                wallet: _,
                players_in_turn_order: _,
                animals: _,
            } => {} // GameUpdate::Start is handled by the game logic when initializing a new player, because then the opponents can be Rc
            GameUpdate::End { ranking: _ } => {} // nothing to do
        }

        self.player
            .borrow_mut()
            .player_actions()
            ._receive_game_update(update)
    }
}

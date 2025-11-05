use crate::messages::actions::{
    AuctionDecision, Bidding, InitialTrade, NoAction, PlayerTurnDecision, SendMoney,
    TradeOpponentDecision,
};

use crate::Value;
use crate::messages::game_updates::{AuctionRound, GameUpdate, TradeOffer};
use crate::messages::message_protocol::{ActionMessage, StateMessage};
use crate::player::base_player::PlayerId;

pub trait PlayerActions {
    fn map_to_action(&mut self, state_msg: StateMessage) -> ActionMessage {
        match state_msg {
            StateMessage::DrawOrTrade() => ActionMessage::PlayerTurnDecision {
                decision: self._draw_or_trade(),
            },
            StateMessage::Trade() => ActionMessage::InitialTrade {
                decision: self._trade(),
            },
            StateMessage::ProvideBidding { state } => ActionMessage::Bidding {
                decision: self._provide_bidding(state),
            },
            StateMessage::BuyOrSell { state } => ActionMessage::AuctionDecision {
                decision: self._buy_or_sell(state),
            },
            StateMessage::SendMoney { player_id, amount } => ActionMessage::SendMoney {
                decision: self._send_money_to_player(&player_id, amount),
            },
            StateMessage::RespondToTrade { offer } => ActionMessage::TradeOpponentDecision {
                decision: self._respond_to_trade(offer),
            },
            StateMessage::GameUpdate { update } => ActionMessage::NoAction {
                decision: self._receive_game_update(update),
            },
        }
    }

    /// If it is a players turn, it must decide whether to draw a card or to trade with an other player.
    /// In the latter case it must provide all the information necessary for the trade.
    fn _draw_or_trade(&mut self) -> PlayerTurnDecision;

    /// In the trading phase the player that is at turn must provide the trade details.
    fn _trade(&mut self) -> InitialTrade;

    /// Each player receives the current state of the auction and must provide a bid or pass.
    /// If all players have bid pass, the auction is over.
    /// The game will inform all players about the result of the auction.
    fn _provide_bidding(&mut self, state: AuctionRound) -> Bidding;

    /// After an auction, the player that was at turn must decide whether to buy or to sell the animal.
    fn _buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision;

    /// After a bid, the player must send the bidden amount to the game, such that the transfer can be handled.
    /// If the player does not send enough money, the game will expose the players wallet if necessary.
    fn _send_money_to_player(&mut self, player: &PlayerId, amount: Value) -> SendMoney;

    /// the opponent receives the trade offer and can decide to accept it or to make a counter offer
    fn _respond_to_trade(&mut self, offer: TradeOffer) -> TradeOpponentDecision;

    /// At the begin and end of the game, after an auction and after trades, each player receives an update of the game.
    /// The game itself will not store all the events. They have to be collected by the entity that implements this trait.
    fn _receive_game_update(&mut self, update: GameUpdate) -> NoAction;

    // Implemented workflow
    //     - 1. players turn -> decided to a) draw or b) trade
    //     -    a) game draws card and start bidding, player receives bidding result at the end
    //     -    b) player must send information about the trade, game notifies opponent, collects the information and decided who has won
    //     - 2. player bid from state information
    //     - 3. player at turn gets information of bidding and then decides to buy or sell the animal
    //     - 4. player can be opponent: receives trade offer and must take it or answer
    //     - 5. player can receive money from other player (that will be handled by the game)
}

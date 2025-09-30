use crate::model::money::money::Money;
use crate::model::money::value::Value;
use crate::model::player::base_player::PlayerId;
use crate::model::player::player_actions::actions::InitialTrade;
use crate::player_actions::actions::{
    AuctionDecision, PlayerTurnDecision, TradeOffer, TradeOpponentDecision,
};
use crate::player_actions::game_updates::{AuctionRound, Bidding, GameUpdate};

pub trait PlayerActions: Send + Sync {
    /// If it is a players turn, it must decide whether to draw a card or to trade with an other player.
    /// In the latter case it must provide all the information necessary for the trade.
    fn draw_or_trade(&mut self) -> PlayerTurnDecision;

    /// In the trading phase the player that is at turn must provide the trade details.
    fn trade(&mut self) -> InitialTrade;

    /// Each player receives the current state of the auction and must provide a bid or pass.
    /// If all players have bid pass, the auction is over.
    /// The game will inform all players about the result of the auction.
    fn provide_bidding(&mut self, state: AuctionRound) -> Bidding;

    /// After an auction, the player that was at turn must decide whether to buy or to sell the animal.
    fn buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision;

    /// After a bid, the player must send the bidden amount to the game, such that the transfer can be handled.
    /// If the player does not send enough money, the game will expose the players wallet if necessary.
    fn send_money_to_player(&mut self, player: &PlayerId, amount: Value) -> Vec<Money>;

    /// After bidding or trade, a player can receive money from another player.
    /// Trade: The opponent will receive at least on money card from the challenger.
    ///        The challenger can receive money card from the opponent.
    ///        If the opponent accepts, the challenger receives an empty vector.
    fn receive_from_player(&mut self, player: &PlayerId, money: Vec<Money>);

    /// the opponent receives the trade offer and can decide to accept it or to make a counter offer
    fn respond_to_trade(&mut self, offer: TradeOffer) -> TradeOpponentDecision;

    /// At the begin and end of the game, after an auction and after trades, each player receives an update of the game.
    /// The game itself will not store all the events. They have to be collected by the entity that implements this trait.
    fn receive_game_update(&mut self, update: GameUpdate);

    // Implemented workflow
    //     - 1. players turn -> decided to a) draw or b) trade
    //     -    a) game draws card and start bidding, player receives bidding result at the end
    //     -    b) player must send information about the trade, game notifies opponent, collects the information and decided who has won
    //     - 2. player bid from state information
    //     - 3. player at turn gets information of bidding and then decides to buy or sell the animal
    //     - 4. player can be opponent: receives trade offer and must take it or answer
    //     - 5. player can receive money from other player (that will be handled by the game)
}

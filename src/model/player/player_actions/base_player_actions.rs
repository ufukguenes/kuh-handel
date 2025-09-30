use crate::player_actions::actions::{AuctionAction, PlayerTurnDecision, TradeOffer};
use crate::player_actions::game_updates::{AuctionRound, Bidding, GameUpdate};
pub trait PlayerActions: Send + Sync {
    /// If it is a players turn, it must decide whether to draw a card or to trade with an other player.
    /// 1. draw
    ///     open card
    ///     start bidding
    ///     decided if to buy yourself

    fn draw_or_trade(&mut self) -> PlayerTurnDecision;

    // auction
    fn provide_bidding(&mut self, state: AuctionRound) -> Bidding;

    fn buy_or_sell(&mut self, state: AuctionRound) -> AuctionAction;

    // trade
    fn respond_to_trade(&mut self, offer: TradeOffer) -> TradeOpponentDecision;

    fn make_trade_decision(&mut self, offer: CounterOffer) -> TradeDecision;

    // overall updates:

    /// At the begin and end of the game, after an auction and after trades, each player receives an update of the game.
    /// The game itself will not store all the events. They have to be collected by the entity that implements this trait.
    fn receive_game_update(&mut self, update: GameUpdate);

    // requirements for the player api
    //      1. players turn -> decided to a) draw or b) trade
    //     -    a) game draws card and start bidding, player receives bidding result at the end
    //         b) player must send information about the trade, game notifies opponent, collects the information and decided who has won
    //     - 2. player bid from state information
    //     - 3. player at turn gets information of bidding and then decides to buy or sell the animal
    //      4. player can be opponent: receives trade offer and must take it or answer
    //      5. player can receive money from other player (that will be handled by the game)
    //      6.
    //      7.
}

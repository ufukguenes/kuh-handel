use crate::messages::actions::{
    AuctionDecision, Bidding, InitialTrade, NoAction, PlayerTurnDecision, SendMoney, TradeOffer,
    TradeOpponentDecision,
};
use crate::messages::game_updates::{AuctionRound, GameUpdate};
use crate::model::{
    money::{money::Money, value::Value},
    player::{base_player::PlayerId, player_actions::base_player_actions::PlayerActions},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ActionMessage {
    PlayerTurnDecision { decision: PlayerTurnDecision },
    InitialTrade { decision: InitialTrade },
    Bidding { decision: Bidding },
    AuctionDecision { decision: AuctionDecision },
    SendMoney { decision: SendMoney },
    TradeOpponentDecision { decision: TradeOpponentDecision },
    NoAction { decision: NoAction },
}

//todo: make the base_player an enum, so that it is always ensured that each action type also has a message
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum StateMessage {
    DrawOrTrade,
    Trade,
    ProvideBidding {
        state: AuctionRound,
    },
    BuyOrSell {
        state: AuctionRound,
    },
    SendMoney {
        player_id: PlayerId,
        amount: Value,
    },
    ReceiveFromPlayer {
        player_id: PlayerId,
        money: Vec<Money>,
    },
    RespondToTrade {
        offer: TradeOffer,
    },
    GameUpdate {
        update: GameUpdate,
    },
}

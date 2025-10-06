use crate::messages::actions::{
    AuctionDecision, Bidding, InitialTrade, PlayerTurnDecision, TradeOffer, TradeOpponentDecision,
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
    SendMoney { decision: Vec<Money> },
    TradeOpponentDecision { decision: TradeOpponentDecision },
    NoAction,
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

impl StateMessage {
    pub fn call_action(self, player: &mut dyn PlayerActions) -> ActionMessage {
        match self {
            StateMessage::DrawOrTrade => ActionMessage::PlayerTurnDecision {
                decision: player.draw_or_trade(),
            },
            StateMessage::Trade => ActionMessage::InitialTrade {
                decision: player.trade(),
            },
            StateMessage::ProvideBidding { state } => ActionMessage::Bidding {
                decision: player.provide_bidding(state),
            },
            StateMessage::BuyOrSell { state } => ActionMessage::AuctionDecision {
                decision: player.buy_or_sell(state),
            },
            StateMessage::SendMoney { player_id, amount } => ActionMessage::SendMoney {
                decision: player.send_money_to_player(&player_id, amount),
            },
            StateMessage::ReceiveFromPlayer { player_id, money } => {
                player.receive_from_player(&player_id, money);
                ActionMessage::NoAction
            }
            StateMessage::RespondToTrade { offer } => ActionMessage::TradeOpponentDecision {
                decision: player.respond_to_trade(offer),
            },
            StateMessage::GameUpdate { update } => {
                player.receive_game_update(update);
                ActionMessage::NoAction
            }
        }
    }
}

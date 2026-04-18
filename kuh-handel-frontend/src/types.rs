use serde::{Deserialize, Serialize};

pub type Money = usize;
pub type Value = usize;
pub type PlayerId = String;
pub type Points = usize;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Animal {
    pub value: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnimalSet {
    pub animal: Animal,
    pub inflation: Vec<Value>,
    pub draw_count: usize,
    pub animals: Vec<Animal>,
}

// bank_notes serialised as [[denom, count], ...] by the server's serde_with
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Wallet {
    pub bank_notes: Vec<(Money, usize)>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuctionRound {
    pub host: PlayerId,
    pub animal: Animal,
    pub bids: Vec<(PlayerId, Bidding)>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TradeOffer {
    pub challenger: PlayerId,
    pub animal: Animal,
    pub animal_count: usize,
    pub challenger_card_offer: usize,
}

// ── inbound messages (server → client) ────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum StateMessage {
    DrawOrTrade(),
    Trade(),
    ProvideBidding { state: AuctionRound },
    BuyOrSell { state: AuctionRound },
    SendMoney { player_id: PlayerId, amount: Value },
    RespondToTrade { offer: TradeOffer },
    GameUpdate { update: GameUpdate },
}

// ── outbound messages (client → server) ───────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum ActionMessage {
    PlayerTurnDecision { decision: PlayerTurnDecision },
    InitialTrade { decision: InitialTrade },
    Bidding { decision: BiddingDecision },
    AuctionDecision { decision: AuctionDecision },
    SendMoney { decision: SendMoneyDecision },
    TradeOpponentDecision { decision: TradeOpponentDecision },
    NoAction { decision: NoActionDecision },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PlayerTurnDecision {
    Draw(),
    Trade(InitialTrade),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitialTrade {
    pub opponent: PlayerId,
    pub animal: Animal,
    pub animal_count: usize,
    pub amount: Vec<Money>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Bidding {
    Pass(),
    Bid(Value),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BiddingDecision {
    Pass(),
    Bid(Value),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AuctionDecision {
    Buy(),
    Sell(),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SendMoneyDecision {
    WasBluff(),
    Amount(Vec<Money>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TradeOpponentDecision {
    Accept(),
    CounterOffer(Vec<Money>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NoActionDecision {
    Ok(),
}

// ── game updates ──────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameUpdate {
    Auction(AuctionKind),
    Trade {
        challenger: PlayerId,
        opponent: PlayerId,
        animal: Animal,
        animal_count: usize,
        receiver: PlayerId,
        money_trade: MoneyTrade,
    },
    Start {
        wallet: Wallet,
        players_in_turn_order: Vec<PlayerId>,
        animals: Vec<AnimalSet>,
    },
    End {
        ranking: Vec<(PlayerId, Points)>,
        illegal_moves_made: Vec<String>,
    },
    ExposePlayer {
        player: PlayerId,
        wallet: Wallet,
    },
    Inflation(Money),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AuctionKind {
    NoBiddings { host_id: PlayerId, animal: Animal },
    NormalAuction {
        rounds: AuctionRound,
        from: PlayerId,
        to: PlayerId,
        money_transfer: MoneyTransfer,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MoneyTransfer {
    Public { card_amount: usize, min_value: Value },
    Private { amount: Vec<Money> },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MoneyTrade {
    Public {
        challenger_card_offer: usize,
        opponent_card_offer: Option<usize>,
    },
    Private {
        challenger_card_offer: Vec<Money>,
        opponent_card_offer: Option<Vec<Money>>,
    },
}

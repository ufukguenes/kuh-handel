use kuh_handel_lib::Value;
use kuh_handel_lib::client::Client;
use kuh_handel_lib::messages::actions::{
    AuctionDecision, Bidding, InitialTrade, NoAction, PlayerTurnDecision, SendMoney,
    TradeOpponentDecision,
};
use kuh_handel_lib::messages::game_updates::{AuctionRound, GameUpdate, TradeOffer};
use kuh_handel_lib::player::base_player::PlayerId;
use kuh_handel_lib::player::player_actions::PlayerActions;
use kuh_handel_lib::player::simple_player::SimplePlayer;

#[derive(Default)]
struct Bot {
    simple_player: SimplePlayer,
}

impl PlayerActions for Bot {
    fn _draw_or_trade(&mut self) -> PlayerTurnDecision {
        self.simple_player._draw_or_trade()
    }

    fn _trade(&mut self) -> InitialTrade {
        self.simple_player._trade()
    }

    fn _provide_bidding(&mut self, state: AuctionRound) -> Bidding {
        self.simple_player._provide_bidding(state)
    }

    fn _buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision {
        self.simple_player._buy_or_sell(state)
    }

    fn _send_money_to_player(&mut self, player: &PlayerId, amount: Value) -> SendMoney {
        self.simple_player._send_money_to_player(player, amount)
    }

    fn _respond_to_trade(&mut self, offer: TradeOffer) -> TradeOpponentDecision {
        self.simple_player._respond_to_trade(offer)
    }

    fn _receive_game_update(&mut self, update: GameUpdate) -> NoAction {
        self.simple_player._receive_game_update(update)
    }
}

pub async fn run(client: &mut Client, num_rounds: u32) {
    for _ in 0..num_rounds {
        client.play_one_round("pvp_games".to_string()).await;
    }
}

#[tokio::main]
async fn main() {
    let name = "your_bot_name".to_string();
    let token = "your_private_token".to_string();
    let base_url = "s://ufuk-guenes.com".to_string(); // "://127.0.0.1:2000"
    let raise_faulty_action_warning = true;
    let play_n_rounds = 1;

    let bot = Box::new(Bot::default());

    let mut client = Client::new(name, token, bot, base_url, raise_faulty_action_warning);

    run(&mut client, play_n_rounds).await;
}

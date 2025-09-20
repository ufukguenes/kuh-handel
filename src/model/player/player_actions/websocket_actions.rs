pub struct WebsocketActions {
    sender: mpsc::Sender<Message>,
}

impl PlayerActions for WebsocketPlayer {
    fn provide_bidding(
        &mut self,
        state: model::player::AuctionState,
    ) -> model::player::AuctionValue {
        todo!()
    }

    fn draw_or_trade(&mut self) -> model::player::FirstPhaseAction {
        todo!()
    }

    fn buy_or_sell(&mut self, state: model::player::AuctionState) -> model::player::AuctionAction {
        todo!()
    }
}

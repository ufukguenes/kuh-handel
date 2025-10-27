from abc import ABC, abstractmethod
from ..messages import actions

class PlayerActions(ABC): 
    @abstractmethod
    def _draw_or_trade(self) -> actions.PlayerTurnDecision: ...

    @abstractmethod
    def _trade(self) -> actions.InitialTrade: ...
    
    @abstractmethod
    def _provide_bidding(self, state) -> actions.Bidding: ...
    
    @abstractmethod
    def _buy_or_sell(self, state) -> actions.AuctionDecision: ...

    @abstractmethod
    def _send_money_to_player(self, player, amount) -> actions.SendMoney: ...
    
    @abstractmethod
    def _respond_to_trade(self, offer) -> actions.TradeOpponentDecision: ...
    
    @abstractmethod
    def _receive_game_update(self, update) -> actions.NoAction: ...

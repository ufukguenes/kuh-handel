from abc import ABC, abstractmethod

class PlayerActions(ABC): 
    @abstractmethod
    def _draw_or_trade(self): pass

    @abstractmethod
    def _trade(self): pass
    
    @abstractmethod
    def _provide_bidding(self, state): pass
    
    @abstractmethod
    def _buy_or_sell(self, state): pass
    def _send_money_to_player(self, player, amount): pass
    
    @abstractmethod
    def _respond_to_trade(self, offer): pass
    
    @abstractmethod
    def _receive_game_update(self, update): pass

use std::collections::HashMap;

use leptos::prelude::*;
use send_wrapper::SendWrapper;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CloseEvent, ErrorEvent, MessageEvent, WebSocket};

use crate::types::*;

const WS_URL: &str = "wss://ufuk-guenes.com/kuh-handel/interactive_game";

const LEGACY_ANIMAL_MAPPING: &[(usize, &str)] = &[
    (10, "cock"),
    (40, "goose"),
    (90, "cat"),
    (160, "dog"),
    (250, "sheep"),
    (350, "goat"),
    (500, "cash cow"),
    (650, "pig"),
    (800, "bull"),
    (1000, "horse"),
];

const RANDOM_ANIMAL_NAMES: &[&str] = &[
    "horse",
    "dog",
    "cat",
    "donkey",
    "cow",
    "giraffe",
    "eagle",
    "pig",
    "mouse",
    "sheep",
    "electric eel",
    "dolphin",
    "bull",
    "cash cow",
    "goat",
    "cock",
];

// ── UI state enums ─────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug)]
pub enum Screen {
    Login,
    Waiting,
    Game,
    LostConnection,
    Error,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ActionView {
    Pending,
    Message,
    DrawOrTrade,
    Auction,
    SendMoney,
    TradeOffer,
    TradeResponse,
}

#[derive(Clone, PartialEq, Debug)]
pub enum TradeMode {
    InitialTrade,
    CounterOffer,
    TradeDecision,
}

// ── view-model types ───────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub struct AnimalInfo {
    pub name: String,
    pub value: usize,
    pub count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlayerInfo {
    pub name: String,
    pub money_count: usize,
    pub animals: Vec<AnimalInfo>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BidEntry {
    pub player: String,
    pub bid: Option<usize>,
}

// ── helpers ────────────────────────────────────────────────────────────────────

fn resolve_animal_name(animal_sets: &[AnimalSet]) -> HashMap<usize, String> {
    let is_legacy = animal_sets.iter().all(|s| {
        LEGACY_ANIMAL_MAPPING
            .iter()
            .any(|&(v, _)| v == s.animal.value)
    });

    let mut mapping = HashMap::new();
    let mut name_count: [usize; 2] = [0, 0];

    for s in animal_sets {
        let name = if is_legacy {
            LEGACY_ANIMAL_MAPPING
                .iter()
                .find(|&&(v, _)| v == s.animal.value)
                .map(|&(_, n)| n.to_string())
                .unwrap_or_else(|| s.animal.value.to_string())
        } else {
            let n = RANDOM_ANIMAL_NAMES.len();
            if name_count[1] == n {
                name_count = [name_count[0] + 1, name_count[0]];
            }
            let name = if name_count[0] == 0 {
                RANDOM_ANIMAL_NAMES[name_count[1]].to_string()
            } else {
                format!(
                    "{}-{}",
                    RANDOM_ANIMAL_NAMES[name_count[0] - 1],
                    RANDOM_ANIMAL_NAMES[name_count[1]]
                )
            };
            name_count[1] += 1;
            name
        };
        mapping.insert(s.animal.value, name);
    }
    mapping
}

fn send_ws(ws: RwSignal<Option<SendWrapper<WebSocket>>>, msg: &ActionMessage) {
    if let Ok(text) = serde_json::to_string(msg) {
        ws.with_untracked(|opt| {
            if let Some(sw) = opt {
                let _ = sw.send_with_str(&text);
            }
        });
    }
}

// ── App component ──────────────────────────────────────────────────────────────

#[component]
pub fn App() -> impl IntoView {
    // ── form inputs ──────────────────────────────────────────────────────────
    let player_id = RwSignal::new(String::new());
    let player_token = RwSignal::new(String::new());

    // ── screen / action view ─────────────────────────────────────────────────
    let screen = RwSignal::new(Screen::Login);
    let action_view = RwSignal::new(ActionView::Pending);

    // ── websocket ────────────────────────────────────────────────────────────
    let ws: RwSignal<Option<SendWrapper<WebSocket>>> = RwSignal::new(None);

    // ── game data ─────────────────────────────────────────────────────────────
    let deck_count = RwSignal::new(0usize);
    let current_card: RwSignal<Option<(String, usize)>> = RwSignal::new(None); // (name, value)
    let highest_bid: RwSignal<Option<usize>> = RwSignal::new(None);
    let highest_bidder: RwSignal<Option<String>> = RwSignal::new(None);
    let animals: RwSignal<Vec<AnimalInfo>> = RwSignal::new(vec![]);
    let players: RwSignal<Vec<PlayerInfo>> = RwSignal::new(vec![]);
    let banknotes: RwSignal<Vec<(usize, usize)>> = RwSignal::new(vec![]); // (value, count)
    let animal_mapping: RwSignal<HashMap<usize, String>> = RwSignal::new(HashMap::new());
    let inverse_animal_mapping: RwSignal<HashMap<String, usize>> = RwSignal::new(HashMap::new());

    // ── auction state ─────────────────────────────────────────────────────────
    let bid_input = RwSignal::new(0u32);
    let bid_min = RwSignal::new(1u32);
    let bids: RwSignal<Vec<BidEntry>> = RwSignal::new(vec![]);
    let is_auction_host = RwSignal::new(false);

    // ── send-money state ──────────────────────────────────────────────────────
    let money_receiver = RwSignal::new(String::new());
    let min_amount = RwSignal::new(0usize);
    let banknote_selection: RwSignal<HashMap<usize, usize>> = RwSignal::new(HashMap::new());
    let on_trade = RwSignal::new(false);
    let trade_mode: RwSignal<Option<TradeMode>> = RwSignal::new(None);

    // ── trade offer state ─────────────────────────────────────────────────────
    let trade_opponent = RwSignal::new(String::new());
    let trade_animal_id = RwSignal::new(String::new()); // "name_Nx"
    let show_offer_back = RwSignal::new(false);
    let show_send_back = RwSignal::new(false);

    // ── trade response state ──────────────────────────────────────────────────
    let trade_offer_data: RwSignal<Option<TradeOffer>> = RwSignal::new(None);

    // ── message state ─────────────────────────────────────────────────────────
    let message_header = RwSignal::new(String::from("Message"));
    let message_body = RwSignal::new(String::new());

    // ── computed: selected money amount ───────────────────────────────────────
    let selected_amount = Memo::new(move |_| {
        banknote_selection
            .get()
            .iter()
            .map(|(v, c)| v * c)
            .sum::<usize>()
    });

    // ── computed: trade opponents ─────────────────────────────────────────────
    let trade_opponents = Memo::new(move |_| {
        let pid = player_id.get();
        players
            .get()
            .into_iter()
            .filter(|p| p.name != pid)
            .collect::<Vec<_>>()
    });

    // ── computed: animals available to trade with selected opponent ───────────
    let trade_animals = Memo::new(move |_| {
        let pid = player_id.get();
        let opp = trade_opponent.get();
        let all_players = players.get();
        let my_animals = all_players
            .iter()
            .find(|p| p.name == pid)
            .map(|p| p.animals.clone())
            .unwrap_or_default();
        let opp_animals = all_players
            .iter()
            .find(|p| p.name == opp)
            .map(|p| p.animals.clone())
            .unwrap_or_default();

        opp_animals
            .into_iter()
            .filter_map(|oa| {
                let my = my_animals.iter().find(|a| a.name == oa.name)?;
                if oa.count > 0 && my.count > 0 {
                    let n = my.count.min(oa.count);
                    let id = format!("{}_{}", oa.name, n);
                    Some((id, oa.name.clone(), n))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    });

    // ── helpers that capture signals ──────────────────────────────────────────

    let send_ok = move || {
        send_ws(
            ws,
            &ActionMessage::NoAction {
                decision: NoActionDecision::Ok(),
            },
        );
    };

    let switch_to_pending = move || action_view.set(ActionView::Pending);

    let render_bids = move |round_bids: Vec<(PlayerId, Bidding)>, host: &str| {
        let all_players = players.get_untracked();
        let mut bid_map: HashMap<String, Option<usize>> = all_players
            .iter()
            .filter(|p| p.name != host)
            .map(|p| (p.name.clone(), None))
            .collect();

        for (player, decision) in &round_bids {
            if let Bidding::Bid(v) = decision {
                let entry = bid_map.entry(player.clone()).or_insert(None);
                *entry = Some(match entry {
                    Some(old) => (*old).max(*v),
                    None => *v,
                });
            }
        }

        let mut top_bid = None::<usize>;
        let mut top_bidder = None::<String>;
        let mut entries: Vec<BidEntry> = bid_map
            .into_iter()
            .map(|(player, bid)| {
                if let Some(b) = bid {
                    if top_bid.map_or(true, |t| b > t) {
                        top_bid = Some(b);
                        top_bidder = Some(player.clone());
                    }
                }
                BidEntry { player, bid }
            })
            .collect();
        entries.sort_by(|a, b| a.player.cmp(&b.player));
        bids.set(entries);
        highest_bid.set(top_bid);
        highest_bidder.set(top_bidder);
    };

    let handle_send_money = move |receiver: String, amount: usize, is_trade: bool| {
        // check for bluff: total wallet < required
        let total: usize = banknotes.get_untracked().iter().map(|(v, c)| v * c).sum();
        if !is_trade && total < amount {
            switch_to_pending();
            send_ws(
                ws,
                &ActionMessage::SendMoney {
                    decision: SendMoneyDecision::WasBluff(),
                },
            );
            return;
        }
        money_receiver.set(receiver);
        min_amount.set(amount);
        banknote_selection.set(HashMap::new());
        on_trade.set(is_trade);
        show_send_back
            .set(is_trade && !matches!(trade_mode.get_untracked(), Some(TradeMode::CounterOffer)));
        action_view.set(ActionView::SendMoney);
    };

    // ── WebSocket connection ──────────────────────────────────────────────────

    let connect = move |_| {
        let pid = player_id.get_untracked();
        let tok = player_token.get_untracked();
        let url = format!(
            "{}?player_id={}&token={}&raise_faulty_action_warning=false",
            WS_URL, pid, tok
        );

        // close any existing connection
        ws.with_untracked(|opt| {
            if let Some(sw) = opt {
                let _ = sw.close();
            }
        });
        ws.set(None);

        let socket = match WebSocket::new(&url) {
            Ok(s) => s,
            Err(_) => {
                screen.set(Screen::Error);
                return;
            }
        };

        // onopen
        {
            let screen = screen;
            let cb = Closure::wrap(Box::new(move |_: web_sys::Event| {
                screen.set(Screen::Waiting);
            }) as Box<dyn FnMut(_)>);
            socket.set_onopen(Some(cb.as_ref().unchecked_ref()));
            cb.forget();
        }

        // onclose
        {
            let screen = screen;
            let ws = ws;
            let cb = Closure::wrap(Box::new(move |_: CloseEvent| {
                ws.set(None);
                screen.set(Screen::LostConnection);
            }) as Box<dyn FnMut(_)>);
            socket.set_onclose(Some(cb.as_ref().unchecked_ref()));
            cb.forget();
        }

        // onerror
        {
            let screen = screen;
            let cb = Closure::wrap(Box::new(move |_: ErrorEvent| {
                screen.set(Screen::Error);
            }) as Box<dyn FnMut(_)>);
            socket.set_onerror(Some(cb.as_ref().unchecked_ref()));
            cb.forget();
        }

        // onmessage — all state captured here
        {
            let animal_mapping = animal_mapping;
            let inverse_animal_mapping = inverse_animal_mapping;
            let screen = screen;
            let action_view = action_view;
            let deck_count = deck_count;
            let current_card = current_card;
            let animals = animals;
            let players = players;
            let banknotes = banknotes;
            let bids = bids;
            let highest_bid = highest_bid;
            let highest_bidder = highest_bidder;
            let bid_input = bid_input;
            let bid_min = bid_min;
            let is_auction_host = is_auction_host;
            let money_receiver = money_receiver;
            let min_amount = min_amount;
            let banknote_selection = banknote_selection;
            let on_trade = on_trade;
            let trade_mode = trade_mode;
            let trade_opponent = trade_opponent;
            let show_offer_back = show_offer_back;
            let show_send_back = show_send_back;
            let trade_offer_data = trade_offer_data;
            let message_header = message_header;
            let message_body = message_body;
            let player_id = player_id;
            let ws = ws;

            let cb = Closure::wrap(Box::new(move |e: MessageEvent| {
                let text = match e.data().as_string() {
                    Some(t) => t,
                    None => return,
                };
                let msg: StateMessage = match serde_json::from_str(&text) {
                    Ok(m) => m,
                    Err(err) => {
                        web_sys::console::error_1(&format!("parse error: {err}\n{text}").into());
                        return;
                    }
                };

                match msg {
                    StateMessage::DrawOrTrade() => {
                        action_view.set(ActionView::DrawOrTrade);
                    }

                    StateMessage::Trade() => {
                        trade_mode.set(Some(TradeMode::InitialTrade));
                        let opps: Vec<_> = players
                            .get_untracked()
                            .into_iter()
                            .filter(|p| p.name != player_id.get_untracked())
                            .collect();
                        if let Some(first) = opps.first() {
                            trade_opponent.set(first.name.clone());
                        }
                        show_offer_back.set(false);
                        action_view.set(ActionView::TradeOffer);
                    }

                    StateMessage::ProvideBidding { state: round } => {
                        let name = animal_mapping
                            .get_untracked()
                            .get(&round.animal.value)
                            .cloned()
                            .unwrap_or_else(|| round.animal.value.to_string());
                        current_card.set(Some((name, round.animal.value)));
                        render_bids(round.bids.clone(), &round.host);
                        let min = highest_bid.get_untracked().unwrap_or(0) + 1;
                        bid_min.set(min as u32);
                        bid_input.set(min as u32);
                        is_auction_host.set(false);
                        action_view.set(ActionView::Auction);
                    }

                    StateMessage::BuyOrSell { state: round } => {
                        let name = animal_mapping
                            .get_untracked()
                            .get(&round.animal.value)
                            .cloned()
                            .unwrap_or_else(|| round.animal.value.to_string());
                        current_card.set(Some((name, round.animal.value)));
                        render_bids(round.bids, &round.host);
                        is_auction_host.set(true);
                        action_view.set(ActionView::Auction);
                    }

                    StateMessage::SendMoney {
                        player_id: recv,
                        amount,
                    } => {
                        let total: usize =
                            banknotes.get_untracked().iter().map(|(v, c)| v * c).sum();
                        if total < amount {
                            action_view.set(ActionView::Pending);
                            send_ws(
                                ws,
                                &ActionMessage::SendMoney {
                                    decision: SendMoneyDecision::WasBluff(),
                                },
                            );
                            return;
                        }
                        money_receiver.set(recv);
                        min_amount.set(amount);
                        banknote_selection.set(HashMap::new());
                        on_trade.set(false);
                        show_send_back.set(false);
                        action_view.set(ActionView::SendMoney);
                    }

                    StateMessage::RespondToTrade { offer } => {
                        trade_offer_data.set(Some(offer));
                        action_view.set(ActionView::TradeResponse);
                    }

                    StateMessage::GameUpdate { update } => {
                        handle_game_update(
                            update,
                            player_id.get_untracked(),
                            screen,
                            action_view,
                            deck_count,
                            current_card,
                            animals,
                            players,
                            banknotes,
                            highest_bid,
                            highest_bidder,
                            animal_mapping,
                            inverse_animal_mapping,
                            message_header,
                            message_body,
                            ws,
                        );
                    }
                }
            }) as Box<dyn FnMut(_)>);
            socket.set_onmessage(Some(cb.as_ref().unchecked_ref()));
            cb.forget();
        }

        ws.set(Some(SendWrapper::new(socket)));
    };

    // ── action button callbacks ───────────────────────────────────────────────

    let on_draw = move |_| {
        switch_to_pending();
        send_ws(
            ws,
            &ActionMessage::PlayerTurnDecision {
                decision: PlayerTurnDecision::Draw(),
            },
        );
    };

    let on_place_trade = move |_| {
        trade_mode.set(Some(TradeMode::TradeDecision));
        let opps: Vec<_> = players
            .get_untracked()
            .into_iter()
            .filter(|p| p.name != player_id.get_untracked())
            .collect();
        if let Some(first) = opps.first() {
            trade_opponent.set(first.name.clone());
        }
        show_offer_back.set(true);
        action_view.set(ActionView::TradeOffer);
    };

    let on_offer_back = move |_| {
        action_view.set(ActionView::DrawOrTrade);
    };

    let on_bid = move |_| {
        let amount = bid_input.get_untracked() as usize;
        switch_to_pending();
        send_ws(
            ws,
            &ActionMessage::Bidding {
                decision: BiddingDecision::Bid(amount),
            },
        );
    };

    let on_pass = move |_| {
        switch_to_pending();
        send_ws(
            ws,
            &ActionMessage::Bidding {
                decision: BiddingDecision::Pass(),
            },
        );
    };

    let on_buy = move |_| {
        switch_to_pending();
        send_ws(
            ws,
            &ActionMessage::AuctionDecision {
                decision: AuctionDecision::Buy(),
            },
        );
    };

    let on_sell = move |_| {
        switch_to_pending();
        send_ws(
            ws,
            &ActionMessage::AuctionDecision {
                decision: AuctionDecision::Sell(),
            },
        );
    };

    let on_send_money = move |_| {
        let sel = banknote_selection.get_untracked();
        let mut money: Vec<usize> = sel
            .iter()
            .flat_map(|(v, c)| std::iter::repeat(*v).take(*c))
            .collect();
        money.sort();
        banknote_selection.set(HashMap::new());

        let msg = if !on_trade.get_untracked() {
            ActionMessage::SendMoney {
                decision: SendMoneyDecision::Amount(money),
            }
        } else if matches!(trade_mode.get_untracked(), Some(TradeMode::CounterOffer)) {
            ActionMessage::TradeOpponentDecision {
                decision: TradeOpponentDecision::CounterOffer(money),
            }
        } else {
            let opp = trade_opponent.get_untracked();
            let tid = trade_animal_id.get_untracked();
            let parts: Vec<&str> = tid.rsplitn(2, '_').collect();
            let count = parts
                .first()
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(1);
            let animal_name = parts.get(1).map(|s| s.to_string()).unwrap_or_default();
            let animal_val = inverse_animal_mapping
                .get_untracked()
                .get(&animal_name)
                .copied()
                .unwrap_or(0);
            let trade = InitialTrade {
                opponent: opp,
                animal: Animal { value: animal_val },
                animal_count: count,
                amount: money,
            };
            if matches!(trade_mode.get_untracked(), Some(TradeMode::TradeDecision)) {
                ActionMessage::PlayerTurnDecision {
                    decision: PlayerTurnDecision::Trade(trade),
                }
            } else {
                ActionMessage::InitialTrade { decision: trade }
            }
        };

        switch_to_pending();
        send_ws(ws, &msg);
    };

    let on_offer = move |_| {
        let opp = trade_opponent.get_untracked();
        on_trade.set(true);
        money_receiver.set(opp);
        min_amount.set(0);
        banknote_selection.set(HashMap::new());
        show_send_back.set(true);
        action_view.set(ActionView::SendMoney);
    };

    let on_wrong_trade = move |_| {
        show_offer_back.set(matches!(
            trade_mode.get_untracked(),
            Some(TradeMode::TradeDecision)
        ));
        action_view.set(ActionView::TradeOffer);
    };

    let on_ok = move |_| {
        switch_to_pending();
        send_ws(
            ws,
            &ActionMessage::NoAction {
                decision: NoActionDecision::Ok(),
            },
        );
    };

    let on_accept_trade = move |_| {
        switch_to_pending();
        send_ws(
            ws,
            &ActionMessage::TradeOpponentDecision {
                decision: TradeOpponentDecision::Accept(),
            },
        );
    };

    let on_counter_trade = move |_| {
        let offer = trade_offer_data.get_untracked();
        let opp = offer.map(|o| o.challenger).unwrap_or_default();
        trade_mode.set(Some(TradeMode::CounterOffer));
        on_trade.set(true);
        money_receiver.set(opp);
        min_amount.set(0);
        banknote_selection.set(HashMap::new());
        show_send_back.set(false);
        action_view.set(ActionView::SendMoney);
    };

    // ── view ──────────────────────────────────────────────────────────────────

    view! {
        <div id="kuh-handel-app">

            // ── Login ──────────────────────────────────────────────────────────
            <div class="box">
                <div class="box" style="background-color:transparent;margin-top:0;">
                    <h4>"Join random game"</h4>
                </div>
                <div style="display:inline-flex;margin-top:1rem;">
                    <div class="box" style="background-color:transparent;margin-top:0;">
                        <div style="display:inline-flex;gap:.25rem;">
                            <div style="padding-right:0.5rem;">
                                <label class="field-label">"Player"</label>
                                <input
                                    class="text-field"
                                    type="text"
                                    placeholder="registered player"
                                    prop:value=move || player_id.get()
                                    on:input=move |e| {
                                        let v = e.target().unwrap()
                                            .unchecked_into::<web_sys::HtmlInputElement>()
                                            .value();
                                        player_id.set(v);
                                    }
                                />
                            </div>
                            <div>
                                <label class="field-label">"Token"</label>
                                <input
                                    class="text-field"
                                    type="text"
                                    placeholder="player token"
                                    prop:value=move || player_token.get()
                                    on:input=move |e| {
                                        let v = e.target().unwrap()
                                            .unchecked_into::<web_sys::HtmlInputElement>()
                                            .value();
                                        player_token.set(v);
                                    }
                                />
                            </div>
                        </div>
                    </div>
                    <div style="display:inline-flex;padding:0.5rem 1rem;">
                        <button class="create-btn" on:click=connect>"Play"</button>
                    </div>
                </div>
            </div>

            // ── Waiting ────────────────────────────────────────────────────────
            {move || (screen.get() == Screen::Waiting).then(|| view! {
                <div class="box">
                    <div class="box-inner" style="text-align:center;">
                        <div style="padding-top:0.5rem;">"Waiting for other players to join."</div>
                    </div>
                </div>
            })}

            // ── Dashboard ──────────────────────────────────────────────────────
            {move || (screen.get() == Screen::Game).then(|| view! {
                <div class="box">
                    <div class="grid" style="margin-bottom:1rem;">
                        // State
                        <div>
                            <div class="box box-transparent"><h4>"State"</h4></div>
                            <div class="box-inner" style="margin-top:0.5rem;">
                                <div><strong>"Deck remaining: "</strong>{move || deck_count.get()}</div>
                                <div>
                                    <strong>"Current card: "</strong>
                                    {move || current_card.get()
                                        .map(|(n, _)| n)
                                        .unwrap_or_else(|| "—".to_string())}
                                </div>
                                <div>
                                    <strong>"Highest bid: "</strong>
                                    {move || highest_bid.get()
                                        .map(|b| b.to_string())
                                        .unwrap_or_else(|| "—".to_string())}
                                </div>
                                <div>
                                    <strong>"Highest bidder: "</strong>
                                    {move || highest_bidder.get()
                                        .unwrap_or_else(|| "—".to_string())}
                                </div>
                            </div>
                        </div>
                        // Animals
                        <div>
                            <div class="box box-transparent"><h4>"Animals"</h4></div>
                            <div class="box-inner" style="margin-top:0.5rem;">
                                <ul>
                                    <For
                                        each=move || animals.get()
                                        key=|a| a.value
                                        children=|a| view! {
                                            <li>{a.name.clone()}" × "{a.count}</li>
                                        }
                                    />
                                </ul>
                            </div>
                        </div>
                        // Wallet
                        <div>
                            <div class="box box-transparent"><h4>"Wallet"</h4></div>
                            <div class="box-inner" style="margin-top:0.5rem;">
                                <ul>
                                    <For
                                        each=move || banknotes.get()
                                        key=|(v, _)| *v
                                        children=|(v, c)| view! {
                                            <li>{v}" × "{c}</li>
                                        }
                                    />
                                </ul>
                            </div>
                        </div>
                    </div>
                    // Players
                    <div class="grid">
                        <div>
                            <div class="box box-transparent"><h4>"Players"</h4></div>
                            <div class="box-inner" style="margin-top:0.5rem;">
                                <ul>
                                    <For
                                        each=move || players.get()
                                        key=|p| p.name.clone()
                                        children=|p| view! {
                                            <li>
                                                <strong>{p.name.clone()}</strong>
                                                " — money cards: "{p.money_count}
                                            </li>
                                        }
                                    />
                                </ul>
                            </div>
                        </div>
                    </div>
                </div>
            })}

            // ── Action box ─────────────────────────────────────────────────────
            {move || (screen.get() == Screen::Game).then(|| view! {
                <div class="box">
                    <div class="box box-transparent" style="margin-bottom:-0.25rem;">
                        <h4>"Player Action"</h4>
                    </div>

                    // Pending
                    {move || (action_view.get() == ActionView::Pending).then(|| view! {
                        <div class="box-inner" style="text-align:center;margin-top:0.5rem;">
                            <div style="padding-top:0.5rem;">"There is currently no action to perform."</div>
                        </div>
                    })}

                    // Message
                    {move || (action_view.get() == ActionView::Message).then(|| view! {
                        <div style="margin-top:0.5rem;">
                            <h5>{move || message_header.get()}</h5>
                            <div class="box-inner" inner_html=move || message_body.get()></div>
                            <div class="flex-btn">
                                <button class="game-btn" on:click=on_ok>"Ok"</button>
                            </div>
                        </div>
                    })}

                    // Draw or Trade
                    {move || (action_view.get() == ActionView::DrawOrTrade).then(|| view! {
                        <div style="margin-top:0.5rem;">
                            <h5>"Draw or Trade"</h5>
                            <div class="flex-btn" style="padding-right:2.5rem;">
                                <button class="game-btn" on:click=on_draw>"Draw"</button>
                            </div>
                            <div class="flex-btn">
                                <button class="game-btn" on:click=on_place_trade>"Trade"</button>
                            </div>
                        </div>
                    })}

                    // Auction
                    {move || (action_view.get() == ActionView::Auction).then(|| view! {
                        <div style="margin-top:0.5rem;">
                            <h5>"Auction"</h5>
                            <div>
                                <strong>"Current card: "</strong>
                                {move || current_card.get().map(|(n,_)| n).unwrap_or_else(|| "—".into())}
                            </div>
                            <div>
                                <strong>"Bids:"</strong>
                                <ul>
                                    <For
                                        each=move || bids.get()
                                        key=|b| b.player.clone()
                                        children=|b| {
                                            let bid_text = b.bid.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string());
                                            view! {
                                                <li><p>{b.player}" → "<strong>{bid_text}</strong></p></li>
                                            }
                                        }
                                    />
                                </ul>
                            </div>

                            // Bidding input (non-host)
                            {move || (!is_auction_host.get()).then(|| view! {
                                <div style="display:inline-flex;margin-top:1rem;">
                                    <div>
                                        <label class="field-label">"Place your bid"</label>
                                        <input
                                            class="text-field"
                                            type="number"
                                            style="width:5rem;"
                                            prop:min=move || bid_min.get()
                                            prop:value=move || bid_input.get()
                                            on:input=move |e| {
                                                let v = e.target().unwrap()
                                                    .unchecked_into::<web_sys::HtmlInputElement>()
                                                    .value()
                                                    .parse::<u32>()
                                                    .unwrap_or(0);
                                                bid_input.set(v);
                                            }
                                        />
                                    </div>
                                    <div class="flex-btn" style="padding-left:1rem;margin-top:-0.5rem;margin-left:0.5rem;padding-right:1.2rem;">
                                        <button class="game-btn" on:click=on_bid>"Bid"</button>
                                    </div>
                                    <div class="flex-btn" style="padding-left:1rem;margin-top:-0.5rem;margin-left:0.5rem;">
                                        <button class="game-btn" on:click=on_pass>"Pass"</button>
                                    </div>
                                </div>
                            })}

                            // Buy/Sell (host)
                            {move || is_auction_host.get().then(|| view! {
                                <div>
                                    <div class="flex-btn" style="padding-right:2.5rem;">
                                        <button class="game-btn" on:click=on_buy>"Buy"</button>
                                    </div>
                                    <div class="flex-btn">
                                        <button class="game-btn" on:click=on_sell>"Sell"</button>
                                    </div>
                                </div>
                            })}
                        </div>
                    })}

                    // Send Money
                    {move || (action_view.get() == ActionView::SendMoney).then(|| view! {
                        <div style="margin-top:0.5rem;">
                            <h5>"Send Money"</h5>
                            <div class="grid" style="margin-bottom:1rem;">
                                <div>
                                    <div class="box">
                                        <div><label class="field-label">"Receiver"</label><strong>{move || money_receiver.get()}</strong></div>
                                        <div><label class="field-label">"Necessary amount"</label><strong>{move || min_amount.get()}</strong></div>
                                        <div><label class="field-label">"Selected amount"</label><strong>{move || selected_amount.get()}</strong></div>
                                        <div class="flex-btn">
                                            <button class="game-btn" on:click=on_send_money>"Send"</button>
                                        </div>
                                    </div>
                                </div>
                                <div>
                                    <div class="box">
                                        <ul>
                                            <For
                                                each=move || banknotes.get()
                                                key=|(v, _)| *v
                                                children=move |(v, max_count)| {
                                                    let cur = move || {
                                                        banknote_selection.get().get(&v).copied().unwrap_or(0)
                                                    };
                                                    view! {
                                                        <li>
                                                            <div style="text-align:right;padding-top:0.25rem;">
                                                                <label class="field-label">"Card "{v}</label>
                                                                <input
                                                                    class="text-field"
                                                                    type="number"
                                                                    style="max-width:4rem;"
                                                                    min="0"
                                                                    prop:max=max_count
                                                                    prop:value=cur
                                                                    on:input=move |e| {
                                                                        let val = e.target().unwrap()
                                                                            .unchecked_into::<web_sys::HtmlInputElement>()
                                                                            .value()
                                                                            .parse::<usize>()
                                                                            .unwrap_or(0)
                                                                            .min(max_count);
                                                                        banknote_selection.update(|m| { m.insert(v, val); });
                                                                    }
                                                                />
                                                            </div>
                                                        </li>
                                                    }
                                                }
                                            />
                                        </ul>
                                    </div>
                                </div>
                            </div>
                            {move || show_send_back.get().then(|| view! {
                                <div style="text-align:left;">
                                    <div class="flex-btn" style="margin-left:2ex;margin-bottom:-1ex;">
                                        <button class="game-btn back-btn" on:click=on_wrong_trade>"Back"</button>
                                    </div>
                                </div>
                            })}
                        </div>
                    })}

                    // Trade Offer
                    {move || (action_view.get() == ActionView::TradeOffer).then(|| view! {
                        <div style="margin-top:0.5rem;">
                            <h5>"Trade Offer"</h5>
                            <div style="padding-top:0.5rem;">
                                <label class="field-label">"Opponent"</label>
                                <select
                                    class="text-field"
                                    prop:value=move || trade_opponent.get()
                                    on:change=move |e| {
                                        let v = e.target().unwrap()
                                            .unchecked_into::<web_sys::HtmlSelectElement>()
                                            .value();
                                        trade_opponent.set(v);
                                    }
                                >
                                    <For
                                        each=move || trade_opponents.get()
                                        key=|p| p.name.clone()
                                        children=|p| view! {
                                            <option value={p.name.clone()}>{p.name.clone()}</option>
                                        }
                                    />
                                </select>
                            </div>
                            <div style="padding-top:0.5rem;">
                                <label class="field-label">"Animal"</label>
                                <select
                                    class="text-field"
                                    prop:value=move || trade_animal_id.get()
                                    on:change=move |e| {
                                        let v = e.target().unwrap()
                                            .unchecked_into::<web_sys::HtmlSelectElement>()
                                            .value();
                                        trade_animal_id.set(v);
                                    }
                                >
                                    <For
                                        each=move || trade_animals.get()
                                        key=|(id, _, _)| id.clone()
                                        children=|(id, name, count)| view! {
                                            <option value={id.clone()}>
                                                {name}" "{count}"×"
                                            </option>
                                        }
                                    />
                                </select>
                            </div>
                            <div class="flex-btn">
                                <button class="game-btn" on:click=on_offer>"Offer"</button>
                            </div>
                            {move || show_offer_back.get().then(|| view! {
                                <div style="text-align:left;">
                                    <div class="flex-btn" style="margin-left:2ex;margin-bottom:-1ex;">
                                        <button class="game-btn back-btn" on:click=on_offer_back>"Back"</button>
                                    </div>
                                </div>
                            })}
                        </div>
                    })}

                    // Trade Response
                    {move || (action_view.get() == ActionView::TradeResponse).then(|| view! {
                        <div style="margin-top:0.5rem;">
                            <h5>"Trade Response"</h5>
                            {move || trade_offer_data.get().map(|o| {
                                let animal_name = animal_mapping
                                    .get_untracked()
                                    .get(&o.animal.value)
                                    .cloned()
                                    .unwrap_or_else(|| o.animal.value.to_string());
                                view! {
                                    <div>"Offering player: "<strong>{o.challenger.clone()}</strong></div>
                                    <div>"Number of cards: "<strong>{o.challenger_card_offer}</strong></div>
                                    <div>"Auction item: "<strong>{animal_name}" "{o.animal_count}"×"</strong></div>
                                }
                            })}
                            <div class="flex-btn" style="padding-right:4rem;">
                                <button class="game-btn" on:click=on_accept_trade>"Accept"</button>
                            </div>
                            <div class="flex-btn">
                                <button class="game-btn" on:click=on_counter_trade>"Counteroffer"</button>
                            </div>
                        </div>
                    })}
                </div>
            })}

            // ── Lost connection ────────────────────────────────────────────────
            {move || (screen.get() == Screen::LostConnection).then(|| view! {
                <div class="box">
                    <div class="box-inner" style="text-align:center;">
                        <div style="padding-top:0.5rem;">"Connection closed! Please connect to a new game."</div>
                    </div>
                </div>
            })}

            // ── Error ──────────────────────────────────────────────────────────
            {move || (screen.get() == Screen::Error).then(|| view! {
                <div class="box">
                    <div class="box-inner" style="text-align:center;">
                        <div style="padding-top:0.5rem;">"Error: no connection possible! Please try again."</div>
                    </div>
                </div>
            })}

        </div>
    }
}

// ── GameUpdate handler (free function to reduce closure size) ─────────────────

#[allow(clippy::too_many_arguments)]
fn handle_game_update(
    update: GameUpdate,
    my_player_id: String,
    screen: RwSignal<Screen>,
    action_view: RwSignal<ActionView>,
    deck_count: RwSignal<usize>,
    current_card: RwSignal<Option<(String, usize)>>,
    animals: RwSignal<Vec<AnimalInfo>>,
    players: RwSignal<Vec<PlayerInfo>>,
    banknotes: RwSignal<Vec<(usize, usize)>>,
    highest_bid: RwSignal<Option<usize>>,
    highest_bidder: RwSignal<Option<String>>,
    animal_mapping: RwSignal<HashMap<usize, String>>,
    inverse_animal_mapping: RwSignal<HashMap<String, usize>>,
    message_header: RwSignal<String>,
    message_body: RwSignal<String>,
    ws: RwSignal<Option<SendWrapper<WebSocket>>>,
) {
    match update {
        GameUpdate::Start {
            wallet,
            players_in_turn_order,
            animals: animal_sets,
        } => {
            // build animal name mapping
            let mapping = resolve_animal_name(&animal_sets);
            let mut inv: HashMap<String, usize> = HashMap::new();
            for (v, n) in &mapping {
                inv.insert(n.clone(), *v);
            }

            let mut total_deck = 0usize;
            let animal_infos: Vec<AnimalInfo> = animal_sets
                .iter()
                .map(|s| {
                    let count = s.animals.len();
                    total_deck += count;
                    AnimalInfo {
                        name: mapping.get(&s.animal.value).cloned().unwrap_or_default(),
                        value: s.animal.value,
                        count,
                    }
                })
                .collect();

            let money_card_count: usize = wallet.bank_notes.iter().map(|(_, c)| c).sum();
            let player_infos: Vec<PlayerInfo> = players_in_turn_order
                .iter()
                .map(|pid| PlayerInfo {
                    name: pid.clone(),
                    money_count: money_card_count,
                    animals: vec![],
                })
                .collect();

            animal_mapping.set(mapping);
            inverse_animal_mapping.set(inv);
            deck_count.set(total_deck);
            animals.set(animal_infos);
            players.set(player_infos);
            banknotes.set(wallet.bank_notes.clone());
            screen.set(Screen::Game);

            // acknowledge
            send_ws(
                ws,
                &ActionMessage::NoAction {
                    decision: NoActionDecision::Ok(),
                },
            );
        }

        GameUpdate::Auction(kind) => {
            deck_count.update(|n| *n = n.saturating_sub(1));

            match kind {
                AuctionKind::NoBiddings { host_id, animal } => {
                    // host gets the animal
                    players.update(|ps| {
                        if let Some(p) = ps.iter_mut().find(|p| p.name == host_id) {
                            let name = animal_mapping
                                .get_untracked()
                                .get(&animal.value)
                                .cloned()
                                .unwrap_or_default();
                            if let Some(a) = p.animals.iter_mut().find(|a| a.name == name) {
                                a.count += 1;
                            } else {
                                p.animals.push(AnimalInfo {
                                    name,
                                    value: animal.value,
                                    count: 1,
                                });
                            }
                        }
                    });
                    animals.update(|list| {
                        if let Some(a) = list.iter_mut().find(|a| a.value == animal.value) {
                            a.count = a.count.saturating_sub(1);
                        }
                    });
                }

                AuctionKind::NormalAuction {
                    from,
                    to,
                    rounds,
                    money_transfer,
                } => {
                    // transfer animal from deck to `from`
                    let animal_val = rounds.animal.value;
                    animals.update(|list| {
                        if let Some(a) = list.iter_mut().find(|a| a.value == animal_val) {
                            a.count = a.count.saturating_sub(1);
                        }
                    });
                    players.update(|ps| {
                        let name = animal_mapping
                            .get_untracked()
                            .get(&animal_val)
                            .cloned()
                            .unwrap_or_default();
                        if let Some(p) = ps.iter_mut().find(|p| p.name == from) {
                            if let Some(a) = p.animals.iter_mut().find(|a| a.name == name) {
                                a.count += 1;
                            } else {
                                p.animals.push(AnimalInfo {
                                    name: name.clone(),
                                    value: animal_val,
                                    count: 1,
                                });
                            }
                        }
                        // update money card counts
                        let paid_cards = match &money_transfer {
                            MoneyTransfer::Public { card_amount, .. } => *card_amount,
                            MoneyTransfer::Private { amount } => amount.len(),
                        };
                        if let Some(p) = ps.iter_mut().find(|p| p.name == from) {
                            p.money_count = p.money_count.saturating_sub(paid_cards);
                        }
                        if let Some(p) = ps.iter_mut().find(|p| p.name == to) {
                            p.money_count += paid_cards;
                        }
                        // update my wallet if private
                        if from == my_player_id || to == my_player_id {
                            if let MoneyTransfer::Private { amount } = &money_transfer {
                                let amount = amount.clone();
                                let bn = banknotes.get_untracked();
                                let mut map: HashMap<usize, usize> = bn.into_iter().collect();
                                if from == my_player_id {
                                    for v in &amount {
                                        let e = map.entry(*v).or_insert(0);
                                        *e = e.saturating_sub(1);
                                        if map[v] == 0 {
                                            map.remove(v);
                                        }
                                    }
                                } else {
                                    for v in &amount {
                                        *map.entry(*v).or_insert(0) += 1;
                                    }
                                }
                                banknotes.set(map.into_iter().collect());
                            }
                        }
                    });

                    // show message about the auction result
                    let animal_name = animal_mapping
                        .get_untracked()
                        .get(&animal_val)
                        .cloned()
                        .unwrap_or_else(|| animal_val.to_string());
                    let body = format!(
                        "<strong>{from}</strong> bought <strong>{animal_name}</strong> from <strong>{to}</strong>."
                    );
                    message_header.set("Auction Result".to_string());
                    message_body.set(body);
                    action_view.set(ActionView::Message);
                    return; // don't send ok here — user clicks Ok button
                }
            }

            send_ws(
                ws,
                &ActionMessage::NoAction {
                    decision: NoActionDecision::Ok(),
                },
            );
        }

        GameUpdate::Trade {
            challenger,
            opponent,
            animal,
            animal_count,
            receiver,
            money_trade,
        } => {
            let animal_name = animal_mapping
                .get_untracked()
                .get(&animal.value)
                .cloned()
                .unwrap_or_else(|| animal.value.to_string());

            // the receiver gets the animal cards
            players.update(|ps| {
                // loser loses animals, winner gains them
                let loser = if receiver == challenger {
                    &opponent
                } else {
                    &challenger
                };
                let winner = &receiver;

                if let Some(p) = ps.iter_mut().find(|p| &p.name == loser) {
                    if let Some(a) = p.animals.iter_mut().find(|a| a.name == animal_name) {
                        a.count = a.count.saturating_sub(animal_count);
                    }
                }
                if let Some(p) = ps.iter_mut().find(|p| &p.name == winner) {
                    if let Some(a) = p.animals.iter_mut().find(|a| a.name == animal_name) {
                        a.count += animal_count;
                    } else {
                        p.animals.push(AnimalInfo {
                            name: animal_name.clone(),
                            value: animal.value,
                            count: animal_count,
                        });
                    }
                }

                // update money counts from public info
                let (chal_cards, opp_cards) = match &money_trade {
                    MoneyTrade::Public {
                        challenger_card_offer,
                        opponent_card_offer,
                    } => (*challenger_card_offer, opponent_card_offer.unwrap_or(0)),
                    MoneyTrade::Private {
                        challenger_card_offer,
                        opponent_card_offer,
                    } => (
                        challenger_card_offer.len(),
                        opponent_card_offer.as_ref().map_or(0, |v| v.len()),
                    ),
                };
                if let Some(p) = ps.iter_mut().find(|p| p.name == challenger) {
                    p.money_count = p.money_count.saturating_sub(chal_cards);
                }
                if let Some(p) = ps.iter_mut().find(|p| p.name == opponent) {
                    p.money_count = p.money_count.saturating_sub(opp_cards);
                }
                if let Some(p) = ps.iter_mut().find(|p| p.name == challenger) {
                    p.money_count += opp_cards;
                }
                if let Some(p) = ps.iter_mut().find(|p| p.name == opponent) {
                    p.money_count += chal_cards;
                }

                // update my wallet if private
                if challenger == my_player_id || opponent == my_player_id {
                    if let MoneyTrade::Private {
                        challenger_card_offer,
                        opponent_card_offer,
                    } = &money_trade
                    {
                        let (i_paid, i_received) = if challenger == my_player_id {
                            (
                                challenger_card_offer.clone(),
                                opponent_card_offer.clone().unwrap_or_default(),
                            )
                        } else {
                            (
                                opponent_card_offer.clone().unwrap_or_default(),
                                challenger_card_offer.clone(),
                            )
                        };
                        let bn = banknotes.get_untracked();
                        let mut map: HashMap<usize, usize> = bn.into_iter().collect();
                        for v in &i_paid {
                            let e = map.entry(*v).or_insert(0);
                            *e = e.saturating_sub(1);
                            if map[v] == 0 {
                                map.remove(v);
                            }
                        }
                        for v in &i_received {
                            *map.entry(*v).or_insert(0) += 1;
                        }
                        banknotes.set(map.into_iter().collect());
                    }
                }
            });

            let body = format!(
                "<strong>{challenger}</strong> traded <strong>{animal_name} {animal_count}×</strong> with <strong>{opponent}</strong>. <strong>{receiver}</strong> keeps the animals."
            );
            message_header.set("Trade Result".to_string());
            message_body.set(body);
            action_view.set(ActionView::Message);
        }

        GameUpdate::End { ranking, .. } => {
            let mut body = "<ol>".to_string();
            for (pid, pts) in &ranking {
                body.push_str(&format!("<li><strong>{pid}</strong>: {pts} pts</li>"));
            }
            body.push_str("</ol>");
            message_header.set("Game Over".to_string());
            message_body.set(body);
            action_view.set(ActionView::Message);
        }

        GameUpdate::ExposePlayer { player, wallet } => {
            players.update(|ps| {
                if let Some(p) = ps.iter_mut().find(|p| p.name == player) {
                    p.money_count = wallet.bank_notes.iter().map(|(_, c)| c).sum();
                }
            });
            if player == my_player_id {
                banknotes.set(wallet.bank_notes);
            }
            send_ws(
                ws,
                &ActionMessage::NoAction {
                    decision: NoActionDecision::Ok(),
                },
            );
        }

        GameUpdate::Inflation(money) => {
            banknotes.update(|bn| {
                if let Some(entry) = bn.iter_mut().find(|(v, _)| *v == money) {
                    entry.1 += 1;
                } else {
                    bn.push((money, 1));
                    bn.sort_by_key(|(v, _)| *v);
                }
            });
            send_ws(
                ws,
                &ActionMessage::NoAction {
                    decision: NoActionDecision::Ok(),
                },
            );
        }
    }
}

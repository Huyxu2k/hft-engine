use crate::types::{OrderID, Price, Trade, U64};
use std::collections::{BTreeMap, HashMap};

pub struct Order {
    id: OrderID,
    shares: U64,
    prev: Option<usize>,
    next: Option<usize>,
    limit_parent: Price,
    is_buy: bool,
}

pub struct Limit {
    total_volume: U64,
    head: Option<usize>,
    tail: Option<usize>,
}

pub struct OrderBook {
    orders_arena: Vec<Order>,
    id_map: HashMap<OrderID, usize>,
    buy_limits: BTreeMap<Price, Limit>,
    sell_limits: BTreeMap<Price, Limit>,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            orders_arena: Vec::with_capacity(100_000),
            id_map: HashMap::with_capacity(100_000),
            buy_limits: BTreeMap::new(),
            sell_limits: BTreeMap::new(),
        }
    }

    pub fn add_order(&mut self, id: OrderID, price: Price, shares: U64, is_buy: bool) {
        let limits = if is_buy {
            &mut self.buy_limits
        } else {
            &mut self.sell_limits
        };

        let limit = limits.entry(price).or_insert(Limit {
            total_volume: U64::zero(),
            head: None,
            tail: None,
        });

        let new_idx = self.orders_arena.len();
        let order = Order {
            id,
            shares,
            prev: limit.tail,
            next: None,
            limit_parent: price,
            is_buy,
        };

        if let Some(old_tail_idx) = limit.tail {
            self.orders_arena[old_tail_idx].next = Some(new_idx);
        } else {
            limit.head = Some(new_idx);
        }

        limit.tail = Some(new_idx);
        limit.total_volume += shares;

        self.orders_arena.push(order);
        self.id_map.insert(id, new_idx);
    }

    pub fn cancel_order(&mut self, id: OrderID) {
        let Some(&idx) = self.id_map.get(&id) else {
            return;
        };

        let order = &self.orders_arena[idx];
        let price = order.limit_parent;
        let shares = order.shares;
        let prev_idx = order.prev;
        let next_idx = order.next;
        let is_buy = order.is_buy;

        let limits = if is_buy {
            &mut self.buy_limits
        } else {
            &mut self.sell_limits
        };

        if let Some(limit) = limits.get_mut(&price) {
            if let Some(p) = prev_idx {
                self.orders_arena[p].next = next_idx;
            } else {
                limit.head = next_idx;
            }

            if let Some(n) = next_idx {
                self.orders_arena[n].prev = prev_idx;
            } else {
                limit.tail = prev_idx;
            }

            limit.total_volume -= shares;

            if limit.head.is_none() {
                limits.remove(&price);
            }
        }

        self.id_map.remove(&id);
    }

    pub fn execute_match(
        &mut self,
        taker_id: OrderID,
        price: Price,
        mut shares_left: U64,
        is_buy: bool,
    ) -> Vec<Trade> {
        let mut trades = Vec::new();

        if is_buy {
            while shares_left > U64::zero() {
                let Some(best_price) = self.sell_limits.keys().next().copied() else {
                    break;
                };
                if best_price > price {
                    break;
                }

                let maker_idx_opt = self.sell_limits.get(&best_price).and_then(|l| l.head);
                let Some(maker_idx) = maker_idx_opt else {
                    self.sell_limits.remove(&best_price);
                    continue;
                };

                let traded = std::cmp::min(shares_left, self.orders_arena[maker_idx].shares);
                let maker_id = self.orders_arena[maker_idx].id;
                self.orders_arena[maker_idx].shares -= traded;
                shares_left -= traded;

                trades.push(Trade {
                    maker_id,
                    taker_id,
                    price: best_price,
                    shares: traded,
                });

                let mut remove_level = false;
                if let Some(limit) = self.sell_limits.get_mut(&best_price) {
                    limit.total_volume -= traded;

                    if self.orders_arena[maker_idx].shares == U64::zero() {
                        let next = self.orders_arena[maker_idx].next;
                        limit.head = next;
                        if let Some(n) = next {
                            self.orders_arena[n].prev = None;
                        } else {
                            limit.tail = None;
                        }
                        self.id_map.remove(&maker_id);
                    }
                    remove_level = limit.head.is_none();
                }
                if remove_level {
                    self.sell_limits.remove(&best_price);
                }
            }
        } else {
            while shares_left > U64::zero() {
                let Some(best_price) = self.buy_limits.keys().next_back().copied() else {
                    break;
                };
                if best_price < price {
                    break;
                }

                let maker_idx_opt = self.buy_limits.get(&best_price).and_then(|l| l.head);
                let Some(maker_idx) = maker_idx_opt else {
                    self.buy_limits.remove(&best_price);
                    continue;
                };

                let traded = std::cmp::min(shares_left, self.orders_arena[maker_idx].shares);
                let maker_id = self.orders_arena[maker_idx].id;
                self.orders_arena[maker_idx].shares -= traded;
                shares_left -= traded;

                trades.push(Trade {
                    maker_id,
                    taker_id,
                    price: best_price,
                    shares: traded,
                });

                let mut remove_level = false;
                if let Some(limit) = self.buy_limits.get_mut(&best_price) {
                    limit.total_volume -= traded;

                    if self.orders_arena[maker_idx].shares == U64::zero() {
                        let next = self.orders_arena[maker_idx].next;
                        limit.head = next;
                        if let Some(n) = next {
                            self.orders_arena[n].prev = None;
                        } else {
                            limit.tail = None;
                        }
                        self.id_map.remove(&maker_id);
                    }
                    remove_level = limit.head.is_none();
                }
                if remove_level {
                    self.buy_limits.remove(&best_price);
                }
            }
        }

        trades
    }
}

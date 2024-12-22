//! The Lr grammar class.

#![allow(missing_docs)]

use std::collections::{BTreeMap, VecDeque};
use std::rc::Rc;

use cfg_grammar::symbol_bit_set::SymbolBitSet;
use cfg_grammar::Cfg;
use cfg_history::RootHistoryNode;
use cfg_symbol::Symbol;

type Dot = u32;
type RuleId = u32;
type SetId = u32;

/// A container of LR(0) items.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Lr0Items {
    pub map: BTreeMap<RuleId, Lr0Item>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Lr0Item {
    pub rhs: Vec<Symbol>,
    pub dot: Dot,
}

/// A builder for an LR(0) item closure.
pub struct Lr0ClosureBuilder<'a> {
    grammar: &'a mut Cfg,
    queue: VecDeque<Lr0Item>,
    terminal_set: SymbolBitSet,
}

/// Builder of LR(0) Finite State Machine.
pub struct Lr0FsmBuilder<'a> {
    closure: Lr0ClosureBuilder<'a>,
    sets_queue: VecDeque<Rc<Lr0Items>>,
    cached_sets: BTreeMap<Rc<Lr0Items>, u32>,
}

/// An LR(0) node.
#[derive(Debug, Eq, PartialEq)]
pub struct Lr0Node {
    /// List of LR(0) items.
    pub items: Rc<Lr0Items>,
    /// List of transitions through terminals.
    pub link: BTreeMap<Symbol, SetId>,
}

impl Lr0Items {
    fn new() -> Self {
        Lr0Items {
            map: BTreeMap::new(),
        }
    }
}

impl<'a> Lr0ClosureBuilder<'a> {
    /// Creates a builder for an LR(0) item closure.
    pub fn new(grammar: &'a mut Cfg) -> Self {
        Lr0ClosureBuilder {
            queue: VecDeque::new(),
            terminal_set: grammar.terminal_symbols(),
            grammar,
        }
    }

    /// We compute the closure for LR(0) items.
    pub fn closure(&mut self, items: &mut Lr0Items) {
        self.queue.clear();
        self.queue.extend(items.map.values().cloned());

        while let Some(item) = self.queue.pop_front() {
            if let Some(nonterminal_postdot) = self.nonterminal_postdot(&item) {
                for (rule_idx, rule) in self.grammar.rules().enumerate() {
                    if rule.lhs == nonterminal_postdot {
                        let new_item = Lr0Item {
                            rhs: rule.rhs.to_vec(),
                            dot: 0,
                        };
                        if items
                            .map
                            .insert(rule_idx as RuleId, new_item.clone())
                            .is_none()
                        {
                            self.queue.push_back(new_item);
                        }
                    }
                }
            }
        }
    }

    /// Advances the items by a symbol.
    pub fn advance(&mut self, items: &Lr0Items, sym: Symbol) -> Option<Lr0Items> {
        let mut new_items = Lr0Items::new();
        for (&rule_id, item) in items.map.iter() {
            if let Some(postdot) = self.postdot(item) {
                if postdot == sym {
                    new_items.map.insert(
                        rule_id,
                        Lr0Item {
                            rhs: item.rhs.clone(),
                            dot: item.dot + 1,
                        },
                    );
                }
            }
        }
        if new_items.map.is_empty() {
            None
        } else {
            self.closure(&mut new_items);
            Some(new_items)
        }
    }

    fn postdot(&self, item: &Lr0Item) -> Option<Symbol> {
        item.rhs.get(item.dot as usize).cloned()
    }

    fn nonterminal_postdot(&self, item: &Lr0Item) -> Option<Symbol> {
        match item.rhs.get(item.dot as usize) {
            Some(&postdot) => {
                if !self.terminal_set[postdot] {
                    Some(postdot)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl<'a> Lr0FsmBuilder<'a> {
    /// Creates a new LR(0) Finite State Machine builder.
    pub fn new(grammar: &'a mut Cfg) -> Self {
        Lr0FsmBuilder {
            closure: Lr0ClosureBuilder::new(grammar),
            sets_queue: VecDeque::new(),
            cached_sets: BTreeMap::new(),
        }
    }

    /// Construct an LR(0) Finite State Machine.
    pub fn make_lr0_fsm(&mut self, start_sym: Symbol) -> Vec<Lr0Node> {
        self.cached_sets.clear();
        self.sets_queue.clear();

        let initial_item_set = self.initial_item_set(start_sym);
        self.introduce_set(initial_item_set, 0);
        let mut result = vec![];
        while let Some(items) = self.sets_queue.pop_front() {
            let mut link = BTreeMap::new();
            let terminals: Vec<Symbol> = self.closure.terminal_set.iter().collect();
            for terminal in terminals {
                if let Some(advanced_set) = self.closure.advance(&items, terminal) {
                    let id = self.id_of(Rc::new(advanced_set));
                    link.insert(terminal, id);
                }
            }
            result.push(Lr0Node { items, link })
        }
        result
    }

    fn initial_item_set(&mut self, start_sym: Symbol) -> Rc<Lr0Items> {
        let (_new_start, new_start_rule_id) = self.augment_grammar(start_sym);
        let initial_item = Lr0Item {
            rhs: vec![start_sym],
            dot: 0,
        };
        let mut initial_item_set = Lr0Items::new();
        initial_item_set.map.insert(new_start_rule_id, initial_item);
        self.closure.closure(&mut initial_item_set);
        Rc::new(initial_item_set)
    }

    fn augment_grammar(&mut self, start_sym: Symbol) -> (Symbol, RuleId) {
        let new_start = self.closure.grammar.next_sym();
        let rule_id = self.closure.grammar.rules().count() as RuleId;
        let history_id = self
            .closure
            .grammar
            .add_history_node(RootHistoryNode::NoOp.into());
        self.closure
            .grammar
            .rule(new_start)
            .rhs_with_history([start_sym], history_id);
        (new_start, rule_id)
    }

    fn id_of(&mut self, items: Rc<Lr0Items>) -> SetId {
        match self.cached_sets.get(&items) {
            None => {
                let id = self.cached_sets.len() as SetId;
                self.introduce_set(items, id);
                id
            }
            Some(&id) => id,
        }
    }

    fn introduce_set(&mut self, items: Rc<Lr0Items>, id: SetId) {
        self.cached_sets.insert(items.clone(), id);
        self.sets_queue.push_back(items);
    }
}

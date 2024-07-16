use std::{
    collections::{BTreeMap, HashMap},
    ops::Bound,
};

use ibc_core::{client::{context::client_state::ClientState, types::Height}, primitives::Timestamp};
use tendermint::Time;

use crate::context::ClientType;

pub enum Direction {
    Next,
    Prev,
}

#[derive(Clone)]
pub struct Storage<C: ClientType> {
    pub current_height: Option<Height>,
    pub client_state: Option<C::ClientState>,
    pub consensus_state: HashMap<String, C::ConsensusState>,
    pub consensus_state_height_map: BTreeMap<Height, C::ConsensusState>,
    pub update_meta: HashMap<Height, (Timestamp, Height)>
}

impl <C: ClientType> Default for Storage<C> {
    fn default() -> Self {
        Self {
            current_height: None,
            client_state: None,
            consensus_state: HashMap::new(),
            consensus_state_height_map: BTreeMap::new(),
            update_meta: HashMap::new()
        }
    }
}

impl <C: ClientType> Storage<C> {
    pub fn get_heights(&self) -> Vec<Height> {
        self.consensus_state_height_map.keys().cloned().collect()
    }

    pub fn get_adjacent_height(
        &self,
        current: &Height,
        direction: Direction
    ) -> Option<C::ConsensusState> {
        match direction {
            Direction::Next => {
                let mut it = self
                .consensus_state_height_map
                .range((Bound::Included(current), Bound::Unbounded));
            it.next().map(|(_, s)| s.to_owned())
            }
            Direction::Prev => {
                let mut it = self
                .consensus_state_height_map
                .range((Bound::Unbounded, Bound::Included(current)));
            it.next().map(|(_, s)| s.to_owned())
            }
        }
    }
}


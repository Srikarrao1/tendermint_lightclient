use ibc_client_tendermint::{client_state::ClientState, consensus_state::ConsensusState};
use crate::context::ClientType;

pub struct Tendermintclient;

impl ClientType for Tendermintclient {
    type ClientState = ClientState;
    type ConsensusState = ConsensusState;
}
use ibc_client_tendermint::client_state::ClientState;
use ibc_core::client::context::client_state::ClientStateCommon;
use ibc_core::client::context::consensus_state::ConsensusState as ConsensusStateTrait;
use ibc_core::client::context::ExtClientValidationContext;

use ibc_core::client::types::error::ClientError;
use ibc_core::handler::types::error::ContextError;

use ibc_core::{
    client:: {
        context:: {
            client_state::ClientStateExecution, ClientExecutionContext, ClientValidationContext
        },
        types::Height
    },
    host::types::identifiers::ClientId,
};

use tendermint::chain::id;
use tendermint::consensus::state;
use tendermint::Time;

use crate::storage::{Direction, Storage};

pub struct Ctx<C: ClientType> {
    storage: Storage<C>
}

impl <C: ClientType> Default for Ctx<C> {
    fn default() -> Self {
        Self { storage: Storage::default() }
    }
}

pub trait ClientType: Sized {
    type ClientState: ClientStateExecution<Ctx<Self>> + Clone;
    type ConsensusState: ConsensusStateTrait + Clone;
}


impl <C: ClientType> ClientValidationContext for Ctx<C> {
    type ClientStateRef = C::ClientState;
    type ConsensusStateRef = C::ConsensusState;

    fn client_state(&self, _client_id: &ClientId) -> Result<Self::ClientStateRef, ContextError> {
        Ok(self.storage.client_state.clone().unwrap())
    }

    fn consensus_state(
            &self,
            client_cons_state_path: &ibc_core::host::types::path::ClientConsensusStatePath,
        ) -> Result<Self::ConsensusStateRef, ContextError> {
        let cons_state = self
        .storage
        .consensus_state
        .get(&client_cons_state_path.leaf());

    match cons_state {
        Some(state) => Ok(state.to_owned()),
        None => Err(ContextError::ClientError(
            ibc_core::client::types::error::ClientError::ConsensusStateNotFound { 
                client_id: client_cons_state_path.clone().client_id, 
                height: Height::new(client_cons_state_path.revision_number, client_cons_state_path.revision_height).unwrap(), },
        )),
      }
    }
    
    fn client_update_meta(
            &self,
            client_id: &ClientId,
            height: &Height,
        ) -> Result<(ibc_core::primitives::Timestamp, Height), ContextError> {
            match self.storage.update_meta.get(height) {
                Some(meta) => Ok(meta.to_owned()),
                None => Err(ClientError::UpdateMetaDataNotFound { client_id: client_id.clone(), height: *height }
              .into()),
            }
    }
}

impl<C: ClientType> ClientExecutionContext for Ctx<C> {

    fn client_state_mut(&self, client_id: &ibc_core::host::types::identifiers::ClientId) -> Result<Self::ClientStateMut, ContextError> {
        self.client_state(client_id)
    }

    type ClientStateMut = C::ClientState;

    fn store_client_state(
            &mut self,
            _client_state_path: ibc_core::host::types::path::ClientStatePath,
            client_state: Self::ClientStateRef,
        ) -> Result<(), ContextError> {
        self.storage.client_state = Some(client_state);
        Ok(())
    }

    fn store_consensus_state(
            &mut self,
            consensus_state_path: ibc_core::host::types::path::ClientConsensusStatePath,
            consensus_state: Self::ConsensusStateRef,
        ) -> Result<(), ContextError> {
        self.storage.consensus_state.insert(consensus_state_path.leaf(), consensus_state);
        Ok(())
    }

    fn delete_consensus_state(
            &mut self,
            consensus_state_path: ibc_core::host::types::path::ClientConsensusStatePath,
        ) -> Result<(), ContextError> {
        self.storage.consensus_state.remove(&consensus_state_path.leaf());
        Ok(())
    }

    fn store_update_meta(
            &mut self,
            _client_id: ibc_core::host::types::identifiers::ClientId,
            height: Height,
            host_timestamp: ibc_core::primitives::Timestamp,
            host_height: Height,
        ) -> Result<(), ContextError> {
        self.storage.update_meta.insert(height, (host_timestamp, host_height));
        Ok(())
    }

    fn delete_update_meta(
            &mut self,
            _client_id: ibc_core::host::types::identifiers::ClientId,
            height: Height,
        ) -> Result<(), ContextError> {
        self.storage.update_meta.remove(&height);
        Ok(())
    }
} 

impl <C: ClientType> ExtClientValidationContext for Ctx<C> {
    fn host_timestamp(&self) -> Result<ibc_core::primitives::Timestamp, ContextError> {
        Ok(Time::now().into())
    }

    fn host_height(&self) -> Result<Height, ContextError> {
        let h = Height::new(0, 1)?;
        Ok(h)
    }

    fn consensus_state_heights(&self, _client_id: &ClientId) -> Result<Vec<Height>, ContextError> {
        Ok(self.storage.get_heights())
    }

    fn next_consensus_state(
            &self,
            _client_id: &ClientId,
            height: &Height,
        ) -> Result<Option<Self::ConsensusStateRef>, ContextError> {
        Ok(self.storage.get_adjacent_height(height, Direction::Next))
    }

    fn prev_consensus_state(
            &self,
            _client_id: &ClientId,
            height: &Height,
        ) -> Result<Option<Self::ConsensusStateRef>, ContextError> {
        Ok(self.storage.get_adjacent_height(height, Direction::Prev))
    }

}
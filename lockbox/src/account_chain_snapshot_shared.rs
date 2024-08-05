use std::{ops::Deref, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::account_chain_snapshot::AccountChainSnapshot;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct AccountChainSnapshotShared {
    inner: Arc<AccountChainSnapshot>,
}

impl From<AccountChainSnapshot> for AccountChainSnapshotShared {
    fn from(snapshot: AccountChainSnapshot) -> Self {
        Self {
            inner: Arc::new(snapshot),
        }
    }
}

impl Deref for AccountChainSnapshotShared {
    type Target = AccountChainSnapshot;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// SPDX-License-Identifier: AGPL-3.0-or-later

//! Bond ledger RPC operations — bridge from API types to core service.

use crate::error::ApiResult;
use crate::types::{
    BondLedgerListRequest, BondLedgerListResponse, BondLedgerRetrieveRequest,
    BondLedgerRetrieveResponse, BondLedgerStoreRequest, BondLedgerStoreResponse,
};

use super::LoamSpineRpcService;

impl LoamSpineRpcService {
    /// Store a bond record.
    ///
    /// # Errors
    ///
    /// Returns error if the core bond ledger store fails.
    pub async fn bond_ledger_store(
        &self,
        request: BondLedgerStoreRequest,
    ) -> ApiResult<BondLedgerStoreResponse> {
        let core = self.core().await;
        core.bond_ledger_store(request.bond_id, request.data)
            .await?;

        Ok(BondLedgerStoreResponse {
            status: "stored".into(),
        })
    }

    /// Retrieve a bond record by ID.
    ///
    /// # Errors
    ///
    /// Returns error on internal failure.
    pub async fn bond_ledger_retrieve(
        &self,
        request: BondLedgerRetrieveRequest,
    ) -> ApiResult<BondLedgerRetrieveResponse> {
        let core = self.core().await;
        let data = core.bond_ledger_retrieve(&request.bond_id).await;

        Ok(BondLedgerRetrieveResponse { data })
    }

    /// List all stored bond IDs.
    ///
    /// # Errors
    ///
    /// Returns error on internal failure.
    pub async fn bond_ledger_list(
        &self,
        _request: BondLedgerListRequest,
    ) -> ApiResult<BondLedgerListResponse> {
        let core = self.core().await;
        let bonds = core.bond_ledger_list().await;

        Ok(BondLedgerListResponse { bonds })
    }
}

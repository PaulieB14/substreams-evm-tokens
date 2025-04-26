use crate::pb::evm::erc1155::events::v1::{Transfer, Uri};
use substreams_abis::evm::token::erc1155::events::{TransferBatch, TransferSingle, Uri as UriEvent};
use substreams_ethereum::{pb::eth::v2 as eth, Event as _};

// Extracts ERC1155 Transfer events (both TransferSingle and TransferBatch)
pub fn get_transfers<'a>(blk: &'a eth::Block) -> impl Iterator<Item = Transfer> + 'a {
    blk.receipts().flat_map(move |receipt| {
        let hash = &receipt.transaction.hash;
        let contract = &receipt.transaction.to;
        receipt.receipt.logs.iter().flat_map(move |log| {
            // Try TransferSingle
            if let Some(event) = TransferSingle::match_and_decode(log) {
                let token_id = event.id.to_string();
                let amount = event.value.to_string();
                return vec![Transfer {
                    block_num: blk.number,
                    tx_hash: hash.clone(),
                    log_index: log.block_index as u64,
                    contract: contract.clone(),
                    operator: event.operator.clone(),
                    from: event.from.clone(),
                    to: event.to.clone(),
                    token_id,
                    amount,
                    uri: None,
                }]
                .into_iter();
            }
            // Try TransferBatch
            if let Some(event) = TransferBatch::match_and_decode(log) {
                return event
                    .ids
                    .iter()
                    .zip(event.values.iter())
                    .map(|(id, value)| Transfer {
                        block_num: blk.number,
                        tx_hash: hash.clone(),
                        log_index: log.block_index as u64,
                        contract: contract.clone(),
                        operator: event.operator.clone(),
                        from: event.from.clone(),
                        to: event.to.clone(),
                        token_id: id.to_string(),
                        amount: value.to_string(),
                        uri: None,
                    })
                    .collect::<Vec<_>>()
                    .into_iter();
            }
            // Not a transfer event
            Vec::new().into_iter()
        })
    })
}

// Extracts ERC1155 Uri events
pub fn get_uris<'a>(blk: &'a eth::Block) -> impl Iterator<Item = Uri> + 'a {
    blk.receipts().flat_map(move |receipt| {
        let contract = &receipt.transaction.to;
        receipt.receipt.logs.iter().filter_map(move |log| {
            if let Some(event) = UriEvent::match_and_decode(log) {
                Some(Uri {
                    block_num: blk.number,
                    tx_hash: receipt.transaction.hash.clone(),
                    log_index: log.block_index as u64,
                    contract: contract.clone(),
                    token_id: event.id.to_string(),
                    uri: event.value.clone(),
                })
            } else {
                None
            }
        })
    })
}

use proto::pb::evm::erc721::events::v1::Transfer;
use substreams_abis::evm::token::erc721;
use substreams_ethereum::pb::eth::v2 as eth;
use substreams_ethereum::Event as _;

pub fn get_transfers<'a>(blk: &'a eth::Block) -> impl Iterator<Item = Transfer> + 'a {
    blk.receipts().flat_map(move |receipt| {
        receipt.receipt.logs.iter().filter_map(move |log| {
            if let Some(event) = erc721::events::Transfer::match_and_decode(log) {
                Some(Transfer {
                    block_num: blk.number,
                    tx_hash: receipt.transaction.hash.to_vec().into(),
                    log_index: log.block_index as u64,
                    contract: log.address.to_vec().into(),
                    from: event.from.to_vec().into(),
                    to: event.to.to_vec().into(),
                    token_id: event.token_id.to_string(),
                    ..Default::default()
                })
            } else {
                None
            }
        })
    })
}

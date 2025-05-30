mod events;
use proto::pb::evm::erc721::events::v1::{Mints, Token};
use std::collections::{HashMap, HashSet};
use substreams::scalar::BigInt;
use substreams_abis::evm::token::erc721::functions;
use substreams_ethereum::{pb::eth::v2 as eth, rpc::RpcBatch};

/// Extracts mints with uri, symbol and name from the logs
#[substreams::handlers::map]
fn map_mints(blk: eth::Block) -> Result<Mints, substreams::errors::Error> {
    let mints: Vec<Token> = events::get_mints(&blk).collect();
    if mints.is_empty() {
        return Ok(Mints { tokens: vec![] });
    }

    let contracts = mints.iter().map(|m| &m.contract).collect::<HashSet<_>>().into_iter().collect::<Vec<_>>();
    let mut symbols: HashMap<&[u8], (Option<String>, Option<String>)> = HashMap::new();
    for contract_chunk in contracts.chunks(50) {
        let batch = contract_chunk.iter().fold(RpcBatch::new(), |batch, address| {
            batch.add(functions::Name {}, address.to_vec()).add(functions::Symbol {}, address.to_vec())
        });
        let responses = batch.execute().expect("failed to execute symbol rpc batch").responses;
        for (i, address) in contract_chunk.iter().enumerate() {
            let name = RpcBatch::decode::<String, functions::Name>(&responses[i * 2]);
            let symbol = RpcBatch::decode::<String, functions::Symbol>(&responses[i * 2 + 1]);
            symbols.insert(address.as_ref(), (symbol, name));
        }
    }

    let mut tokens = vec![];
    for chunk in mints.chunks(100) {
        let batch = chunk.iter().fold(RpcBatch::new(), |batch, mint| {
            let token_id = mint.token_id.parse::<BigInt>().expect("invalid token_id");
            batch.add(functions::TokenUri { token_id }, mint.contract.to_vec())
        });
        let responses = batch.execute().expect("failed to execute uri rpc batch").responses;
        for (i, mint) in chunk.into_iter().enumerate() {
            let uri = RpcBatch::decode::<String, functions::TokenUri>(&responses[i]);
            let (symbol, name) = symbols.get(&mint.contract.as_ref()).cloned().unwrap_or((None, None));
            tokens.push(Token {
                uri,
                symbol,
                name,
                contract: mint.contract.clone(),
                token_id: mint.token_id.clone(),
            });
        }
    }

    substreams::log::info!("{} contracts, {} mints", contracts.len(), tokens.len());

    Ok(Mints { tokens })
}

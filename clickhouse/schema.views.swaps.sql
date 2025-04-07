CREATE TABLE IF NOT EXISTS swaps (
   -- block --
   block_num            UInt32,
   block_hash           FixedString(66),
   timestamp            DateTime(0, 'UTC'),

   -- ordering --
   ordinal              UInt64, -- log.ordinal
   `index`              UInt64, -- relative index
   global_sequence      UInt64, -- latest global sequence (block_num << 32 + index)

   -- transaction --
   transaction_id       FixedString(66),

   -- call --
   caller               FixedString(42) COMMENT 'caller address', -- call.caller

   -- swaps --
   address              FixedString(42) COMMENT 'pool address', -- log.address
   sender               FixedString(42) COMMENT 'sender address',
   recipient            FixedString(42) COMMENT 'recipient address',
   amount0              Int256 COMMENT 'token0 amount',
   amount1              Int256 COMMENT 'token1 amount',
   -- TO-DO: add token0 and token1 addresses
   price                Float64 COMMENT 'computed price',
   exchange             LowCardinality(String) COMMENT 'exchange name', -- 'uniswap_v2' or 'uniswap_v3'
)
ENGINE = ReplacingMergeTree(global_sequence)
PRIMARY KEY (timestamp, block_num, `index`)
ORDER BY (timestamp, block_num, `index`);

-- Uniswap::V2::Pair:Swap --
CREATE MATERIALIZED VIEW IF NOT EXISTS uniswap_v2_swaps_mv
TO uniswap_swaps AS
SELECT
   block_num,
   block_hash,
   timestamp,
   ordinal,
   `index`,
   global_sequence,
   transaction_id,
   caller,
   address,
   sender,
   `to` AS recipient,
   amount0_in - amount0_out AS amount0,
   amount1_in - amount1_out AS amount1,
   abs((amount1_in - amount1_out) / (amount0_in - amount0_out)) AS price,
   'uniswap_v2' AS exchange
FROM uniswap_v2_swaps;

-- Uniswap::V3::Pool:Swap --
CREATE MATERIALIZED VIEW IF NOT EXISTS uniswap_v3_swaps_mv
TO uniswap_swaps AS
SELECT
   block_num,
   block_hash,
   timestamp,
   ordinal,
   `index`,
   global_sequence,
   transaction_id,
   caller,
   address,
   sender,
   recipient,
   amount0,
   amount1,
   pow(1.0001, tick) AS price,
   'uniswap_v3' AS exchange
FROM uniswap_v3_swaps;
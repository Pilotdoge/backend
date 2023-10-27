-- Your SQL goes here
-- store amm events
CREATE TABLE query_accounts (
     address text NOT NULL,
     claimable_amount numeric NOT NULL,
     query_time bigint NOT NULL,
     PRIMARY KEY (address)
);

CREATE TABLE claimed_accounts (
    address text NOT NULL,
    claimed_amount numeric NOT NULL,
    claimed_time bigint NOT NULL,
    PRIMARY KEY (address)
);

CREATE TABLE last_sync_block (
    block_number bigint NOT NULL,
    PRIMARY KEY (block_number)
);
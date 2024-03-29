create table if not exists ethereummainnet.eth_events (
    id          text not null constraint eth_events_pk primary key,
    blocknumber integer,
    blockhash   text,
    txhash      text,
    txindex     integer,
    logindex    integer,
    address     text,
    topic0      text,
    topic1      text,
    topic2      text,
    topic3      text,
    topic4      text,
    data        text,
    addressF    text,
    commitment  text,
    num_logs    integer
);
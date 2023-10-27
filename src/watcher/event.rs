use ethabi::{decode,Address, ParamType, Uint};
use web3::types::{H160, Log};

#[derive(Debug, Clone)]
pub struct ClaimEvent {
    pub address: Address,
    pub amount: Uint,
    pub claimed_time: Uint,
}

impl TryFrom<Log> for ClaimEvent {
    type Error = ethabi::Error;

    fn try_from(event: Log) -> Result<Self, Self::Error> {
        let dec_ev = decode(
            &[
                ParamType::Uint(256),
                ParamType::Uint(256),
            ],
            &event.data.0,
        )?;
        Ok(ClaimEvent {
            address: H160::from_slice(&event.topics[1].as_bytes()[12..]),
            amount: dec_ev[0].clone().into_uint().unwrap(),
            claimed_time: dec_ev[1].clone().into_uint().unwrap(),
        })
    }
}
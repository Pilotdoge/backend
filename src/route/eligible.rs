use std::collections::HashMap;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use actix_web::{HttpRequest, HttpResponse, web};
use bigdecimal::{BigDecimal, Zero};
use num::BigInt;
use num::bigint::ToBigInt;
use qstring::QString;
use rbatis::rbdc::decimal::Decimal;
use crate::server::AppState;
use serde::{Serialize, Deserialize};
use crate::db;
use crate::db::tables::QueryAccount;
use crate::route::BackendResponse;
use crate::route::err::BackendError;

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct EligibleResult {
    pub count: u32,
    pub gas: HashMap<String,String>,
}
#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct OrbiterEligibleResp {
    pub code: u32,
    pub msg: String,
    pub result: EligibleResult,
}

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct EligibleResp {
    pub eth_gas_cost: String,
    pub claimable_amount: String,
}

pub async fn get_eligible(data: web::Data<AppState>, req: HttpRequest)
                          -> actix_web::Result<HttpResponse> {
    if data.config.claim_start {
        return Ok(HttpResponse::BadRequest().finish());
    }
    let query_str = req.query_string();
    let qs = QString::from(query_str);
    let address = qs.get("address").unwrap_or("0");
    let now = SystemTime::now();
    let since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    let timestamp = since_epoch.as_secs();
    let base_url = "https://openapi.orbiter.finance/mainnet/v1/gas";
    let url = format!("{}?address={}", base_url, address);
    match reqwest::Client::new().get(url).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                let ret = resp.text().await.unwrap();
                let eligible_ret: OrbiterEligibleResp = serde_json::from_str(&ret).unwrap();
                let gas_eth_cost = eligible_ret.result.gas.get("ETH");
                if gas_eth_cost.is_none() {
                    let resp = BackendResponse {
                        code: BackendError::Ok,
                        error: None,
                        data: Some(EligibleResp {
                            eth_gas_cost: "0".to_string(),
                            claimable_amount: "0".to_string(),
                        })
                    };
                    return Ok(HttpResponse::Ok().json(resp));
                }
                let tokens_number_per_gas = BigDecimal::from_str(&data.config.tokens_number_per_gas).unwrap_or_default();
                let gas_eth_cost = BigDecimal::from_str(gas_eth_cost.unwrap()).unwrap_or_default();
                let eligible_min_gas = BigDecimal::from_str(&data.config.eligible_min_gas).unwrap_or_default();
                let claimable_amount = if gas_eth_cost.gt(&eligible_min_gas) && !tokens_number_per_gas.is_zero() {
                    (gas_eth_cost.clone() / tokens_number_per_gas).to_bigint().unwrap()
                } else {
                    BigInt::from(0)
                };

                if let Err(e) = db::save_query_account(data.db.clone(), QueryAccount {
                    address: address.to_string(),
                    claimable_amount: Decimal::from_str(&claimable_amount.to_string()).unwrap(),
                    query_time: timestamp as i64,
                }).await {
                    log::warn!("save_query_account failed ,{e}")
                };
                let resp = BackendResponse {
                    code: BackendError::Ok,
                    error: None,
                    data: Some(EligibleResp {
                        eth_gas_cost: gas_eth_cost.to_string(),
                        claimable_amount: claimable_amount.to_string(),
                    })
                };
                Ok(HttpResponse::Ok().json(resp))
            } else {
                let resp = BackendResponse {
                    code: BackendError::InternalErr,
                    error: Some("Orbiter api return failed".to_owned()),
                    data: None::<()>
                };
                Ok(HttpResponse::Ok().json(resp))
            }

        },
        Err(_e) => {
            let resp = BackendResponse {
                code: BackendError::InternalErr,
                error: Some("Orbiter api connected failed".to_owned()),
                data: None::<()>
            };
            Ok(HttpResponse::Ok().json(resp))
        }
    }
}

use std::str::FromStr;
use actix_web::{HttpRequest, HttpResponse, web};
use bigdecimal::BigDecimal;
use num::BigUint;
use crate::db;
use crate::route::BackendResponse;
use crate::route::err::BackendError;
use crate::server::AppState;

pub async fn get_queried_addresses_number(data: web::Data<AppState>, _req: HttpRequest)
                          -> actix_web::Result<HttpResponse> {
    match db::db_get_queried_addresses_number(&data.db).await {
        Ok(query_number) => {
            let resp = BackendResponse {
                code: BackendError::Ok,
                error: None,
                data: Some(query_number)
            };
            Ok(HttpResponse::Ok().json(resp))
        },
        Err(e) => {
            log::warn!("get_queried_addresses_number failed,{e}");
            let resp = BackendResponse {
                code: BackendError::InternalErr,
                error: Some("get_queried_addresses_number failed".to_owned()),
                data: None::<()>
            };
            Ok(HttpResponse::Ok().json(resp))
        }
    }
}

pub async fn get_total_claimed_number(data: web::Data<AppState>, _req: HttpRequest)
                                    -> actix_web::Result<HttpResponse> {
    match db::db_get_total_claimed_number(&data.db).await {
        Ok(query_number) => {
            let resp = BackendResponse {
                code: BackendError::Ok,
                error: None,
                data: Some(query_number)
            };
            Ok(HttpResponse::Ok().json(resp))
        },
        Err(e) => {
            log::warn!("get_total_claimed_number failed,{e}");
            let resp = BackendResponse {
                code: BackendError::InternalErr,
                error: Some("get_total_claimed_number failed".to_owned()),
                data: None::<()>
            };
            Ok(HttpResponse::Ok().json(resp))
        }
    }
}

pub async fn get_total_claimed_amount(data: web::Data<AppState>, _req: HttpRequest)
                                      -> actix_web::Result<HttpResponse> {
    match db::db_get_total_claimed_amount(&data.db).await {
        Ok(amount) => {
            let amount = BigDecimal::from_str(&amount.0.to_string()).unwrap();
            let resp = BackendResponse {
                code: BackendError::Ok,
                error: None,
                data: Some(amount.to_string())
            };
            Ok(HttpResponse::Ok().json(resp))
        },
        Err(e) => {
            log::warn!("get_total_claimed_amount failed,{e}");
            let resp = BackendResponse {
                code: BackendError::InternalErr,
                error: Some("get_total_claimed_amount failed".to_owned()),
                data: None::<()>
            };
            Ok(HttpResponse::Ok().json(resp))
        }
    }
}
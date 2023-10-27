use actix_web::{HttpRequest, HttpResponse, web};
use merkle_tree_rs::standard::LeafType;
use qstring::QString;
use serde::{Deserialize, Serialize};
use crate::route::BackendResponse;
use crate::route::err::BackendError;
use crate::server::AppState;

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct EligibleProofResp {
    pub address: String,
    pub amount: String,
    pub proof: Vec<String>,
}
pub async fn get_eligible_tree_root(data: web::Data<AppState>, _req: HttpRequest)
                                    -> actix_web::Result<HttpResponse> {
    if !data.config.claim_start {
        let resp = BackendResponse {
            code: BackendError::InvalidParameters,
            error: Some("claim not start".to_string()),
            data: None::<()>
        };
        return Ok(HttpResponse::Ok().json(resp));
    }

    let root = data.eligible_tree.as_ref().unwrap().lock().unwrap().root();
    let resp = BackendResponse {
        code: BackendError::Ok,
        error: None,
        data: Some(root)
    };
    Ok(HttpResponse::Ok().json(resp))
}

pub async fn get_eligible_proof(data: web::Data<AppState>, req: HttpRequest)
                                    -> actix_web::Result<HttpResponse> {
    if !data.config.claim_start {
        let resp = BackendResponse {
            code: BackendError::InvalidParameters,
            error: Some("claim not start".to_string()),
            data: None::<()>
        };
        return Ok(HttpResponse::Ok().json(resp));
    }

    let query_str = req.query_string();
    let qs = QString::from(query_str);
    let address = qs.get("address").unwrap_or("0");
    let tree = data.eligible_tree.as_ref().unwrap().lock().unwrap();
    for (i,v) in (*tree).clone().enumerate() {
        // println!("Value : {:?} {:?}", i,v);
        if v[0] == address {
            let proof = tree.get_proof(LeafType::Number(i));
            println!("Proof : {:?}", proof);
            let resp = BackendResponse {
                code: BackendError::Ok,
                error: None,
                data: Some(EligibleProofResp {
                    address: address.to_string(),
                    amount: v[1].to_string(),
                    proof,
                })
            };
            return Ok(HttpResponse::Ok().json(resp));
        }
    }

    //not found
    let resp = BackendResponse {
        code: BackendError::InvalidParameters,
        error: Some("account is not eligible".to_string()),
        data: None::<()>
    };
    Ok(HttpResponse::Ok().json(resp))
}
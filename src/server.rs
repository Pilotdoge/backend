use actix_web::{HttpServer, web};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use actix_web::App;
use std::thread;
use actix_cors::Cors;
use merkle_tree_rs::standard::StandardMerkleTree;
use crate::config::Config;
use crate::route::eligible::get_eligible;
use crate::route::merkle::{get_eligible_proof, get_eligible_tree_root};
use crate::route::stat::{get_queried_addresses_number, get_total_claimed_amount, get_total_claimed_number};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: rbatis::RBatis,
    pub eligible_tree: Option<Arc<Mutex<StandardMerkleTree>>>,
}

pub async fn run_server(app_state: AppState) {
    thread::Builder::new()
        .spawn(move || {
            actix_rt::System::new().block_on(async move {
                run_rpc_server(app_state).await
            });
        })
        .expect("failed to start endpoint server");

}

pub async fn run_rpc_server(app_state: AppState) {
    let works_number = app_state.config.workers;
    let bind_to = SocketAddr::new("0.0.0.0".parse().unwrap(),
                                  app_state.config.port as u16);
    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .app_data(web::Data::new(app_state.clone()))
            .route("/get_eligible", web::get().to(get_eligible))
            .route("/get_queried_addresses_number", web::get().to(get_queried_addresses_number))
            .route("/get_total_claimed_number", web::get().to(get_total_claimed_number))
            .route("/get_total_claimed_amount", web::get().to(get_total_claimed_amount))
            .route("/get_eligible_tree_root", web::get().to(get_eligible_tree_root))
            .route("/get_eligible_proof", web::get().to(get_eligible_proof))
    })
        .workers(works_number as usize)
        .bind(&bind_to)
        .expect("failed to bind")
        .run()
        .await
        .expect("failed to run endpoint server");
}
// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use tonic::transport::Server;
use log::*;
use std::env;

mod product_catalog_service;
use product_catalog_service::shop::product_catalog_service_server::ProductCatalogServiceServer;
use product_catalog_service::ProductCatalogServer;

mod telemetry;
use telemetry::{init_logger, init_reqwest_tracing, init_tracer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting Rust Product Catalog Service");
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<ProductCatalogServiceServer<ProductCatalogServer>>()
        .await;

    init_logger()?;
    init_reqwest_tracing(init_tracer()?)?;

    info!("OTel pipeline created");
    let port = env::var("PRODUCT_CATALOG_PORT").expect("$PRODUCT_CATALOG_PORT is not set");
    let addr = format!("0.0.0.0:{}", port).parse()?;
    info!("listening on {}", addr);
    
    let catalog_server = ProductCatalogServer::new().await?;

    Server::builder()
        .add_service(ProductCatalogServiceServer::new(catalog_server))
        .add_service(health_service)
        .serve(addr)
        .await?;

    Ok(())
}
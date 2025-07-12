// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use opentelemetry::trace::{SpanKind, TraceContextExt, Tracer};
use opentelemetry::{global, propagation::Extractor, trace::Span, Context, KeyValue};
use opentelemetry_semantic_conventions as semconv;
use shop::product_catalog_service_server::ProductCatalogService;
use shop::{Empty, GetProductRequest, ListProductsResponse, Product, SearchProductsRequest, SearchProductsResponse};
use tonic::{Request, Response, Status};
use std::collections::HashMap;
use std::env;
use std::time::Duration;
use tokio::time::interval;
use log::*;
use serde_json;
use open_feature::{OpenFeature, EvaluationContext, EvaluationOptions};
use open_feature_flagd::{FlagdProvider, FlagdOptions};
use std::sync::{Arc, RwLock};

// RPC constants taken from original Go implementation
const RPC_SYSTEM_GRPC: &'static str = "grpc";
const RPC_SERVICE_PRODUCT_CATALOG: &'static str = "oteldemo.ProductCatalogService";
const RPC_GRPC_STATUS_CODE_OK: i64 = 0;
const RPC_GRPC_STATUS_CODE_NOT_FOUND: i64 = 5;
const RPC_GRPC_STATUS_CODE_INTERNAL: i64 = 13;

pub mod shop {
    tonic::include_proto!("oteldemo");
}

pub struct ProductCatalogServer {
    products: Arc<RwLock<HashMap<String, Product>>>,
    products_list: Arc<RwLock<Vec<Product>>>,
    openfeature: OpenFeature,
}

struct MetadataMap<'a>(&'a tonic::metadata::MetadataMap);

impl<'a> Extractor for MetadataMap<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|metadata| metadata.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0
            .keys()
            .map(|key| match key {
                tonic::metadata::KeyRef::Ascii(v) => v.as_str(),
                tonic::metadata::KeyRef::Binary(v) => v.as_str(),
            })
            .collect::<Vec<_>>()
    }
}

impl ProductCatalogServer {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize OpenFeature with flagd provider (matching Go implementation)
        let mut openfeature = OpenFeature::default();
        let flagd_options = FlagdOptions::default();
        match FlagdProvider::new(flagd_options).await {
            Ok(flagd_provider) => {
                openfeature.set_provider(flagd_provider).await;
                info!("OpenFeature flagd provider initialized successfully");
            }
            Err(e) => {
                warn!("Failed to initialize flagd provider: {:?}, continuing without feature flags", e);
            }
        }
        
        let mut server = Self {
            products: Arc::new(RwLock::new(HashMap::new())),
            products_list: Arc::new(RwLock::new(Vec::new())),
            openfeature,
        };
        
        server.load_products().await?;
        server.start_reload_timer();
        
        Ok(server)
    }

    async fn load_products(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Loading Product Catalog...");
        
        let products_data = tokio::fs::read_to_string("./products/products.json").await?;
        let products_json: serde_json::Value = serde_json::from_str(&products_data)?;
        let products_array = products_json["products"].as_array().unwrap();
        
        let mut products = Vec::new();
        for product_value in products_array {
            let product = Product {
                id: product_value["id"].as_str().unwrap().to_string(),
                name: product_value["name"].as_str().unwrap().to_string(),
                description: product_value["description"].as_str().unwrap().to_string(),
                picture: product_value["picture"].as_str().unwrap().to_string(),
                price_usd: Some(shop::Money {
                    currency_code: product_value["priceUsd"]["currencyCode"].as_str().unwrap().to_string(),
                    units: product_value["priceUsd"]["units"].as_i64().unwrap(),
                    nanos: product_value["priceUsd"]["nanos"].as_i64().unwrap() as i32,
                }),
                categories: product_value["categories"].as_array().unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap().to_string())
                    .collect(),
            };
            products.push(product);
        }
        
        // Update main data structures to updated products
        {
            let mut products_map = self.products.write().unwrap();
            let mut products_list = self.products_list.write().unwrap();
            
            products_map.clear();
            *products_list = products;
            
            for product in products_list.iter() {
                products_map.insert(product.id.clone(), product.clone());
            }
            
            info!("Loaded {} products", products_list.len());
        }
        Ok(())
    }

    fn start_reload_timer(&self) {
        let reload_interval = env::var("PRODUCT_CATALOG_RELOAD_INTERVAL")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<u64>()
            .unwrap_or(10);
        
        info!("Product Catalog reload interval: {} seconds", reload_interval);
        
        let products = self.products.clone();
        let products_list = self.products_list.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(reload_interval));
            loop {
                interval.tick().await;
                info!("Reloading Product Catalog...");
                
                match Self::reload_products_from_disk(products.clone(), products_list.clone()).await {
                    Ok(_) => {},
                    Err(e) => {
                        warn!("Error reading product files: {}", e);
                        continue;
                    }
                }
            }
        });
    }

    async fn reload_products_from_disk(
        products: Arc<RwLock<HashMap<String, Product>>>,
        products_list: Arc<RwLock<Vec<Product>>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Re- read products from disk
        let products_data = tokio::fs::read_to_string("./products/products.json").await?;
        let products_json: serde_json::Value = serde_json::from_str(&products_data)?;
        let products_array = products_json["products"].as_array().unwrap();
        
        let mut new_products = Vec::new();
        for product_value in products_array {
            let product = Product {
                id: product_value["id"].as_str().unwrap().to_string(),
                name: product_value["name"].as_str().unwrap().to_string(),
                description: product_value["description"].as_str().unwrap().to_string(),
                picture: product_value["picture"].as_str().unwrap().to_string(),
                price_usd: Some(shop::Money {
                    currency_code: product_value["priceUsd"]["currencyCode"].as_str().unwrap().to_string(),
                    units: product_value["priceUsd"]["units"].as_i64().unwrap(),
                    nanos: product_value["priceUsd"]["nanos"].as_i64().unwrap() as i32,
                }),
                categories: product_value["categories"].as_array().unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap().to_string())
                    .collect(),
            };
            new_products.push(product);
        }
        
        // Update main data structures to updated products
        {
            let mut products_map = products.write().unwrap();
            let mut products_vec = products_list.write().unwrap();
            
            products_map.clear();
            *products_vec = new_products;
            
            for product in products_vec.iter() {
                products_map.insert(product.id.clone(), product.clone());
            }
            
            info!("Reloaded {} products", products_vec.len());
        }
        
        Ok(())
    }

    async fn check_product_failure(&self, _ctx: &Context, id: &str) -> bool {
        if id != "OLJCESPC7Z" {
            return false;
        }
        
        let client = self.openfeature.create_client();
        let evaluation_context = EvaluationContext::default();
        let evaluation_options = EvaluationOptions::default();
        
        match client.get_bool_value("productCatalogFailure", Some(&evaluation_context), Some(&evaluation_options)).await {
            Ok(evaluation_result) => evaluation_result,
            Err(e) => {
                warn!("Failed to evaluate feature flag 'productCatalogFailure': {:?}", e);
                false
            }
        }
    }
}

#[tonic::async_trait]
impl ProductCatalogService for ProductCatalogServer {
    async fn list_products(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<ListProductsResponse>, Status> {
        let parent_cx = global::get_text_map_propagator(|prop| {
            prop.extract(&MetadataMap(request.metadata()))
        });

        let tracer = global::tracer("product-catalog");
        let mut span = tracer
            .span_builder("oteldemo.ProductCatalogService/ListProducts")
            .with_kind(SpanKind::Server)
            .start_with_context(&tracer, &parent_cx);

        // span attributes taken from Go implementation
        span.set_attribute(KeyValue::new(semconv::trace::RPC_SYSTEM, RPC_SYSTEM_GRPC));
        span.set_attribute(KeyValue::new(semconv::trace::RPC_SERVICE, RPC_SERVICE_PRODUCT_CATALOG));
        span.set_attribute(KeyValue::new(semconv::trace::RPC_METHOD, "ListProducts"));
        let products_list = self.products_list.read().unwrap();
        span.set_attribute(KeyValue::new("app.products.count", products_list.len() as i64));

        let response = ListProductsResponse {
            products: products_list.clone(),
        };

        span.set_attribute(KeyValue::new(
            semconv::trace::RPC_GRPC_STATUS_CODE,
            RPC_GRPC_STATUS_CODE_OK,
        ));

        Ok(Response::new(response))
    }

    async fn get_product(
        &self,
        request: Request<GetProductRequest>,
    ) -> Result<Response<Product>, Status> {
        let parent_cx = global::get_text_map_propagator(|prop| {
            prop.extract(&MetadataMap(request.metadata()))
        });

        let request_message = request.into_inner();
        let product_id = &request_message.id;

        let tracer = global::tracer("product-catalog");
        let mut span = tracer
            .span_builder("oteldemo.ProductCatalogService/GetProduct")
            .with_kind(SpanKind::Server)
            .start_with_context(&tracer, &parent_cx);

        // span attributes taken from Go implementation
        span.set_attribute(KeyValue::new(semconv::trace::RPC_SYSTEM, RPC_SYSTEM_GRPC));
        span.set_attribute(KeyValue::new(semconv::trace::RPC_SERVICE, RPC_SERVICE_PRODUCT_CATALOG));
        span.set_attribute(KeyValue::new(semconv::trace::RPC_METHOD, "GetProduct"));
        span.set_attribute(KeyValue::new("app.product.id", product_id.clone()));

        let cx = Context::current_with_span(span);


        if self.check_product_failure(&cx, product_id).await {
            let msg = "Error: Product Catalog Fail Feature Flag Enabled";
            cx.span().set_status(opentelemetry::trace::Status::error(msg));
            cx.span().add_event(msg.to_string(), vec![]);
            cx.span().set_attribute(KeyValue::new(
                semconv::trace::RPC_GRPC_STATUS_CODE,
                RPC_GRPC_STATUS_CODE_INTERNAL,
            ));
            return Err(Status::internal(msg));
        }


        let products = self.products.read().unwrap();
        match products.get(product_id) {
            Some(product) => {
                cx.span().add_event("Product Found".to_string(), vec![]);
                cx.span().set_attribute(KeyValue::new("app.product.id", product_id.clone()));
                cx.span().set_attribute(KeyValue::new("app.product.name", product.name.clone()));
                cx.span().set_attribute(KeyValue::new(
                    semconv::trace::RPC_GRPC_STATUS_CODE,
                    RPC_GRPC_STATUS_CODE_OK,
                ));
                Ok(Response::new(product.clone()))
            }
            None => {
                let msg = format!("Product Not Found: {}", product_id);
                cx.span().set_status(opentelemetry::trace::Status::error(msg.clone()));
                cx.span().add_event(msg.clone(), vec![]);
                cx.span().set_attribute(KeyValue::new(
                    semconv::trace::RPC_GRPC_STATUS_CODE,
                    RPC_GRPC_STATUS_CODE_NOT_FOUND,
                ));
                Err(Status::not_found(msg))
            }
        }
    }

    async fn search_products(
        &self,
        request: Request<SearchProductsRequest>,
    ) -> Result<Response<SearchProductsResponse>, Status> {
        let parent_cx = global::get_text_map_propagator(|prop| {
            prop.extract(&MetadataMap(request.metadata()))
        });

        let request_message = request.into_inner();
        let query = request_message.query.to_lowercase();

        let tracer = global::tracer("product-catalog");
        let mut span = tracer
            .span_builder("oteldemo.ProductCatalogService/SearchProducts")
            .with_kind(SpanKind::Server)
            .start_with_context(&tracer, &parent_cx);

        // span attributes taken from Go implementation
        span.set_attribute(KeyValue::new(semconv::trace::RPC_SYSTEM, RPC_SYSTEM_GRPC));
        span.set_attribute(KeyValue::new(semconv::trace::RPC_SERVICE, RPC_SERVICE_PRODUCT_CATALOG));
        span.set_attribute(KeyValue::new(semconv::trace::RPC_METHOD, "SearchProducts"));

        let products_list = self.products_list.read().unwrap();
        let results: Vec<Product> = products_list
            .iter()
            .filter(|product| {
                product.name.to_lowercase().contains(&query)
                    || product.description.to_lowercase().contains(&query)
            })
            .cloned()
            .collect();

        span.set_attribute(KeyValue::new("app.products_search.count", results.len() as i64));
        span.set_attribute(KeyValue::new(
            semconv::trace::RPC_GRPC_STATUS_CODE,
            RPC_GRPC_STATUS_CODE_OK,
        ));

        Ok(Response::new(SearchProductsResponse { results }))
    }
}
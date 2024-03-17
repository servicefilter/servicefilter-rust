use std::sync::Arc;

use async_trait::async_trait;
use servicefilter_core::{filter::{ServicefilterChain, ServicefilterExchange, ServicefilterFilter}, service::{FilterConfig, ServiceConfig}};
use tonic::body::empty_body;

pub struct EmptyResponseFilter {
    service_base_load_info: Arc<ServiceConfig>,
    filter_config: FilterConfig
}

impl EmptyResponseFilter {
    pub fn new(
        service_base_load_info: Arc<ServiceConfig>,
        filter_config: FilterConfig,) -> Self {
        Self{
            service_base_load_info,
            filter_config,
        }
    }
}

#[async_trait]
impl ServicefilterFilter for EmptyResponseFilter {

    async fn do_filter(&self, exchange: &mut dyn ServicefilterExchange, chain: &dyn ServicefilterChain) {
        tracing::info!("EmptyResponseFilter args: {:?}", &self.filter_config.args);
        if "empty_response" == exchange.get_target_service_name() {
            let args = &self.filter_config.args;
            let mut resp_status = 200;
            if let Some(resp_status_arg) = args.get("resp_status") {
                resp_status = resp_status_arg.parse().unwrap();
            }
    
            let mut grpc_status = String::from("12");
            if let Some(grpc_status_arg) = args.get("grpc_status") {
                grpc_status = String::from(grpc_status_arg);
            }
    
            let resp = Box::pin(async move {
                Ok(
                    http::Response::builder()
                        .status(resp_status)
                        .header("grpc-status", grpc_status)
                        .header("content-type", "application/grpc")
                        .body(empty_body())
                        .unwrap(),
                )
            });
            exchange.set_resp(resp);
        }
        
        chain.do_chian(exchange).await;
    }
}
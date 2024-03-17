use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use servicefilter_core::{filter::{ServicefilterChain, ServicefilterExchange, ServicefilterFilter}, service::{FilterConfig, ServiceConfig}};
use tonic::body::empty_body;


pub struct ForwardSwitchFilter {
    service_config_base: Arc<ServiceConfig>,
    filter_config: FilterConfig, 
}

impl ForwardSwitchFilter {
    pub fn new(
        service_config_base: Arc<ServiceConfig>,
        filter_config: FilterConfig, 
        ) -> Self {
        Self{
            service_config_base,
            filter_config,
        }
    }
}

#[async_trait]
impl ServicefilterFilter for ForwardSwitchFilter {

    async fn do_filter(&self, exchange: &mut dyn ServicefilterExchange, chain: &dyn ServicefilterChain) {
        let enable_forward = self.filter_config.args.get("enable_forward").ok_or("false").unwrap();
        if "true" == enable_forward {
            chain.do_chian(exchange).await;
        } else {
            let resp = Box::pin(async move {
                Ok(
                    http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(empty_body())
                        .unwrap(),
                )
            });
            exchange.set_resp(resp);
        }
        
    }
}
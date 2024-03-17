use std::sync::Arc;

use async_trait::async_trait;
use servicefilter_core::{filter::{ServicefilterChain, ServicefilterExchange, ServicefilterFilter}, service::{FilterConfig, ServiceConfig}};

pub struct LogFilter {
    service_base_load_info: Arc<ServiceConfig>,
    filter_config: FilterConfig
}

impl LogFilter {
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
impl ServicefilterFilter for LogFilter {

    async fn do_filter(&self, exchange: &mut dyn ServicefilterExchange, chain: &dyn ServicefilterChain) {
        tracing::info!("LogFilter name: {}, target service name: {:?}",self.filter_config.filter_id, &exchange.get_target_service_name());
        chain.do_chian(exchange).await;
    }
}
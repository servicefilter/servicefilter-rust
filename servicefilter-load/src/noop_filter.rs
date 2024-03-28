use async_trait::async_trait;
use servicefilter_core::filter::{ServicefilterChain, ServicefilterExchange, ServicefilterFilter};

pub struct NoopFilter {
    filter_plugin_name: String
}

impl NoopFilter {
    pub fn new(filter_plugin_name: String) -> Self {
        Self{filter_plugin_name}        
    }
}

#[async_trait]
impl ServicefilterFilter for NoopFilter {

    async fn do_filter(&self, exchange: &mut dyn ServicefilterExchange, chain: &dyn ServicefilterChain) {
        tracing::info!("noop filter: {}", self.filter_plugin_name);
        chain.do_chian(exchange).await;
    }
}
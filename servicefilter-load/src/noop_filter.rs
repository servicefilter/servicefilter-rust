use async_trait::async_trait;
use servicefilter_core::filter::{ServicefilterChain, ServicefilterExchange, ServicefilterFilter};

pub struct NoopFilter {
}

impl NoopFilter {
    pub fn new() -> Self {
        Self{}        
    }
}

#[async_trait]
impl ServicefilterFilter for NoopFilter {

    async fn do_filter(&self, exchange: &mut dyn ServicefilterExchange, chain: &dyn ServicefilterChain) {
        tracing::info!("noop filter");
        chain.do_chian(exchange).await;
    }
}
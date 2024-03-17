use std::sync::Arc;

use async_trait::async_trait;
use servicefilter_core::{filter::{ServicefilterChain, ServicefilterExchange, ServicefilterFilter}, service::{FilterConfig, ServiceConfig}, X_SERVICEFILTER_TARGET_SERVICE_NAME};
use tonic::metadata::MetadataValue;
use tonic::IntoRequest;

mod app_grpc {
    tonic::include_proto!("servicefilter.control.app.proto");
}

pub struct MockLocalFilter {
    service_base_load_info: Arc<ServiceConfig>,
    filter_config: FilterConfig,
    channel_gen : Arc<Box<dyn servicefilter_core::channel::FilterReqChannelGen>>,
}

impl MockLocalFilter {
    pub fn new(
        service_base_load_info: Arc<ServiceConfig>,
        filter_config: FilterConfig,
        channel_gen : Arc<Box<dyn servicefilter_core::channel::FilterReqChannelGen>>,
    ) -> Self {
        Self{
            service_base_load_info,
            filter_config,
            channel_gen,
        }
    }
}

#[async_trait]
impl ServicefilterFilter for MockLocalFilter {

    async fn do_filter(&self, exchange: &mut dyn ServicefilterExchange, chain: &dyn ServicefilterChain) {
        let channel = self.channel_gen.gen_channel().await;
        let req = app_grpc::HealthRequest{};
        let mut req = req.into_request();
        let service_name: MetadataValue<_> = "empty_response".parse().unwrap();
        req.metadata_mut().insert(X_SERVICEFILTER_TARGET_SERVICE_NAME,service_name);

        let mut client = app_grpc::servicefilter_manager_service_client::ServicefilterManagerServiceClient::new(channel);
        let resp = client.health(req).await;

        tracing::info!("MockLocalFilter resp: {:?}", resp);
        tracing::info!("MockLocalFilter name: {}, target service name: {:?}",self.filter_config.filter_id, &exchange.get_target_service_name());

        // TODO Mark request has been processed
        chain.do_chian(exchange).await;
    }
}
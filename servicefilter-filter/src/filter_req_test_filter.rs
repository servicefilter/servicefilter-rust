use std::sync::Arc;

use async_trait::async_trait;
use servicefilter_core::{filter::{ServicefilterChain, ServicefilterExchange, ServicefilterFilter}, service::{FilterConfig, ServiceConfig}, X_SERVICEFILTER_TARGET_SERVICE_NAME};
use tonic::metadata::MetadataValue;
use tonic::IntoRequest;

mod app_grpc {
    tonic::include_proto!("servicefilter.control.app.proto");
}

pub struct FilterReqTestFilter {
    service_base_load_info: Arc<ServiceConfig>,
    filter_config: FilterConfig,
    channel_gen : Arc<Box<dyn servicefilter_core::channel::FilterReqChannelGen>>,
}

impl FilterReqTestFilter {
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
impl ServicefilterFilter for FilterReqTestFilter {

    async fn do_filter(&self, exchange: &mut dyn ServicefilterExchange, chain: &dyn ServicefilterChain) {
        let channel = self.channel_gen.gen_channel().await;
        let req = app_grpc::HealthRequest{};
        let mut req = req.into_request();
        let service_name: MetadataValue<_> = "grpc".parse().unwrap();
        req.metadata_mut().insert(X_SERVICEFILTER_TARGET_SERVICE_NAME,service_name);

        let mut client = app_grpc::servicefilter_manager_service_client::ServicefilterManagerServiceClient::new(channel);
        let resp = client.health(req).await;

        tracing::info!("FilterReqTestFilter resp: {:?}", resp);
        tracing::info!("FilterReqTestFilter name: {}, target service name: {:?}",self.filter_config.filter_id, &exchange.get_target_service_name());
        chain.do_chian(exchange).await;
    }
}
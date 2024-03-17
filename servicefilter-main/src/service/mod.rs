mod chain;
mod filter;
mod channel;


use std::sync::Arc;

use servicefilter_core::{filter::{FilterKind, ServicefilterFilter}, service::{FilterConfig, ProtocolListenConfig, ServiceConfig, ServiceConfigOpreate, ServiceGenChain, ServiceListenConfig}, Result};
use servicefilter_load::LoadFactory;
use tokio::{sync::RwLock, task::JoinHandle};

use crate::cmd::config::{ServicefilterFilterDefineConfig, ServicefilterServiceConfig};

use self::{chain::{RoutingChainGen, RoutingGenChain}, channel::{FilterReqChannelGen, FilterReqGenChain}};


pub struct ServiceRunHandler {
    app_id: String, 
    service_config: ServicefilterServiceConfig,
}

impl ServiceRunHandler {

    pub fn new(
        app_id: String,
        service_config: ServicefilterServiceConfig,
        config_operate: &mut ServiceConfigOpreate,
    ) -> Self {
        Self { app_id, service_config,  }
    }

    pub async fn run(&self, load_factory: Arc<LoadFactory>) -> Result<()> {
        let service_config = &self.service_config;
        let service_listen = &self.service_config.service_listen;

        let mut service_listen_config: Option<ServiceListenConfig> = None;
        if let Some(service_listen) = service_listen {
            service_listen_config = Some(ServiceListenConfig::new(service_listen.protocol.clone(), service_listen.address.clone()));
        }

        let service_config_base = Arc::new(ServiceConfig::new(
            service_config.service_id.clone(),
            service_config.service_name.clone(),
            service_config.alias_names.clone(),
            service_config.attributes.clone(),
            service_listen_config,
        ));
        let routing_chan_gen = Self::build_chain(self.service_config.clone(), service_config_base.clone(), load_factory.clone()).await;
        let chain_gen = Arc::new(RwLock::new(routing_chan_gen));
        let out_gen_chain : Box<dyn ServiceGenChain> = Box::new(RoutingGenChain::new(chain_gen.clone(), FilterKind::OUTPUT));
        let out_gen : Arc<Box<dyn servicefilter_core::channel::FilterReqChannelGen>>= Arc::new(Box::new(FilterReqChannelGen::new(Arc::new(out_gen_chain))));
        let local_filters: Arc<Vec<Box<dyn ServicefilterFilter>>> = Arc::new(Self::build_filter(&load_factory, &service_config_base, &service_config.filter.local, &out_gen).await);
        let mut chain_gen_write_lock = chain_gen.write().await;
        chain_gen_write_lock.rebuild_local(local_filters);
        drop(chain_gen_write_lock);

        let listen = &service_config.listen;
        let listen_config = ProtocolListenConfig::new(listen.protocol.clone(), listen.address.clone(), listen.args.clone());
        let server = load_factory.load_protocol_server(&listen.plugin_name).unwrap();
        let gen_chain: Box<dyn ServiceGenChain> = Box::new(RoutingGenChain::new(chain_gen.clone(), FilterKind::PREROUTING));
        let start = server.start(service_config_base.clone(), listen_config, gen_chain);
        if let Err(e) = start {
            return Err(e);
        }

        let local_listen = &service_config.local_listen;
        if let Some(local_listen) = local_listen {
            let local_listen_config = ProtocolListenConfig::new(local_listen.protocol.clone(), local_listen.address.clone(), local_listen.args.clone());
            let local_server = load_factory.load_protocol_server(&local_listen.plugin_name).unwrap();
            let local_gen_chain : Box<dyn ServiceGenChain> = Box::new(RoutingGenChain::new(chain_gen, FilterKind::OUTPUT));
            let local_start = local_server.start(service_config_base.clone(), local_listen_config, local_gen_chain);
            if let Err(e) = local_start {
                return Err(e);
            }
    
            tokio::select! {
                _ = start.unwrap() => {},
                _ = local_start.unwrap() => {},
            }
        } else {
            tokio::select! {
                _ = start.unwrap() => {},
            }
        }
        
        
        return Ok(());
    }

    async fn build_chain(service_config: ServicefilterServiceConfig, service_config_base: Arc<ServiceConfig>, load_factory: Arc<LoadFactory>) -> RoutingChainGen {
        // let local_service_id = String::from(&service_config.service_id);
        let local_service_name = String::from(&service_config.service_name);
        let local_service_alias_names = Arc::new(service_config.alias_names.clone());

        let filter = &service_config.filter;
        let channel_filters = Arc::new(Self::build_req_filter(&load_factory, &service_config_base, &filter.channel).await);
        let channel_gen_chain: Arc<Box<dyn ServiceGenChain>> = Arc::new(Box::new(FilterReqGenChain::new(channel_filters)));
        let channel_gen : Arc<Box<dyn servicefilter_core::channel::FilterReqChannelGen>>= Arc::new(Box::new(FilterReqChannelGen::new(channel_gen_chain)));
        
        let prerouting_filters: Arc<Vec<Box<dyn ServicefilterFilter>>> = Arc::new(Self::build_filter(&load_factory, &service_config_base, &filter.prerouting, &channel_gen).await);
        let input_filters: Arc<Vec<Box<dyn ServicefilterFilter>>> = Arc::new(Self::build_filter(&load_factory, &service_config_base, &filter.input, &channel_gen).await);

        // let local_filters: Arc<Vec<Box<dyn ServicefilterFilter>>> = Arc::new(Self::build_filter(&load_factory, &service_config_base, &filter.local, ).await);
        
        let output_filters: Arc<Vec<Box<dyn ServicefilterFilter>>> = Arc::new(Self::build_filter(&load_factory, &service_config_base, &filter.output, &channel_gen).await);
        let forward_filters: Arc<Vec<Box<dyn ServicefilterFilter>>> = Arc::new(Self::build_filter(&load_factory, &service_config_base, &filter.forward, &channel_gen).await);
        let postrouting_filters: Arc<Vec<Box<dyn ServicefilterFilter>>> = Arc::new(Self::build_filter(&load_factory, &service_config_base, &filter.postrouting, &channel_gen).await);
        
        return RoutingChainGen::new(local_service_name, local_service_alias_names, 
            prerouting_filters, input_filters, Arc::default(), output_filters, forward_filters, postrouting_filters);
    }

    async fn build_req_filter(load_filter: &Arc<LoadFactory>, 
        service_config_base: &Arc<ServiceConfig>, 
        filter_configs: &Vec<ServicefilterFilterDefineConfig>,
    ) -> Vec<Box<dyn ServicefilterFilter>>{
        let mut filters: Vec<Box<dyn ServicefilterFilter>> = vec![];
        for config in filter_configs {
            let filter_config_base = FilterConfig::new(config.filter_id.clone(), config.filter_name.clone(), config.args.clone());
            let filter_result = load_filter.load_req_filter(&config.plugin_name, service_config_base.clone(), filter_config_base, ).await;
            // TODO deal no filter config
            let filter  = filter_result.unwrap();
            filters.push(filter);
        }
        return filters;
    }

    async fn build_filter(load_filter: &Arc<LoadFactory>, 
        service_config_base: &Arc<ServiceConfig>, 
        filter_configs: &Vec<ServicefilterFilterDefineConfig>,
        channel_gen : &Arc<Box<dyn servicefilter_core::channel::FilterReqChannelGen>>,
    ) -> Vec<Box<dyn ServicefilterFilter>>{
        let mut filters: Vec<Box<dyn ServicefilterFilter>> = vec![];
        for config in filter_configs {
            let filter_config_base = FilterConfig::new(config.filter_id.clone(), config.filter_name.clone(), config.args.clone());
            let filter_result = load_filter.load_filter(config.plugin_name.clone(), service_config_base.clone(), filter_config_base, channel_gen.clone()).await;
            // TODO deal no filter config
            let filter  = filter_result.unwrap();
            filters.push(filter);
        }
        return filters;
    }
}

pub struct ServiceOperateHandler {
    pub handler: JoinHandle<()>,
    pub opreate: ServiceConfigOpreate,
}

impl ServiceOperateHandler {
    pub fn new(
        handler: JoinHandle<()>,
        opreate: ServiceConfigOpreate,
    ) -> Self {
        Self {
            handler,
            opreate,
        }
    }
}

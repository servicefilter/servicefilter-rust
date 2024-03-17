use std::sync::{Arc, RwLock};

use servicefilter_core::{filter::{FilterKind, ServicefilterChain, ServicefilterExchange, ServicefilterFilter}, service::ServiceGenChain};
use tonic::async_trait;

pub struct RoutingGenChain {
    routing_chain: Arc<tokio::sync::RwLock<RoutingChainGen>>,
    filter_kind: FilterKind
}

impl RoutingGenChain {
    pub fn new(
        routing_chain: Arc<tokio::sync::RwLock<RoutingChainGen>>,
        filter_kind: FilterKind
    ) -> Self {
        Self { routing_chain, filter_kind }
    }
}

#[async_trait]
impl ServiceGenChain for RoutingGenChain {

    async fn gen_chain(&self) -> Box<dyn ServicefilterChain> {
        return self.routing_chain.read().await.gen_kind_chain(self.filter_kind);
    }
}

pub struct RoutingChainGen {

    local_service_name: String,

    local_service_alias_names: Arc<Vec<String>>,

    prerouting_filters: Arc<Vec<Box<dyn ServicefilterFilter>>>,

    input_filters: Arc<Vec<Box<dyn ServicefilterFilter>>>,

    local_filters: Arc<Vec<Box<dyn ServicefilterFilter>>>,

    output_filters: Arc<Vec<Box<dyn ServicefilterFilter>>>,

    forward_filters: Arc<Vec<Box<dyn ServicefilterFilter>>>,

    postrouting_filters: Arc<Vec<Box<dyn ServicefilterFilter>>>,
}

impl RoutingChainGen {
    pub fn new(
        local_service_name: String,
        local_service_alias_names: Arc<Vec<String>>,
        prerouting_filters: Arc<Vec<Box<dyn ServicefilterFilter>>>,
        input_filters: Arc<Vec<Box<dyn ServicefilterFilter>>>,
        local_filters: Arc<Vec<Box<dyn ServicefilterFilter>>>,
        output_filters: Arc<Vec<Box<dyn ServicefilterFilter>>>,
        forward_filters: Arc<Vec<Box<dyn ServicefilterFilter>>>,
        postrouting_filters: Arc<Vec<Box<dyn ServicefilterFilter>>>,
    ) -> Self {
        Self {
            local_service_name,
            local_service_alias_names, 
            prerouting_filters, 
            input_filters, 
            local_filters,
            output_filters, 
            forward_filters, 
            postrouting_filters,
         }
    }

    pub fn rebuild_local(&mut self, local_filters: Arc<Vec<Box<dyn ServicefilterFilter>>>,) {
        self.local_filters = local_filters;
    }
    
    fn gen_kind_chain(&self, filter_kind: FilterKind) -> Box<dyn ServicefilterChain> {
        return Box::new(VirtualServicefilterRoutingChain::new(filter_kind, self));
    }

}

// #[async_trait]
// impl ServiceProtocolChain for RoutingChain {

//     async fn gen_chain(&self, filter_kind: FilterKind) -> Box<dyn ServicefilterChain> {
//         return Box::new(VirtualServicefilterRoutingChain::new(filter_kind, self));
//     }
    
// }

// #[async_trait]
// impl ServiceGenChain for RoutingChain {

//     async fn gen_chain(&self) -> Box<dyn ServicefilterChain> {

//     }
// }

struct VirtualServicefilterRoutingChain {

    current_filter_index: RwLock<usize>,

    current_filter_kind: RwLock<FilterKind>,

    local_service_name: String,

    local_service_alias_names: Arc<Vec<String>>,

    prerouting_filters: Arc<Vec<Box<dyn ServicefilterFilter>>>,

    input_filters: Arc<Vec<Box<dyn ServicefilterFilter>>>,

    local_filters: Arc<Vec<Box<dyn ServicefilterFilter>>>,

    output_filters: Arc<Vec<Box<dyn ServicefilterFilter>>>,

    forward_filters: Arc<Vec<Box<dyn ServicefilterFilter>>>,

    postrouting_filters: Arc<Vec<Box<dyn ServicefilterFilter>>>,
}

impl VirtualServicefilterRoutingChain {
    fn new(current_filter_kind: FilterKind, fliter_chain: &RoutingChainGen) -> Self {
        Self { current_filter_index: RwLock::new(0), 
            current_filter_kind: RwLock::new(current_filter_kind), 
            local_service_name: fliter_chain.local_service_name.clone(),
            local_service_alias_names: fliter_chain.local_service_alias_names.clone(),
            prerouting_filters: fliter_chain.prerouting_filters.clone(),
            input_filters: fliter_chain.input_filters.clone(),
            local_filters: fliter_chain.local_filters.clone(),
            output_filters: fliter_chain.output_filters.clone(),
            forward_filters: fliter_chain.forward_filters.clone(),
            postrouting_filters: fliter_chain.postrouting_filters.clone(),
        }
    }

    fn calc_current_filter(&self, exchange : &mut dyn ServicefilterExchange) -> Option<&Box<dyn ServicefilterFilter>> {

        let mut current_filter: Option<&Box<dyn ServicefilterFilter>>;
        let cur_filter_kind = self.get_current_filter_kind();
        let mut current_filter_index = self.get_current_filter_index();
        match cur_filter_kind {
            FilterKind::PREROUTING => {
                current_filter = self.prerouting_filters.get(current_filter_index);
                current_filter_index += 1;
                self.set_current_filter_index(current_filter_index);
                if current_filter.is_none() {
                    self.set_current_filter_index(0);
                    let next_filter_kind = self.calc_pre_next_filter_kind(exchange);
                    self.set_current_filter_kind(next_filter_kind);
                    current_filter = self.calc_current_filter(exchange);
                }

            }
            FilterKind::INPUT => {
                current_filter = self.input_filters.get(current_filter_index);
                current_filter_index += 1;
                self.set_current_filter_index(current_filter_index);
                if current_filter.is_none() {
                    // //TODO 
                    // let socket = self.local_service_rsocket.write();
                    // let req = exchange.gen_req();
                    // let result = socket.request_response(req.to_owned()).await;
                    // exchange.result = Some(result);

                    // drop(socket);

                    self.set_current_filter_index(0);
                    let next_filter_kind = FilterKind::LOCAL;
                    self.set_current_filter_kind(next_filter_kind);
                    current_filter = self.calc_current_filter(exchange);
                    // current_filter = None;
                }
            }
            FilterKind::LOCAL => {
                current_filter = self.local_filters.get(current_filter_index);
                current_filter_index += 1;
                self.set_current_filter_index(current_filter_index);
                if current_filter.is_none() {
                    // //TODO 
                    // let socket = self.local_service_rsocket.write();
                    // let req = exchange.gen_req();
                    // let result = socket.request_response(req.to_owned()).await;
                    // exchange.result = Some(result);

                    // // drop(socket);

                    // self.set_current_filter_index(0);
                    // let next_filter_kind = FilterKind::OUTPUT;
                    // self.set_current_filter_kind(next_filter_kind);
                    // current_filter = self.calc_current_filter(exchange);
                    current_filter = None;
                }
            }
            FilterKind::OUTPUT => {
                current_filter = self.output_filters.get(current_filter_index);
                current_filter_index += 1;
                self.set_current_filter_index(current_filter_index);
                if current_filter.is_none() {
                    self.set_current_filter_index(0);
                    let next_filter_kind = self.calc_output_next_filter_kind(exchange);
                    self.set_current_filter_kind(next_filter_kind);
                    current_filter = self.calc_current_filter(exchange);
                }
            }
            FilterKind::FORWARD => {
                current_filter = self.forward_filters.get(current_filter_index);
                current_filter_index += 1;
                self.set_current_filter_index(current_filter_index);
                if current_filter.is_none() {
                    self.set_current_filter_index(0);
                    let next_filter_kind = FilterKind::POSTROUTING;
                    self.set_current_filter_kind(next_filter_kind);
                    current_filter = self.calc_current_filter(exchange);
                }
            }
            FilterKind::POSTROUTING => {
                current_filter = self.postrouting_filters.get(current_filter_index);
                current_filter_index += 1;
                self.set_current_filter_index(current_filter_index);
            }
        }
        
        current_filter
    }

    fn calc_pre_next_filter_kind(&self, exchange : &dyn ServicefilterExchange) -> FilterKind {

        // let request_hearders = &exchange.request_hearders;
        let trget_service_name = exchange.get_target_service_name();

        let local_service_name = &self.local_service_name;
        let local_service_tags = &self.local_service_alias_names;

        if local_service_name == trget_service_name || local_service_tags.contains(&String::from(trget_service_name)) {
            return FilterKind::INPUT;
        }
        
        return FilterKind::FORWARD;

    }

    fn calc_output_next_filter_kind(&self, _ : &dyn ServicefilterExchange) -> FilterKind {
        // let request_hearders = &exchange.request_hearders;
        // let trget_service_name = &request_hearders.target_service_name;

        //TODO like Loopback Address
        // let local_service_name = self.local_service_name.read().clone();
        // if &local_service_name == trget_service_name {
        //     return FilterKind::PREROUTING;
        // }
        return FilterKind::POSTROUTING;

    }

    fn get_current_filter_index(&self) -> usize {
        *self.current_filter_index.read().unwrap()
    }

    fn set_current_filter_index(&self, new_index: usize) {
        *self.current_filter_index.write().unwrap() = new_index;
    }

    fn get_current_filter_kind(&self) -> FilterKind {
        return *self.current_filter_kind.read().unwrap();
    }

    fn set_current_filter_kind(&self, new_filter_kind: FilterKind) {
        *self.current_filter_kind.write().unwrap() = new_filter_kind;
    }
    
}


#[async_trait]
impl ServicefilterChain for VirtualServicefilterRoutingChain {

    async fn do_chian(&self, exchange : &mut dyn ServicefilterExchange) {

        let current_filter_option = self.calc_current_filter(exchange);
        
        if let Some(current_filter) = current_filter_option {
            current_filter.do_filter(exchange, self).await;
        } else {
            // TODO write response
        }
    }
}
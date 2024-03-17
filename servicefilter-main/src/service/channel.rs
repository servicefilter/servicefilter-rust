use std::{collections::HashMap, sync::Arc};
use servicefilter_core::{channel::FilterReqChannel, filter::{ServicefilterChain, ServicefilterFilter}, service::ServiceGenChain, X_SERVICEFILTER_TARGET_SERVICE_ID, X_SERVICEFILTER_TARGET_SERVICE_NAME};
use tokio::sync::RwLock;
use tonic::{async_trait, body::{empty_body, BoxBody}, codegen::Service};
use super::filter::ReqExchange;
use servicefilter_core::filter::ServicefilterExchange;

pub struct FilterReqChain {
    index: RwLock<usize>,
    filters: Arc<Vec<Box<dyn ServicefilterFilter>>>,
}

impl FilterReqChain {
    pub fn new(filters: Arc<Vec<Box<dyn ServicefilterFilter>>>) -> Self {
        Self { index: RwLock::new(0), filters }
    }
}

#[async_trait]
impl ServicefilterChain for FilterReqChain {
    
    async fn do_chian(&self, exchange : &mut dyn ServicefilterExchange) {
        let mut index = self.index.write().await;
        let filter_op = self.filters.get(*index);
        if let Some(filter) = filter_op {
            *index += 1;
            drop(index);
            filter.do_filter(exchange, self).await;
        }
    }
}

pub struct FilterReqGenChain {
    filters: Arc<Vec<Box<dyn ServicefilterFilter>>>,
}

impl FilterReqGenChain {
    pub fn new(filters: Arc<Vec<Box<dyn ServicefilterFilter>>>,) -> Self {
        Self { filters }
    }
}

#[async_trait]
impl ServiceGenChain for FilterReqGenChain {
    async fn gen_chain(&self) -> Box<dyn ServicefilterChain> {
        return Box::new(FilterReqChain::new(self.filters.clone()));
    }
}

pub struct FilterReqChannelGen {
    gen_chain: Arc<Box<dyn ServiceGenChain>>,
    
}

impl FilterReqChannelGen {
    pub fn new(gen_chain: Arc<Box<dyn ServiceGenChain>>,) -> Self {
        Self {gen_chain }
    }
}

#[async_trait]
impl servicefilter_core::channel::FilterReqChannelGen for FilterReqChannelGen {
    async fn gen_channel(&self) -> Box<FilterReqChannel> {
        return Box::new(VirtualFilterReqChannel::new(self.gen_chain.clone()));
    }
}


struct VirtualFilterReqChannel {
    gen_chain: Arc<Box<dyn ServiceGenChain>>,
}

impl VirtualFilterReqChannel  {
    fn new(gen_chain: Arc<Box<dyn ServiceGenChain>>,) -> Self {
        Self { gen_chain }
    }
}

impl Service<http::Request<BoxBody>> for VirtualFilterReqChannel {
    type Response = http::Response<tonic::body::BoxBody>;
    type Error = std::convert::Infallible;
    type Future = tonic::codegen::BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<BoxBody>) -> Self::Future {
        let gen_chain = self.gen_chain.clone();
        return Box::pin(async move {
            let headers = req.headers();
            let target_service_id_op = headers.get(X_SERVICEFILTER_TARGET_SERVICE_ID);
            let mut target_service_id = None;
            if let Some(id) = target_service_id_op {
                target_service_id = Some(String::from(id.to_str().unwrap()));

            }
            let target_service_name = headers.get(X_SERVICEFILTER_TARGET_SERVICE_NAME).unwrap().to_str().unwrap();

            let mut exchange = ReqExchange::new(
                target_service_id, 
                String::from(target_service_name), 
                HashMap::new(),
                Some(req),
                None,
                None,
            );
            let chain = gen_chain.gen_chain().await;
            chain.do_chian(&mut exchange).await;

            // exchange.resp.unwrap()
            let resp = exchange.get_resp();
            match resp {
                Some(resp) => {
                    return resp.await;
                },
                None => {
                    return Ok(
                        http::Response::builder()
                            .status(200)
                            .header("grpc-status", "12")
                            .header("content-type", "application/grpc")
                            .body(empty_body())
                            .unwrap(),
                    );
                },
            }
        });
    }
}

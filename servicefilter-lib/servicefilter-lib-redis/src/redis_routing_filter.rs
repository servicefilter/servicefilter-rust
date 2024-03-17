use std::pin::Pin;

use async_trait::async_trait;
use servicefilter_core::filter::{ServicefilterChain, ServicefilterExchange, ServicefilterFilter};

use redis::AsyncCommands;
use redis_operate::{GetRequest,GetResponse, SetRequest, SetResponse, DeleteRequest, DeleteResponse, ExistsRequest, ExistsResponse, ExpireRequest, ExpireResponse, KeysRequest, KeysResponse, Message, SubscribeRequest, PublishRequest, PublishResponse, redis_operate_service_server::{RedisOperateServiceServer, RedisOperateService}};

use tokio::sync::mpsc;
use tonic::{codegen::Service, Status};

use tokio_stream::Stream;
use tokio_stream::StreamExt;

pub mod redis_operate {
    tonic::include_proto!("servicefilter.sdk.redis");
}

pub struct RedisRoutingFilter {
    
}

impl RedisRoutingFilter {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl ServicefilterFilter for RedisRoutingFilter {
    
    async fn do_filter(&self, exchange: &mut dyn ServicefilterExchange, chain: &dyn ServicefilterChain) {
        if let Some(instance) = exchange.get_service_instance() {
            if &"redis".to_string() == instance.get_service_protocol() {
                
                let addr = instance.get_address().clone();
                let client = RedisClient::new(addr);
                let mut server = RedisOperateServiceServer::new(client);
                // server.poll_ready(cx)
                let req = exchange.get_req();
                let resp = server.call(req.unwrap());
                exchange.set_resp(resp);
            }
        }
        
        chain.do_chian(exchange).await;
    }
}

struct RedisClient {
    addr: String,
}

// TODO use redis connection pool
impl RedisClient {
    pub fn new(addr: String) -> Self {
        
        Self {
            addr,
        }
    }
    
}

#[tonic::async_trait]
impl RedisOperateService for RedisClient {

    async fn get(
        &self,
        request: tonic::Request<GetRequest>,
    ) -> std::result::Result<tonic::Response<GetResponse>, tonic::Status> {
        let req = request.into_inner();
        let key = req.key;

        let client = redis::Client::open(self.addr.clone()).unwrap();
        let mut con = client.get_async_connection().await.unwrap();

        let rv: String = con.get(key).await.unwrap();
        Ok(tonic::Response::new(GetResponse{result: Some(redis_operate::get_response::Result::Value(rv))}))
    }

    async fn set(
        &self,
        request: tonic::Request<SetRequest>,
    ) -> std::result::Result<tonic::Response<SetResponse>, tonic::Status> {
        let req = request.into_inner();
        let key = req.key;
        let val = req.value;

        let client = redis::Client::open(self.addr.clone()).unwrap();
        let mut con = client.get_async_connection().await.unwrap();
        
        let _:() = con.set(key, val).await.unwrap();
        Ok(tonic::Response::new(SetResponse{success: true}))
    }

    async fn delete(
        &self,
        request: tonic::Request<DeleteRequest>,
    ) -> std::result::Result<tonic::Response<DeleteResponse>, tonic::Status> {
        let req = request.into_inner();
        let key = req.key;

        let client = redis::Client::open(self.addr.clone()).unwrap();
        let mut con = client.get_async_connection().await.unwrap();

        let _:() = con.del(key).await.unwrap();
        Ok(tonic::Response::new(DeleteResponse{deleted: 1}))
    }

    async fn exists(
        &self,
        request: tonic::Request<ExistsRequest>,
    ) -> std::result::Result<tonic::Response<ExistsResponse>, tonic::Status> {
        let req = request.into_inner();
        let keys = req.keys;

        let client = redis::Client::open(self.addr.clone()).unwrap();
        let mut con = client.get_async_connection().await.unwrap();

        let count = con.exists(keys).await.unwrap();
        Ok(tonic::Response::new(ExistsResponse{count: count}))
    }

    async fn expire(
        &self,
        request: tonic::Request<ExpireRequest>,
    ) -> std::result::Result<tonic::Response<ExpireResponse>, tonic::Status> {
        let req = request.into_inner();
        let key = req.key;
        let seconds = req.seconds as i64;

        let client = redis::Client::open(self.addr.clone()).unwrap();
        let mut con = client.get_async_connection().await.unwrap();

        let _:() = con.expire(key, seconds).await.unwrap();
        Ok(tonic::Response::new(ExpireResponse{success: true}))
    }

    async fn keys(
        &self,
        request: tonic::Request<KeysRequest>,
    ) -> std::result::Result<tonic::Response<KeysResponse>, tonic::Status> {
        let req = request.into_inner();
        let pattern = req.pattern;

        let client = redis::Client::open(self.addr.clone()).unwrap();
        let mut con = client.get_async_connection().await.unwrap();

        let keys = con.keys(pattern).await.unwrap();
        Ok(tonic::Response::new(KeysResponse{keys: keys}))
    }

    type SubscribeStream = Pin<Box<dyn Stream<Item = std::result::Result<Message, Status>> + Send + Sync + 'static>>;

    /// RPC service for subscribing to a channel. This is a streaming RPC, where the server sends messages to the client.
    async fn subscribe(
        &self,
        request: tonic::Request<SubscribeRequest>,
    ) -> std::result::Result<tonic::Response<Self::SubscribeStream>, tonic::Status> {
        
        let (tx, rx) = mpsc::channel(4);
        let channel_name = request.into_inner().channel;

        let addr = self.addr.clone();
        tokio::spawn(async move {
            let client = redis::Client::open(addr).unwrap();
            let con = client.get_async_connection().await.unwrap();

            let mut pubsub = con.into_pubsub();
            pubsub.subscribe(&channel_name).await.unwrap();
            let mut stream = pubsub.on_message();

            loop {
                let msg = stream.next().await.unwrap();
                let payload: String = msg.get_payload().unwrap();
                let reply = Message {
                    channel: msg.get_channel_name().to_string(),
                    data: payload,
                };
                let _ = tx.send(Ok(reply)).await;
            }
        });

        Ok(tonic::Response::new(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)) as Self::SubscribeStream))
    }

    /// RPC service for publishing a message to a channel.
    async fn publish(
        &self,
        request: tonic::Request<PublishRequest>,
    ) -> std::result::Result<tonic::Response<PublishResponse>, tonic::Status> {
        let req = request.into_inner();
        let channel = req.channel;
        let data = req.data;

        let client = redis::Client::open(self.addr.clone()).unwrap();
        let mut con = client.get_async_connection().await.unwrap();

        let success = con.publish(channel, data).await.unwrap();
        Ok(tonic::Response::new(PublishResponse{success: success}))
    }
    
}
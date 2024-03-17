use std::time::Duration;

use async_trait::async_trait;
use servicefilter_core::{filter::{ServicefilterChain, ServicefilterExchange, ServicefilterFilter}, service::{FilterConfig, ServiceConfig}};
use tonic::body::empty_body;
pub struct TimeoutFilter {

}

impl TimeoutFilter {
    pub fn new() -> Self {
        Self{}
    }
}

#[async_trait]
impl ServicefilterFilter for TimeoutFilter {

    async fn do_filter(&self, exchange: &mut dyn ServicefilterExchange, chain: &dyn ServicefilterChain) {
        chain.do_chian(exchange).await;

        let resp = exchange.get_resp();
        if let Some(resp_future) = resp {
            let timeout_duration = Duration::from_secs(10);
            let resp = Box::pin(async move {
                match tokio::time::timeout(timeout_duration, resp_future).await {
                    Ok(result) => match result {
                        // 任务在超时前完成
                        Ok(_) => result,
                        // 异步操作返回错误
                        Err(e) => {
                            result
                        }
                    },
                    // 超时发生
                    Err(e) => {
                        tracing::warn!("err is {}", e.to_string());
                        let resp = http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .header("hello11111", "world")
                        .body(empty_body())
                        .unwrap();
                        Ok(resp)
                    },
                }
            });
            exchange.set_resp(resp);
        }
        
    }
}
use std::{future::Future, pin::Pin, task::{Context, Poll}, time::Duration};

use anyhow::Error;
use chrono::{DateTime, Utc};
use rocpp_client::v16::{Diagnostics, DiagnosticsResponse};
use tokio::time::Timeout;

use crate::interface::ftp::FtpService;

pub struct DiagnosticsService {
    file_name: Option<String>,
    upload_fut: Option<Pin<Box<Timeout<Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>>>>>,
}

impl DiagnosticsService {
    pub fn new() -> Self {
        Self {
            file_name: None,
            upload_fut: None
        }
    }
}

impl Diagnostics for DiagnosticsService {
    async fn get_file_name(&mut self, _start_time: Option<DateTime<Utc>>, _stop_time: Option<DateTime<Utc>>) -> Option<String> {
        let file_name = "hello.txt".to_string();
        self.file_name = Some(file_name.clone());
        Some(file_name)
    }
    async fn diagnostics_upload(&mut self, location: String, timeout: u64) {
        let ftp = FtpService::new(location);
        let file_name = self.file_name.clone().unwrap();
        let inner_fut: Pin<Box<dyn Future<Output = Result<(), Error>> + Send>> = Box::pin(async move { ftp.upload(file_name).await });
        let timed = tokio::time::timeout(Duration::from_secs(timeout), inner_fut);
        self.upload_fut = Some(Box::pin(timed));
        
    }
    fn poll_diagnostics_upload(&mut self, cx: &mut Context<'_>) -> Poll<DiagnosticsResponse> {
        let timed_fut = match self.upload_fut.as_mut() {
            Some(fut) => fut,
            None => {
                return Poll::Pending
            }
        };
        match timed_fut.as_mut().poll(cx) {
            Poll::Ready(t) => {
                let res = match t {
                    Ok(t) => {
                        match t {
                            Ok(_) => {
                                DiagnosticsResponse::Success
                            },
                            Err(_) => {
                                DiagnosticsResponse::Failed
                            }
                        }
                    }
                    Err(_) => {
                        DiagnosticsResponse::Timeout
                    }
                };
                self.upload_fut.take();
                Poll::Ready(res)
            }
            Poll::Pending => Poll::Pending
        }
    }
}
use std::{
    collections::HashMap,
    time::Duration,
};
use tonic::codegen::http::uri::Uri;

use crate::db_resolver_state::DBResolverState;
use crate::WasmParser;
use anyhow::Result;
use async_trait::async_trait;
use futures::StreamExt;
use sqlx::PgPool;
use tokio_util::time::delay_queue::DelayQueue;
use int_enum::IntEnum;
use substreams_sink::{OffchainData};

#[repr(i32)]
#[derive(Copy, Clone, IntEnum)]
pub enum TaskState {
    Queued = 0,
    UnknownURI = 1,
    UnknownParser = 2,
    DownloadFailed = 3,
    ParsingFailed = 4,
    Finished = 5,
}

/// Resolve task
#[derive(Clone)]
pub struct ResolveTask {
    pub manifest: String,
    pub request: OffchainData,
    pub num_retries: i32,
}

impl ResolveTask {
    fn increment_try_counter(&mut self) -> bool {
        if self.num_retries < self.request.max_retries {
            self.num_retries = self.num_retries + 1;
            true
        } else {
            false
        }
    }
}

/// Link resolver
#[async_trait]
pub trait LinkResolver: Send + Sync + 'static {
    async fn download(&self, uri: &str) -> Result<Vec<u8>>;
}

/// Resolver state
#[async_trait]
pub trait ResolverState {
    async fn load_tasks(&mut self) -> Result<DelayQueue<ResolveTask>>;
    async fn add_task(&mut self, task: &ResolveTask) -> Result<()>;
    async fn update_task_state(&mut self, task: &ResolveTask, state: TaskState) -> Result<()>;
    async fn update_retry_counter(&mut self, task: &ResolveTask) -> Result<()>;
}

/// Off-chain content parser
pub trait ContentParser {
    fn parse(&mut self, task: &ResolveTask, content: Vec<u8>) -> Result<()>;
}

/// Off-chain content resolver
pub struct Resolver {
    state: DBResolverState,
    queue: DelayQueue<ResolveTask>,
    downloaders: HashMap<String, Box<dyn LinkResolver>>,
    parsers: HashMap<String, Box<dyn ContentParser>>,
    connection_pool: PgPool,
}

impl Resolver {
    pub async fn new(pg_database_url: &str) -> Result<Self> {
        let connection_pool = PgPool::connect(pg_database_url).await?;
        let mut state = DBResolverState::new(connection_pool.clone()).await?;
        Ok(Self {
            queue: state.load_tasks().await?,
            state,
            downloaders: HashMap::new(),
            parsers: HashMap::new(),
            connection_pool,
        })
    }

    pub fn with_link_resolver(
        mut self,
        manifest: String,
        downloader: Box<dyn LinkResolver>,
    ) -> Self {
        self.downloaders.insert(manifest, downloader);
        self
    }

    pub fn with_parser(mut self, manifest: String, wasm_bytes: &[u8]) -> Result<Self> {
        self.parsers.insert(
            manifest,
            Box::new(WasmParser::new(wasm_bytes, self.connection_pool.clone())?),
        );
        Ok(self)
    }

    pub async fn add_task(&mut self, manifest: &str, request: OffchainData) -> Result<()> {
        let task = ResolveTask {
            manifest: manifest.to_string(),
            request,
            num_retries: 0,
        };
        self.state.add_task(&task).await?;
        self.queue.insert(task, Duration::ZERO);
        Ok(())
    }

    pub async fn run(&mut self, exit_on_completion: bool) -> Result<()> {
        loop {
            if exit_on_completion && self.queue.is_empty() {
                break;
            }

            if let Some(mut expired) = self.queue.next().await {
                let task = expired.get_mut();
                debug!("processing task {} {}", task.request.uri, self.queue.len());

                let mut downloader = None;
                let uri = task.request.uri.parse::<Uri>()?;
                if let Some(protocol) = uri.scheme() {
                    if let Some(d) = self.downloaders.get(protocol.as_str()) {
                        downloader = Some(d);
                    }
                }
                if downloader.is_none() {
                    self.state
                        .update_task_state(&task, TaskState::UnknownURI)
                        .await?;
                    debug!("no downloader for {}", task.request.uri);
                    continue;
                }

                let parser = self.parsers.get_mut(&task.manifest);
                if parser.is_none() {
                    self.state
                        .update_task_state(&task, TaskState::UnknownParser)
                        .await?;
                    debug!("no parser for {}", task.manifest);
                    continue;
                }

                let new_state = match downloader.unwrap().download(&task.request.uri).await {
                    Ok(bytes) => {
                        if parser.unwrap().parse(&task, bytes).is_err() {
                            TaskState::ParsingFailed
                        } else {
                            TaskState::Finished
                        }
                    }
                    Err(_) => match task.increment_try_counter() {
                        true => {
                            trace!(
                                "scheduling retry {} {}",
                                task.num_retries,
                                task.request.max_retries
                            );
                            self.state.update_retry_counter(&task).await?;
                            self.queue.insert(
                                task.clone(),
                                Duration::from_secs(task.request.wait_before_retry as u64),
                            );
                            TaskState::Queued
                        }
                        false => TaskState::DownloadFailed,
                    },
                };
                self.state.update_task_state(&task, new_state).await?;
            }
        }
        Ok(())
    }

    pub async fn stop() -> Result<()> {
        unimplemented!("todo: implement")
    }
}
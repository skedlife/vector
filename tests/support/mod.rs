// Using a shared mod like this is probably not the best idea, since we have to
// disable the `dead_code` lint, as we don't need all of the helpers from here
// all over the place.
#![allow(clippy::type_complexity)]
#![allow(dead_code)]

use async_trait::async_trait;
use futures::{
    compat::Sink01CompatExt, future, stream::BoxStream, FutureExt, Sink, SinkExt, StreamExt,
    TryFutureExt,
};
use futures01::{sink::Sink as Sink01, stream, sync::mpsc::Receiver, Async, Future, Stream};
use serde::{Deserialize, Serialize};
use snafu::Snafu;
use std::{
    fs::{create_dir, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
};
use tracing::{error, info};
use vector::{
    buffers::Acker,
    config::{DataType, GlobalOptions, SinkConfig, SinkContext, SourceConfig, TransformConfig},
    event::{metric::MetricValue, Value},
    shutdown::ShutdownSignal,
    sinks::{util::StreamSink, Healthcheck, VectorSink},
    sources::Source,
    test_util::{temp_dir, temp_file},
    transforms::Transform,
    Event, Pipeline,
};

pub fn sink(channel_size: usize) -> (Receiver<Event>, MockSinkConfig<Pipeline>) {
    let (tx, rx) = Pipeline::new_with_buffer(channel_size);
    let sink = MockSinkConfig::new(tx, true);
    (rx, sink)
}

pub fn sink_failing_healthcheck(
    channel_size: usize,
) -> (Receiver<Event>, MockSinkConfig<Pipeline>) {
    let (tx, rx) = Pipeline::new_with_buffer(channel_size);
    let sink = MockSinkConfig::new(tx, false);
    (rx, sink)
}

pub fn sink_dead() -> MockSinkConfig<DeadSink<Event>> {
    MockSinkConfig::new(DeadSink::new(), false)
}

pub fn source() -> (Pipeline, MockSourceConfig) {
    let (tx, rx) = Pipeline::new_with_buffer(0);
    let source = MockSourceConfig::new(rx);
    (tx, source)
}

pub fn source_with_event_counter() -> (Pipeline, MockSourceConfig, Arc<AtomicUsize>) {
    let event_counter = Arc::new(AtomicUsize::new(0));
    let (tx, rx) = Pipeline::new_with_buffer(0);
    let source = MockSourceConfig::new_with_event_counter(rx, event_counter.clone());
    (tx, source, event_counter)
}

pub fn transform(suffix: &str, increase: f64) -> MockTransformConfig {
    MockTransformConfig::new(suffix.to_owned(), increase)
}

/// Creates a file with given content
pub fn create_file(config: &str) -> PathBuf {
    let path = temp_file();
    overwrite_file(path.clone(), config);
    path
}

/// Overwrites file with given content
pub fn overwrite_file(path: PathBuf, config: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .unwrap();

    file.write_all(config.as_bytes()).unwrap();
    file.flush().unwrap();
    file.sync_all().unwrap();
}

pub fn create_directory() -> PathBuf {
    let path = temp_dir();
    create_dir(path.clone()).unwrap();
    path
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MockSourceConfig {
    #[serde(skip)]
    receiver: Arc<Mutex<Option<Receiver<Event>>>>,
    #[serde(skip)]
    event_counter: Option<Arc<AtomicUsize>>,
    #[serde(skip)]
    data_type: Option<DataType>,
}

impl MockSourceConfig {
    pub fn new(receiver: Receiver<Event>) -> Self {
        Self {
            receiver: Arc::new(Mutex::new(Some(receiver))),
            event_counter: None,
            data_type: Some(DataType::Any),
        }
    }

    pub fn new_with_event_counter(
        receiver: Receiver<Event>,
        event_counter: Arc<AtomicUsize>,
    ) -> Self {
        Self {
            receiver: Arc::new(Mutex::new(Some(receiver))),
            event_counter: Some(event_counter),
            data_type: Some(DataType::Any),
        }
    }

    pub fn set_data_type(&mut self, data_type: DataType) {
        self.data_type = Some(data_type)
    }
}

#[async_trait]
#[typetag::serde(name = "mock")]
impl SourceConfig for MockSourceConfig {
    async fn build(
        &self,
        _name: &str,
        _globals: &GlobalOptions,
        shutdown: ShutdownSignal,
        out: Pipeline,
    ) -> Result<Source, vector::Error> {
        let wrapped = self.receiver.clone();
        let event_counter = self.event_counter.clone();
        let mut recv = wrapped.lock().unwrap().take().unwrap();
        let mut shutdown = Some(shutdown.unit_error().boxed().compat());
        let mut _token = None;
        let source =
            futures01::future::lazy(move || {
                stream::poll_fn(move || {
                    if let Some(until) = shutdown.as_mut() {
                        match until.poll() {
                            Ok(Async::Ready(res)) => {
                                _token = Some(res);
                                shutdown.take();
                                recv.close();
                            }
                            Err(_) => {
                                shutdown.take();
                            }
                            Ok(Async::NotReady) => {}
                        }
                    }

                    recv.poll()
                })
                .map(move |x| {
                    if let Some(counter) = &event_counter {
                        counter.fetch_add(1, Ordering::Relaxed);
                    }
                    x
                })
                .forward(out.sink_map_err(
                    |error| error!(message = "Error sending in sink..", error = ?error),
                ))
                .map(|_| info!("Finished sending."))
            });
        Ok(Box::new(source))
    }

    fn output_type(&self) -> DataType {
        self.data_type.clone().unwrap()
    }

    fn source_type(&self) -> &'static str {
        "mock"
    }
}

pub struct MockTransform {
    suffix: String,
    increase: f64,
}

impl Transform for MockTransform {
    fn transform(&mut self, mut event: Event) -> Option<Event> {
        match &mut event {
            Event::Log(log) => {
                let mut v = log
                    .get(vector::config::log_schema().message_key())
                    .unwrap()
                    .to_string_lossy();
                v.push_str(&self.suffix);
                log.insert(vector::config::log_schema().message_key(), Value::from(v));
            }
            Event::Metric(metric) => match metric.value {
                MetricValue::Counter { ref mut value } => {
                    *value += self.increase;
                }
                MetricValue::Distribution {
                    ref mut values,
                    ref mut sample_rates,
                    statistic: _,
                } => {
                    values.push(self.increase);
                    sample_rates.push(1);
                }
                MetricValue::AggregatedHistogram {
                    ref mut count,
                    ref mut sum,
                    ..
                } => {
                    *count += 1;
                    *sum += self.increase;
                }
                MetricValue::AggregatedSummary {
                    ref mut count,
                    ref mut sum,
                    ..
                } => {
                    *count += 1;
                    *sum += self.increase;
                }
                MetricValue::Gauge { ref mut value, .. } => {
                    *value += self.increase;
                }
                MetricValue::Set { ref mut values, .. } => {
                    values.insert(self.suffix.clone());
                }
            },
        };
        Some(event)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MockTransformConfig {
    suffix: String,
    increase: f64,
}

impl MockTransformConfig {
    pub fn new(suffix: String, increase: f64) -> Self {
        Self { suffix, increase }
    }
}

#[async_trait]
#[typetag::serde(name = "mock")]
impl TransformConfig for MockTransformConfig {
    async fn build(&self) -> Result<Box<dyn Transform>, vector::Error> {
        Ok(Box::new(MockTransform {
            suffix: self.suffix.clone(),
            increase: self.increase,
        }))
    }

    fn input_type(&self) -> DataType {
        DataType::Any
    }

    fn output_type(&self) -> DataType {
        DataType::Any
    }

    fn transform_type(&self) -> &'static str {
        "mock"
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MockSinkConfig<T>
where
    T: Sink01<SinkItem = Event> + std::fmt::Debug + Clone + Send + 'static,
    <T as Sink01>::SinkError: std::fmt::Debug,
{
    #[serde(skip)]
    sink: Option<T>,
    #[serde(skip)]
    healthy: bool,
}

impl<T> MockSinkConfig<T>
where
    T: Sink01<SinkItem = Event> + std::fmt::Debug + Clone + Send + 'static,
    <T as Sink01>::SinkError: std::fmt::Debug,
{
    pub fn new(sink: T, healthy: bool) -> Self {
        Self {
            sink: Some(sink),
            healthy,
        }
    }
}

#[derive(Debug, Snafu)]
enum HealthcheckError {
    #[snafu(display("unhealthy"))]
    Unhealthy,
}

#[async_trait]
#[typetag::serialize(name = "mock")]
impl<T> SinkConfig for MockSinkConfig<T>
where
    T: Sink01<SinkItem = Event> + std::fmt::Debug + Clone + Send + Sync + 'static,
    <T as Sink01>::SinkError: std::fmt::Debug,
{
    async fn build(&self, cx: SinkContext) -> Result<(VectorSink, Healthcheck), vector::Error> {
        let sink = MockSink {
            acker: cx.acker(),
            sink: self.sink.clone().unwrap().sink_compat(),
        };

        let healthcheck = if self.healthy {
            future::ok(())
        } else {
            future::err(HealthcheckError::Unhealthy.into())
        };

        Ok((VectorSink::Stream(Box::new(sink)), healthcheck.boxed()))
    }

    fn input_type(&self) -> DataType {
        DataType::Any
    }

    fn sink_type(&self) -> &'static str {
        "mock"
    }

    fn typetag_deserialize(&self) {
        unimplemented!("not intended for use in real configs")
    }
}

struct MockSink<S> {
    acker: Acker,
    sink: S,
}

#[async_trait]
impl<S> StreamSink for MockSink<S>
where
    S: Sink<Event> + Send + std::marker::Unpin,
    <S as Sink<Event>>::Error: std::fmt::Debug,
{
    async fn run(&mut self, mut input: BoxStream<'_, Event>) -> Result<(), ()> {
        while let Some(event) = input.next().await {
            if let Err(error) = self.sink.send(event).await {
                error!(message = "Ingesting an event failed at mock sink.", ?error);
            }

            self.acker.ack(1);
        }

        Ok(())
    }
}

/// Represents a sink that's never ready.
/// Useful to simulate an upstream sink server that is down.
#[derive(Debug, Clone)]
pub struct DeadSink<T>(std::marker::PhantomData<T>);

impl<T> DeadSink<T> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<T> Sink01 for DeadSink<T> {
    type SinkItem = T;
    type SinkError = &'static str;

    fn start_send(
        &mut self,
        item: Self::SinkItem,
    ) -> futures01::StartSend<Self::SinkItem, Self::SinkError> {
        Ok(futures01::AsyncSink::NotReady(item))
    }

    fn poll_complete(&mut self) -> futures01::Poll<(), Self::SinkError> {
        Ok(futures01::Async::Ready(()))
    }
}

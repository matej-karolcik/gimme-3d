use std::sync::Arc;

use prometheus::{
    HistogramOpts, HistogramVec, IntCounter, IntCounterVec, IntGauge, Opts, Registry,
};
use warp::{Rejection, Reply};

pub struct Metrics {
    incoming_requests: IntCounter,
    connected_clients: IntGauge,
    response_code_collector: IntCounterVec,
    response_time_collector: HistogramVec,
    registry: Registry,
}

impl Metrics {
    pub fn new_arc() -> Arc<Self> {
        let incoming_requests = IntCounter::new("incoming_requests", "Incoming Requests")
            .expect("metric can be created");
        let connected_clients =
            IntGauge::new("connected_clients", "Connected Clients").expect("metric can be created");
        let response_code_collector = IntCounterVec::new(
            Opts::new("response_code", "Response Codes"),
            &["env", "statuscode", "type"],
        )
        .expect("metric can be created");
        let response_time_collector = HistogramVec::new(
            HistogramOpts::new("response_time", "Response Times"),
            &["env"],
        )
        .expect("metric can be created");

        let registry = Registry::new();

        registry
            .register(Box::new(incoming_requests.clone()))
            .expect("collector can be registered");
        registry
            .register(Box::new(connected_clients.clone()))
            .expect("collector can be registered");
        registry
            .register(Box::new(response_code_collector.clone()))
            .expect("collector can be registered");
        registry
            .register(Box::new(response_time_collector.clone()))
            .expect("collector can be registered");

        Arc::new(Metrics {
            incoming_requests,
            connected_clients,
            response_code_collector,
            response_time_collector,
            registry,
        })
    }

    async fn metrics_handler(m: Arc<Metrics>) -> Result<impl Reply, Rejection> {
        use prometheus::Encoder;
        let encoder = prometheus::TextEncoder::new();

        let mut buffer = Vec::new();
        if let Err(e) = encoder.encode(&m.registry.gather(), &mut buffer) {
            eprintln!("could not encode custom metrics: {}", e);
        };
        let mut res = String::from_utf8(buffer.clone()).unwrap_or_else(|e| {
            eprintln!("custom metrics could not be from_utf8'd: {}", e);
            String::default()
        });
        buffer.clear();

        let mut buffer = Vec::new();
        if let Err(e) = encoder.encode(&prometheus::gather(), &mut buffer) {
            eprintln!("could not encode prometheus metrics: {}", e);
        };
        let res_custom = String::from_utf8(buffer.clone()).unwrap_or_else(|e| {
            eprintln!("prometheus metrics could not be from_utf8'd: {}", e);
            String::default()
        });
        buffer.clear();

        res.push_str(&res_custom);
        Ok(res)
    }
}

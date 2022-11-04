use std::{collections::HashMap, mem::MaybeUninit, sync::Arc, time::Duration};

use opentelemetry::{
    sdk::{
        trace::{self, RandomIdGenerator, Sampler, Tracer},
        Resource,
    },
    KeyValue,
};
use opentelemetry_otlp::{Protocol, WithExportConfig};

pub struct Lightstep();

#[macro_export]
macro_rules! tracer {
    ($name:expr) => {{
        opentelemetry::global::set_text_map_propagator(
            opentelemetry::sdk::propagation::TraceContextPropagator::new(),
        );
        $crate::Lightstep::get($name, &env!("VERGEN_GIT_SHA_SHORT"))
    }};
}

impl Lightstep {
    /// Gets a reference to the tracer.
    ///
    /// # Panics
    ///
    /// Panics if the tracer can not be initialized.
    #[allow(clippy::must_use_candidate)]
    pub fn get(service: &str, version: &str) -> Arc<Tracer> {
        static mut SINGLETON: MaybeUninit<Arc<Tracer>> = MaybeUninit::uninit();
        static mut INIT: bool = false;

        unsafe {
            if !INIT {
                SINGLETON.write(Arc::new({
                    opentelemetry_otlp::new_pipeline()
                        .tracing()
                        .with_exporter(
                            opentelemetry_otlp::new_exporter()
                            .http()
                            .with_http_client(reqwest::Client::new())
                            .with_headers({
                                let mut map = HashMap::new();
                                map.insert("lightstep-access-token".to_string(), std::env::var("LIGHTSTEP_ACCESS_TOKEN")
                                .expect("Expected the LIGHTSTEP_ACCESS_TOKEN in the environment"));
                                map
                            })
                            .with_endpoint("https://ingest.lightstep.com:443/traces/otlp/v0.9")
                            .with_protocol(Protocol::HttpBinary)
                            .with_timeout(Duration::from_secs(3))
                        )
                        .with_trace_config(
                            trace::config()
                                .with_sampler(Sampler::AlwaysOn)
                                .with_id_generator(RandomIdGenerator::default())
                                .with_max_events_per_span(64)
                                .with_max_attributes_per_span(16)
                                .with_max_events_per_span(16)
                                .with_resource(Resource::new(vec![
                                    KeyValue::new("service.name", service.to_string()),
                                    KeyValue::new("service.version", version.to_string()),
                                ])),
                        )
                        .install_batch(opentelemetry::runtime::Tokio).unwrap()
                }));
                INIT = true;
            }
            SINGLETON.assume_init_ref().clone()
        }
    }
}

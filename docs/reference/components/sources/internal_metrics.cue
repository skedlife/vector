package metadata

components: sources: internal_metrics: {
	title:       "Internal Metrics"
	description: "Exports [Prometheus](\(urls.prometheus)) metrics about Vector components and the Vector instance itself. Must be paired with a [`prometheus` sink](../../sinks/prometheus)."

	classes: {
		commonly_used: true
		delivery:      "at_least_once"
		deployment_roles: ["aggregator", "daemon", "sidecar"]
		development:   "beta"
		egress_method: "batch"
	}

	features: {
		collect: {
			checkpoint: enabled: false
			from: {
				name:     "host"
				thing:    "a \(name)"
				url:      urls.host
				versions: null
			}
		}
		multiline: enabled: false
	}

	support: {
		platforms: {
			"aarch64-unknown-linux-gnu":  true
			"aarch64-unknown-linux-musl": true
			"x86_64-apple-darwin":        false
			"x86_64-pc-windows-msv":      false
			"x86_64-unknown-linux-gnu":   true
			"x86_64-unknown-linux-musl":  true
		}

		requirements: []
		warnings: []
		notices: []
	}

	output: metrics: {
		_default_tags: {
			instance: {
				description: "The Vector instance identified by host and port."
				required:    true
				examples: [_values.instance]
			}
			job: {
				description: "The name of the job producing Vector metrics."
				type: string: default: "vector"
			}
		}

		_component_tags: {
			component_kind: {
				description: "The component's kind (`source`, `sink`, or `transform`)."
				required:    true
				type: string: enum: {
					sink:      "Sink component."
					source:    "Source component."
					transform: "Transform component."
				}
			}
			component_name: {
				description: "The name of the component as specified in the Vector configuration."
				required:    true
				type: string: examples: ["file_source", "splunk_sink"]
			}
			component_type: {
				description: "The type of component (source, transform, or sink)."
				required:    true
				type: string: examples: ["file", "http", "honeycomb", "splunk_hec"]
			}
		}

		api_started_total: {
			description: "The number of times the Vector GraphQL API has been started."
			required: false
			type: "counter"
		}
		auto_concurrency_in_flight: {
			type: "histogram"
		auto_concurrency_limit: {
			type: "histogram"
		}
		auto_concurrency_averaged_rtt: {
			type: "histogram"
		}
		auto_concurrency_observed_rtt: {
			type: "histogram"
		}
		collect_duration_nanoseconds: {
			type: "histogram"
		}
		events_processed_total: {
			description: "The total number of events processed by this component."
			required:    true
			type:        "counter"
			tags:        _default_tags & _component_tags
		}
		http_error_response_total: {
			type: "counter"
		}
		http_request_errors_total: {
			type: "counter"
		}
		memory_used: {
			type: "gauge"
		}
		open_connections: {
			type: "gauge"
		}
		parse_errors_total: {
			type: "counter"
		}
		processed_bytes_total: {
			description: "The total number of bytes processed by the component."
			required:    true
			type:        "counter"
			tags:        _default_tags & _component_tags
		}
		processing_errors_total: {
			type: "counter"
		}
		request_duration_nanoseconds: {
			type: "histogram"
		}
		requests_completed_total: {
			type: "counter"
		}
		uptime_seconds: {
			description: "The total number of seconds the Vector instance has been up."
			required:    true
			type:        "gauge"
			tags:        _default_tags
		}
	}
}

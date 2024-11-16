use clap::Parser;
use tracing::level_filters::LevelFilter;

#[derive(Debug, Clone, Parser)]
pub struct OperatorConfig {
    /// `HCloud` API token.
    #[arg(short = 't', long, env = "LB_TRACKER_HCLOUD_TOKEN")]
    pub hcloud_token: String,

    /// Label name where to find the balancer name.
    #[arg(
        short = 'l',
        long,
        env = "LB_TRACKER_BALANCER_LABEL_NAME",
        default_value = "lb-tracker/balancer"
    )]
    pub balancer_label_name: String,

    /// Label name where to find node selector for node port services.
    #[arg(
        long,
        env = "LB_TRACKER_NODE_PORT_NODE_SELECTOR_NAMES",
        default_value = "lb-tracker/node-selector"
    )]
    pub node_port_node_selector_label: String,

    /// Default load balancer healthcheck retries cound.
    #[arg(long, env = "LB_TRACKER_DEFAULT_LB_RETRIES", default_value = "3")]
    pub default_lb_retries: i32,

    /// Default load balancer healthcheck timeout.
    #[arg(long, env = "LB_TRACKER_DEFAULT_LB_TIMEOUT", default_value = "10")]
    pub default_lb_timeout: i32,

    /// Default load balancer healhcheck interval.
    #[arg(long, env = "LB_TRACKER_DEFAULT_LB_INTERVAL", default_value = "15")]
    pub default_lb_interval: i32,

    // Log level for the operator.
    #[arg(long, env = "LB_TRACKER_LOG_LEVEL", default_value = "INFO")]
    pub log_level: LevelFilter,
}

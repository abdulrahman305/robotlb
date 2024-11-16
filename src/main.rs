#![warn(
    // Base lints.
    clippy::all,
    // Some pedantic lints.
    clippy::pedantic,
    // New lints which are cool.
    clippy::nursery,
)]
#![
    allow(
        // I don't care about this.
        clippy::module_name_repetitions, 
        // Yo, the hell you should put
        // it in docs, if signature is clear as sky.
        clippy::missing_errors_doc
    )
]

use clap::Parser;
use config::OperatorConfig;
use error::{LBTrackerError, LBTrackerResult};
use futures::StreamExt;
use hcloud::apis::configuration::Configuration as HCloudConfig;
use k8s_openapi::api::core::v1::{Node, Service};
use kube::{
    api::ListParams,
    runtime::{controller::Action, watcher, Controller},
    Resource, ResourceExt,
};
use label_filter::LabelFilter;
use lb::LoadBalancer;
use std::{str::FromStr, sync::Arc, time::Duration};
use tracing::Instrument;

pub mod config;
pub mod consts;
pub mod error;
pub mod finalizers;
pub mod label_filter;
pub mod lb;

#[tokio::main]
async fn main() -> LBTrackerResult<()> {
    dotenvy::dotenv().ok();
    let operator_config = config::OperatorConfig::parse();
    tracing_subscriber::fmt()
        .with_max_level(operator_config.log_level)
        .init();

    let mut hcloud_conf = HCloudConfig::new();
    hcloud_conf.bearer_access_token = Some(operator_config.hcloud_token.clone());

    tracing::info!(
        "Starting lb-tracker operator v{}",
        env!("CARGO_PKG_VERSION")
    );
    let kube_client = kube::Client::try_default().await?;
    tracing::info!("Kube client is connected");
    watcher::Config::default();
    let context = Arc::new(CurrentContext::new(
        kube_client.clone(),
        operator_config.clone(),
        hcloud_conf,
    ));
    tracing::info!("Starting the controller");
    Controller::new(
        kube::Api::<Service>::all(kube_client),
        watcher::Config::default(),
    )
    .run(reconcile_service, on_error, context)
    .for_each(|reconcilation_result| async move {
        match reconcilation_result {
            Ok((service, _action)) => {
                tracing::info!("Reconcilation of a service {} was successful", service.name);
            }
            Err(err) => match err {
                // During reconcilation process,
                // the controller has decided to skip the service.
                kube::runtime::controller::Error::ReconcilerFailed(
                    LBTrackerError::SkipService,
                    _,
                ) => {}
                _ => {
                    tracing::error!("Error reconciling service: {:?}", err);
                }
            },
        }
    })
    .await;
    Ok(())
}

#[derive(Clone)]
pub struct CurrentContext {
    pub client: kube::Client,
    pub config: OperatorConfig,
    pub hcloud_config: HCloudConfig,
}
impl CurrentContext {
    #[must_use] pub const fn new(client: kube::Client, config: OperatorConfig, hcloud_config: HCloudConfig) -> Self {
        Self {
            client,
            config,
            hcloud_config,
        }
    }
}

/// Reconcile the service.
/// This function is called by the controller for each service.
/// It will create or update the load balancer based on the service.
/// If the service is being deleted, it will clean up the resources.
#[tracing::instrument(skip(svc,context), fields(service=svc.name_any()))]
pub async fn reconcile_service(
    svc: Arc<Service>,
    context: Arc<CurrentContext>,
) -> LBTrackerResult<Action> {
    tracing::info!("Starting service reconcilation");
    let lb = LoadBalancer::try_from_svc(&svc, &context)?;

    // If the service is being deleted, we need to clean up the resources.
    if svc.meta().deletion_timestamp.is_some() {
        tracing::info!("Service deletion detected. Cleaning up resources.");
        lb.cleanup().await?;
        finalizers::remove(context.client.clone(), &svc).await?;
        return Ok(Action::await_change());
    }

    let Some(spec) = &svc.spec else {
        tracing::warn!("Service has no spec, skipping");
        return Err(LBTrackerError::SkipService);
    };

    // Add finalizer if it's not there yet.
    if !finalizers::check(&svc) {
        finalizers::add(context.client.clone(), &svc).await?;
    }

    // Based on the service type, we will reconcile the load balancer.
    match spec.type_.as_deref() {
        Some("NodePort") => {
            reconcile_node_port(lb, svc.clone(), context)
                .in_current_span()
                .await
        }
        Some("LoadBalancer") => {
            reconcile_load_balancer(lb, svc.clone())
                .in_current_span()
                .await
        }
        _ => {
            tracing::warn!("Service type is not supported, skipping");
            Err(LBTrackerError::UnsupportedServiceType)
        }
    }
}

/// Reconcile the load balancer type of service.
/// This function will wait until the service has IP address.
/// Then it will create or update the load balancer.
pub async fn reconcile_load_balancer(
    mut lb: LoadBalancer,
    svc: Arc<Service>,
) -> LBTrackerResult<Action> {
    let Some(svc_ingress) = svc.status.clone().unwrap_or_default().load_balancer.unwrap_or_default().ingress else {
        tracing::warn!("Service hasn't yet got IP, skipping");
        return Err(LBTrackerError::SkipService);
    };
    for ingress in svc_ingress {
        if ingress.hostname.is_some(){
            tracing::warn!("Hostname based loadbalancing is not supported, skipping");
            continue;
        }
        if let Some(ip) = &ingress.ip {
            lb.add_target(ip);
        }
    }
    for port in svc.spec.clone().unwrap_or_default().ports.unwrap_or_default(){
        lb.add_service(port.port, port.port);
    }
    lb.reconcile().await?;
    Ok(Action::requeue(Duration::from_secs(10)))
}

/// Reconcile the NodePort type of service.
/// This function will find the nodes based on the node selector
/// and create or update the load balancer.
pub async fn reconcile_node_port(
    mut lb: LoadBalancer,
    svc: Arc<Service>,
    context: Arc<CurrentContext>,
) -> LBTrackerResult<Action> {
    let label_filter = svc
        .annotations()
        .get(consts::LB_NODE_SELECTOR)
        .map(String::as_str)
        .map(LabelFilter::from_str)
        .transpose()?
        .unwrap_or_default();
    let nodes_api = kube::Api::<Node>::all(context.client.clone());
    let nodes = nodes_api
        .list(&ListParams::default())
        .await?
        .into_iter()
        .filter(|node| label_filter.check(node.labels()))
        .collect::<Vec<_>>();

    for node in nodes {
        let Some(status) = node.status else {
            continue;
        };
        let Some(addresses) = status.addresses else {
            continue;
        };
        for addr in addresses {
            if addr.type_ == "InternalIP" {
                lb.add_target(&addr.address);
            }
        }
    }

    for port in svc.spec.clone().unwrap_or_default().ports.unwrap_or_default() {
        let Some(node_port) = port.node_port else {
            tracing::warn!("Node port is not set for target_port {}", port.port);
            continue;
        };
        lb.add_service(port.port, node_port);
    }

    lb.reconcile().await?;
    Ok(Action::requeue(Duration::from_secs(10)))
}

/// Handle the error during reconcilation.
#[allow(clippy::needless_pass_by_value)]
fn on_error(_: Arc<Service>, error: &LBTrackerError, _context: Arc<CurrentContext>) -> Action {
    match error {
        LBTrackerError::SkipService => Action::requeue(Duration::from_secs(60 * 5)),
        _ => Action::requeue(Duration::from_secs(60)),
    }
}

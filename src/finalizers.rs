use k8s_openapi::{api::core::v1::Service, serde_json::json};
use kube::{
    api::{Patch, PatchParams},
    Api, Client, ResourceExt,
};

use crate::{consts, error::LBTrackerResult};

pub async fn add(client: Client, svc: &Service) -> LBTrackerResult<()> {
    let api = Api::<Service>::namespaced(client, svc.namespace().unwrap().as_str());
    let patch = json!({
        "metadata": {
            "finalizers": [consts::FINALIZER_NAME]
        }
    });
    api.patch(
        svc.name_any().as_str(),
        &PatchParams::default(),
        &Patch::Merge(patch),
    )
    .await?;
    Ok(())
}

pub fn check(service: &Service) -> bool {
    service
        .metadata
        .finalizers
        .as_ref()
        .map_or(false, |finalizers| {
            finalizers.contains(&consts::FINALIZER_NAME.to_string())
        })
}

pub async fn remove(client: Client, svc: &Service) -> LBTrackerResult<()> {
    let api = Api::<Service>::namespaced(client, svc.namespace().unwrap().as_str());
    let finalizers = svc
        .finalizers()
        .into_iter()
        .filter(|item| item.as_str() != consts::FINALIZER_NAME)
        .collect::<Vec<_>>();
    let patch = json!({
        "metadata": {
            "finalizers": finalizers
        }
    });
    api.patch(
        svc.name_any().as_str(),
        &PatchParams::default(),
        &Patch::Merge(patch),
    )
    .await?;
    Ok(())
}

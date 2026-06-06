use std::collections::BTreeMap;
use anyhow::Context;
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{
    Container, EnvVar, EnvVarSource, PersistentVolumeClaim, PersistentVolumeClaimSpec,
    PodSpec, PodTemplateSpec, ResourceRequirements, Secret, SecretKeySelector, Service,
    ServicePort, ServiceSpec, Volume, VolumeMount,
};
use k8s_openapi::api::networking::v1::{
    NetworkPolicy, NetworkPolicyEgressRule, NetworkPolicyIngressRule, NetworkPolicyPeer,
    NetworkPolicyPort, NetworkPolicySpec,
};
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{LabelSelector, ObjectMeta};
use kube::api::{DeleteParams, PostParams};
use kube::{Api, Client};
use rand::RngCore;

pub struct Provisioner {
    client: Client,
    engine_image: String,
}

impl Provisioner {
    pub async fn new(engine_image: String) -> anyhow::Result<Self> {
        let client = Client::try_default()
            .await
            .context("connect kubernetes — set KUBECONFIG or run in-cluster")?;
        Ok(Self {
            client,
            engine_image,
        })
    }

    pub async fn ensure_tenant(&self, namespace: &str, tenant_id: &str) -> anyhow::Result<()> {
        let ns_api: Api<k8s_openapi::api::core::v1::Namespace> = Api::all(self.client.clone());
        let ns = k8s_openapi::api::core::v1::Namespace {
            metadata: ObjectMeta {
                name: Some(namespace.to_string()),
                labels: Some(BTreeMap::from([
                    ("houston.ai/tenant".into(), tenant_id.into()),
                    ("houston.ai/managed-by".into(), "houston-control-plane".into()),
                ])),
                ..Default::default()
            },
            ..Default::default()
        };

        match ns_api.create(&PostParams::default(), &ns).await {
            Ok(_) => {}
            Err(kube::Error::Api(err)) if err.code == 409 => {}
            Err(e) => return Err(e.into()),
        }

        let np_api: Api<NetworkPolicy> = Api::namespaced(self.client.clone(), namespace);
        let policy = tenant_network_policy(namespace);
        match np_api.create(&PostParams::default(), &policy).await {
            Ok(_) => {}
            Err(kube::Error::Api(err)) if err.code == 409 => {}
            Err(e) => return Err(e.into()),
        }

        Ok(())
    }

    pub async fn deploy_agent(
        &self,
        namespace: &str,
        agent_id: &str,
        provider: &str,
        api_key: Option<&str>,
    ) -> anyhow::Result<(String, String)> {
        let service_name = format!("agent-{agent_id}");
        let secret_name = format!("{service_name}-secret");
        let pvc_name = format!("{service_name}-data");
        let deployment_name = service_name.clone();
        let engine_token = random_token();

        let mut data = BTreeMap::from([(
            "HOUSTON_ENGINE_TOKEN".to_string(),
            engine_token.clone(),
        )]);
        if let Some(key) = api_key {
            if let Some(env_key) = provider_env_key(provider) {
                data.insert(env_key.to_string(), key.to_string());
            }
        }

        let secret = Secret {
            metadata: ObjectMeta {
                name: Some(secret_name.clone()),
                namespace: Some(namespace.to_string()),
                ..Default::default()
            },
            string_data: Some(data),
            ..Default::default()
        };
        let secrets: Api<Secret> = Api::namespaced(self.client.clone(), namespace);
        match secrets.create(&PostParams::default(), &secret).await {
            Ok(_) => {}
            Err(kube::Error::Api(err)) if err.code == 409 => {
                secrets
                    .replace(&secret_name, &PostParams::default(), &secret)
                    .await?;
            }
            Err(e) => return Err(e.into()),
        }

        let pvc = PersistentVolumeClaim {
            metadata: ObjectMeta {
                name: Some(pvc_name.clone()),
                namespace: Some(namespace.to_string()),
                ..Default::default()
            },
            spec: Some(PersistentVolumeClaimSpec {
                access_modes: Some(vec!["ReadWriteOnce".into()]),
                resources: Some(k8s_openapi::api::core::v1::VolumeResourceRequirements {
                    requests: Some(BTreeMap::from([(
                        "storage".into(),
                        Quantity("2Gi".into()),
                    )])),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let pvcs: Api<PersistentVolumeClaim> = Api::namespaced(self.client.clone(), namespace);
        match pvcs.create(&PostParams::default(), &pvc).await {
            Ok(_) => {}
            Err(kube::Error::Api(err)) if err.code == 409 => {}
            Err(e) => return Err(e.into()),
        }

        let deployment = agent_deployment(
            &deployment_name,
            namespace,
            agent_id,
            &self.engine_image,
            &secret_name,
            &pvc_name,
            provider,
        );
        let deployments: Api<Deployment> = Api::namespaced(self.client.clone(), namespace);
        match deployments
            .create(&PostParams::default(), &deployment)
            .await
        {
            Ok(_) => {}
            Err(kube::Error::Api(err)) if err.code == 409 => {
                deployments
                    .replace(&deployment_name, &PostParams::default(), &deployment)
                    .await?;
            }
            Err(e) => return Err(e.into()),
        }

        let service = Service {
            metadata: ObjectMeta {
                name: Some(service_name.clone()),
                namespace: Some(namespace.to_string()),
                labels: Some(BTreeMap::from([
                    ("app".into(), deployment_name.clone()),
                    ("houston.ai/agent".into(), agent_id.into()),
                ])),
                ..Default::default()
            },
            spec: Some(ServiceSpec {
                selector: Some(BTreeMap::from([("app".into(), deployment_name.clone())])),
                ports: Some(vec![ServicePort {
                    name: Some("http".into()),
                    port: 7777,
                    target_port: Some(k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(7777)),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };
        let services: Api<Service> = Api::namespaced(self.client.clone(), namespace);
        match services.create(&PostParams::default(), &service).await {
            Ok(_) => {}
            Err(kube::Error::Api(err)) if err.code == 409 => {
                services
                    .replace(&service_name, &PostParams::default(), &service)
                    .await?;
            }
            Err(e) => return Err(e.into()),
        }

        // Fire and forget: return as soon as K8s accepts the resources.
        // The pod will reach Running state asynchronously.
        Ok((service_name, engine_token))
    }

    pub async fn delete_agent(&self, namespace: &str, service_name: &str) -> anyhow::Result<()> {
        let dp = DeleteParams::default();
        let deployments: Api<Deployment> = Api::namespaced(self.client.clone(), namespace);
        let _ = deployments.delete(service_name, &dp).await;
        let services: Api<Service> = Api::namespaced(self.client.clone(), namespace);
        let _ = services.delete(service_name, &dp).await;
        let secret_name = format!("{service_name}-secret");
        let secrets: Api<Secret> = Api::namespaced(self.client.clone(), namespace);
        let _ = secrets.delete(&secret_name, &dp).await;
        let pvc_name = format!("{service_name}-data");
        let pvcs: Api<PersistentVolumeClaim> = Api::namespaced(self.client.clone(), namespace);
        let _ = pvcs.delete(&pvc_name, &dp).await;
        Ok(())
    }

    pub fn engine_base_url(namespace: &str, service_name: &str) -> String {
        format!("http://{service_name}.{namespace}.svc.cluster.local:7777")
    }

}

fn agent_deployment(
    name: &str,
    namespace: &str,
    agent_id: &str,
    image: &str,
    secret_name: &str,
    pvc_name: &str,
    provider: &str,
) -> Deployment {
    let mut env = vec![
        env_from_secret("HOUSTON_ENGINE_TOKEN", secret_name, "HOUSTON_ENGINE_TOKEN"),
        plain_env("HOUSTON_BIND", "0.0.0.0:7777"),
        plain_env("HOUSTON_BIND_ALL", "1"),
        plain_env("HOUSTON_NO_PARENT_WATCHDOG", "1"),
        plain_env("HOUSTON_HOME", "/data/.houston"),
        plain_env("HOUSTON_DOCS", "/data/Houston"),
        plain_env("RUST_LOG", "info,houston=debug"),
    ];
    if let Some(key) = provider_env_key(provider) {
        env.push(env_from_secret(key, secret_name, key));
    }

    Deployment {
        metadata: ObjectMeta {
            name: Some(name.to_string()),
            namespace: Some(namespace.to_string()),
            labels: Some(BTreeMap::from([
                ("app".into(), name.to_string()),
                ("houston.ai/agent".into(), agent_id.into()),
                ("houston.ai/provider".into(), provider.into()),
            ])),
            ..Default::default()
        },
        spec: Some(k8s_openapi::api::apps::v1::DeploymentSpec {
            replicas: Some(1),
            selector: LabelSelector {
                match_labels: Some(BTreeMap::from([("app".into(), name.to_string())])),
                ..Default::default()
            },
                template: PodTemplateSpec {
                metadata: Some(ObjectMeta {
                    labels: Some(BTreeMap::from([
                        ("app".into(), name.to_string()),
                        ("houston.ai/agent".into(), agent_id.into()),
                    ])),
                    ..Default::default()
                }),
                spec: Some(PodSpec {
                    automount_service_account_token: Some(false),
                    security_context: Some(k8s_openapi::api::core::v1::PodSecurityContext {
                        run_as_user: Some(1000),
                        fs_group: Some(1000),
                        ..Default::default()
                    }),
                    containers: vec![Container {
                        name: "engine".into(),
                        image: Some(image.to_string()),
                        image_pull_policy: Some("IfNotPresent".into()),
                        ports: Some(vec![k8s_openapi::api::core::v1::ContainerPort {
                            container_port: 7777,
                            ..Default::default()
                        }]),
                        env: Some(env),
                        volume_mounts: Some(vec![
                            VolumeMount {
                                name: "houston-data".into(),
                                mount_path: "/data/.houston".into(),
                                ..Default::default()
                            },
                            VolumeMount {
                                name: "houston-data".into(),
                                mount_path: "/data/Houston".into(),
                                sub_path: Some("Houston".into()),
                                ..Default::default()
                            },
                        ]),
                        resources: Some(ResourceRequirements {
                            requests: Some(BTreeMap::from([
                                ("cpu".into(), Quantity("100m".into())),
                                ("memory".into(), Quantity("256Mi".into())),
                            ])),
                            limits: Some(BTreeMap::from([
                                ("cpu".into(), Quantity("500m".into())),
                                ("memory".into(), Quantity("512Mi".into())),
                            ])),
                            ..Default::default()
                        }),
                        security_context: Some(k8s_openapi::api::core::v1::SecurityContext {
                            allow_privilege_escalation: Some(false),
                            read_only_root_filesystem: Some(false),
                            ..Default::default()
                        }),
                        readiness_probe: Some(k8s_openapi::api::core::v1::Probe {
                            tcp_socket: Some(k8s_openapi::api::core::v1::TCPSocketAction {
                                port: k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(7777),
                                ..Default::default()
                            }),
                            initial_delay_seconds: Some(5),
                            period_seconds: Some(5),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                    volumes: Some(vec![Volume {
                        name: "houston-data".into(),
                        persistent_volume_claim: Some(
                            k8s_openapi::api::core::v1::PersistentVolumeClaimVolumeSource {
                                claim_name: pvc_name.to_string(),
                                ..Default::default()
                            },
                        ),
                        ..Default::default()
                    }]),
                    ..Default::default()
                }),
            },
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn tenant_network_policy(_namespace: &str) -> NetworkPolicy {
    NetworkPolicy {
        metadata: ObjectMeta {
            name: Some("tenant-isolation".into()),
            ..Default::default()
        },
        spec: Some(NetworkPolicySpec {
            pod_selector: LabelSelector {
                ..Default::default()
            },
            policy_types: Some(vec!["Ingress".into(), "Egress".into()]),
            ingress: Some(vec![NetworkPolicyIngressRule {
                from: Some(vec![NetworkPolicyPeer {
                    namespace_selector: Some(LabelSelector {
                        match_labels: Some(BTreeMap::from([(
                            "houston.ai/component".into(),
                            "control-plane".into(),
                        )])),
                        ..Default::default()
                    }),
                    ..Default::default()
                }]),
                ports: Some(vec![NetworkPolicyPort {
                    protocol: Some("TCP".into()),
                    port: Some(k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(7777)),
                    ..Default::default()
                }]),
                ..Default::default()
            }]),
            egress: Some(vec![
                NetworkPolicyEgressRule {
                    to: Some(vec![NetworkPolicyPeer {
                        namespace_selector: Some(LabelSelector {
                            ..Default::default()
                        }),
                        ..Default::default()
                    }]),
                    ports: Some(vec![
                        np_port("TCP", 443),
                        np_port("TCP", 53),
                        np_port("UDP", 53),
                    ]),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn np_port(protocol: &str, port: i32) -> NetworkPolicyPort {
    NetworkPolicyPort {
        protocol: Some(protocol.into()),
        port: Some(k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(port)),
        ..Default::default()
    }
}

fn plain_env(name: &str, value: &str) -> EnvVar {
    EnvVar {
        name: name.into(),
        value: Some(value.into()),
        ..Default::default()
    }
}

fn env_from_secret(name: &str, secret: &str, key: &str) -> EnvVar {
    EnvVar {
        name: name.into(),
        value_from: Some(EnvVarSource {
            secret_key_ref: Some(SecretKeySelector {
                name: secret.into(),
                key: key.into(),
                optional: Some(false),
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn provider_env_key(provider: &str) -> Option<&'static str> {
    match provider.to_lowercase().as_str() {
        "anthropic" | "claude" => Some("ANTHROPIC_API_KEY"),
        "openai" | "codex" | "gpt" => Some("OPENAI_API_KEY"),
        "gemini" | "google" => Some("GEMINI_API_KEY"),
        _ => None,
    }
}

fn random_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

mod hex {
    pub fn encode(bytes: [u8; 32]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }
}

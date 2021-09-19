use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct DockerToken {
    pub token: String,
    pub access_token: String,
    pub expires_in: i32,
    pub issued_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DockerImageConfig {
    architecture: String,
    pub config: DockerContainerConfig,
    container: Option<String>,
    container_config: Option<DockerContainerConfig>,
    created: String,
    docker_version: Option<String>,
    history: Vec<DockerImageHistory>,
    os: String,
    rootfs: DockerImageRootfs,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct DockerContainerConfig {
    Hostname: Option<String>,
    Domainname: Option<String>,
    User: Option<String>,
    AttachStdin: Option<bool>,
    AttachStdout: Option<bool>,
    AttachStderr: Option<bool>,
    ExposedPorts: Option<Value>,
    Tty: Option<bool>,
    OpenStdin: Option<bool>,
    StdinOnce: Option<bool>,
    pub Env: Vec<String>,
    pub Cmd: Vec<String>,
    Image: Option<String>,
    Volumes: Option<Vec<String>>,
    WorkingDir: Option<String>,
    pub Entrypoint: Option<Vec<String>>,
    OnBuild: Option<Vec<String>>,
    Labels: Option<HashMap<String, String>>,
    StopSignal: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DockerImageHistory {
    created: String,
    created_by: String,
    empty_layer: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DockerImageRootfs {
    r#type: String,
    diff_ids: Vec<String>,
}

use reqwest;
use log::{error, info, debug};
use oci_spec::image::ImageManifest;
use reqwest::header::{ACCEPT, AUTHORIZATION};
use crate::models::docker::DockerToken;
use crate::storage::image::{store_manifest, load_manifest, store_config, load_config, save_layer};
use clap::ArgMatches;

struct PullRequest {
    image: String,
    tag: String,
    scope: String,
    token: Option<String>,
}

impl PullRequest {
    pub fn new(scope: String, image: String, tag: String) -> PullRequest {
        PullRequest {
            image,
            tag,
            scope,
            token: None,
        }
    }

    pub async fn docker_hub_auth(&mut self) -> PullRequest {
        info!("Auth to Docker Hub");
        let url = format!("https://auth.docker.io/token?service=registry.docker.io&scope=repository:{}/{}:pull", self.scope, self.image);
        let resp = reqwest::get(url).await;
        match resp {
            Ok(resp) => {
                match resp.json::<DockerToken>().await {
                    Ok(data) => {
                        self.token = Some(data.token);
                        PullRequest {
                            image: self.image.clone(),
                            tag: self.tag.clone(),
                            scope: self.scope.clone(),
                            token: self.token.clone(),
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse Docker Token: {}", e);
                        PullRequest {
                            image: self.image.clone(),
                            tag: self.tag.clone(),
                            scope: self.scope.clone(),
                            token: None,
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to auth Docker Hub: {}", e);
                PullRequest {
                    image: self.image.clone(),
                    tag: self.tag.clone(),
                    scope: self.scope.clone(),
                    token: None,
                }
            }
        }
    }

    pub async fn docker_hub_manifest(&self) -> Result<ImageManifest, Box<dyn std::error::Error>> {
        info!("Fetching image manifest");
        let url = format!("https://registry.hub.docker.com/v2/{}/{}/manifests/{}", self.scope, self.image, self.tag);
        let client = reqwest::Client::new();
        let resp = client.get(url)
            .header(AUTHORIZATION, format!("Bearer {}", self.token.clone().unwrap()))
            .header(ACCEPT, "application/vnd.docker.distribution.manifest.v2+json")
            .send()
            .await;
        match resp {
            Ok(resp) => {
                let text = match resp.text().await {
                    Ok(text) => text,
                    Err(e) => {
                        error!("Failed to fetch manifest: {}", e);
                        return Err(Box::new(e));
                    }
                };
                debug!("{:?}", text);
                match store_manifest(&self.image, &self.tag, &text) {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Failed to store manifest: {}", e);
                        return Err(e);
                    }
                }
                match load_manifest(&self.image, &self.tag) {
                    Ok(manifest) => {
                        info!("Successfully fetched manifest");
                        return Ok(manifest);
                    }
                    Err(e) => {
                        error!("Failed to load manifest: {}", e);
                        return Err(e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to fetch manifest: {}", e);
                return Err(Box::new(e));
            }
        }
    }

    pub async fn fetch_layers(&self, manifest: ImageManifest) -> Result<(), Box<dyn std::error::Error>> {
        let config_digest = manifest.config().digest();
        let url = format!("https://registry.hub.docker.com/v2/{}/{}/blobs/{}", self.scope, self.image, config_digest);
        let client = reqwest::Client::new();
        let resp = client.get(url)
            .header(AUTHORIZATION, format!("Bearer {}", self.token.clone().unwrap()))
            .send()
            .await;
        match resp {
            Ok(resp) => {
                let text = match resp.text().await {
                    Ok(text) => text,
                    Err(e) => {
                        error!("Failed to fetch config: {}", e);
                        return Err(Box::new(e));
                    }
                };
                match store_config(&self.image, &self.tag, &text) {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Failed to store config: {}", e);
                        return Err(e);
                    }
                }
                match load_config(&self.image, &self.tag) {
                    Ok(config) => {
                        debug!("{:?}", config);
                        info!("Successfully fetched image config");
                    }
                    Err(e) => {
                        error!("Bad image config format: {}", e);
                        return Err(e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to fetch config: {}", e);
                return Err(Box::new(e));
            }
        }
        
        info!("Fetching image layers");
        let layers = manifest.layers();
        for layer in layers.iter() {
            let digest = layer.digest();
            let url = format!("https://registry.hub.docker.com/v2/{}/{}/blobs/{}", self.scope, self.image, digest);
            let client = reqwest::Client::new();
            let req = client.get(url)
                .header(AUTHORIZATION, format!("Bearer {}", self.token.clone().unwrap()))
                .header(ACCEPT, "application/vnd.docker.image.rootfs.diff.tar.gzip");
            info!("Fetching layer {}", digest);
            match save_layer(&self.image, &self.tag, &digest, req).await {
                Ok(_) => {
                    info!("Successfully fetched layer {}", digest);
                }
                Err(e) => {
                    error!("Failed to fetch layer: {}", e);
                    return Err(e);
                }
            }
        }
        Ok(())
    }
}

pub async fn pull(arg: &ArgMatches) -> bool {
    let image = arg.value_of("image").unwrap();
    let mut pull_request = match sscanf::scanf!(image, "{}/{}:{}", String, String, String) {
        Some((scope, image, tag)) => {
            PullRequest::new(scope, image, tag)
        }
        None => {
            error!("Invalid image format: {}", image);
            return false;
        }
    };
    pull_request.docker_hub_auth().await;
    if pull_request.token.is_none() {
        return false;
    }
    let manifest = match pull_request.docker_hub_manifest().await {
        Ok(manifest) => manifest,
        Err(_) => {
            return false;
        }
    };
    match pull_request.fetch_layers(manifest).await {
        Ok(_) => {
            true
        }
        Err(_) => {
            false
        }
    }
}

pub async fn pull_image(scope: String, image: String, tag: String) -> bool {
    let mut pull_request = PullRequest::new(scope, image, tag);
    pull_request.docker_hub_auth().await;
    if pull_request.token.is_none() {
        return false;
    }
    let manifest = match pull_request.docker_hub_manifest().await {
        Ok(manifest) => manifest,
        Err(_) => {
            return false;
        }
    };
    match pull_request.fetch_layers(manifest).await {
        Ok(_) => {
            true
        }
        Err(_) => {
            false
        }
    }
}
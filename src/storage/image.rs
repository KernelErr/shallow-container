use log::error;
use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use oci_spec::image::ImageManifest;
use crate::models::docker::DockerImageConfig;
use reqwest::RequestBuilder;
use std::io::{copy, Cursor};
use crate::storage::IMAGE_PATH;

pub fn store_manifest(image: &str, tag: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = format!("{}/{}/{}", IMAGE_PATH, image, tag);
    match create_dir_all(path.clone()) {
        Ok(_) => {
            let output = format!("{}/{}", path, "manifest.json");
            let mut f = match File::create(output.to_string()) {
                Ok(f) => f,
                Err(e) => {
                    error!("Failed to write manifest: {}", e);
                    return Err(Box::new(e));
                }
            };
            match f.write_all(content.as_bytes()) {
                Ok(_) => Ok(()),
                Err(e) => {
                    error!("Failed to write manifest: {}", e);
                    return Err(Box::new(e));
                }  
            }
        }
        Err(e) => {
            error!("Failed to create manifest directory: {}", e);
            return Err(Box::new(e));
        }
    }
}

pub fn load_manifest(image: &str, tag: &str) -> Result<ImageManifest, Box<dyn std::error::Error>> {
    let path = format!("{}/{}/{}/manifest.json", IMAGE_PATH, image, tag);
    match ImageManifest::from_file(path) {
        Ok(manifest) => Ok(manifest),
        Err(e) => {
            error!("Failed to load manifest: {}", e);
            return Err(Box::new(e));
        }
    }
}

pub fn store_config(image: &str, tag: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = format!("{}/{}/{}", IMAGE_PATH, image, tag);
    match create_dir_all(path.clone()) {
        Ok(_) => {
            let output = format!("{}/{}", path, "config.json");
            let mut f = match File::create(output.to_string()) {
                Ok(f) => f,
                Err(e) => {
                    error!("Failed to write image config: {}", e);
                    return Err(Box::new(e));
                }
            };
            match f.write_all(content.as_bytes()) {
                Ok(_) => Ok(()),
                Err(e) => {
                    error!("Failed to write image config: {}", e);
                    return Err(Box::new(e));
                }  
            }
        }
        Err(e) => {
            error!("Failed to create image config directory: {}", e);
            return Err(Box::new(e));
        }
    }
}

pub fn load_config(image: &str, tag: &str) -> Result<DockerImageConfig, Box<dyn std::error::Error>> {
    let path = format!("{}/{}/{}/config.json", IMAGE_PATH, image, tag);
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(e) => {
            error!("Failed to load image config: {}", e);
            return Err(Box::new(e));
        }
    };
    let mut data = String::new();
    match file.read_to_string(&mut data) {
        Ok(_) => {
            let image_config: DockerImageConfig = match serde_json::from_str(&data) {
                Ok(image_config) => image_config,
                Err(e) => {
                    error!("Failed to parse image config: {}", e);
                    return Err(Box::new(e));
                }
            };
            Ok(image_config)
        },
        Err(e) => {
            error!("Failed to load image config: {}", e);
            return Err(Box::new(e));
        }
    }
}

pub async fn save_layer(image: &str, tag: &str, digest: &str, req: RequestBuilder) -> Result<(), Box<dyn std::error::Error>> {
    let path = format!("{}/{}/{}/layers", IMAGE_PATH, image, tag);
    match create_dir_all(path.clone()) {
        Ok(_) => {
            let resp = req.send().await?;
            let mut dest = {
                let output = format!("{}/{}", path, digest);
                File::create(output)?
            };
            let mut content = Cursor::new(resp.bytes().await?);
            copy(&mut content, &mut dest)?;
            Ok(())
        }
        Err(e) => {
            error!("Failed to create layer directory: {}", e);
            return Err(Box::new(e));
        }
    }
}

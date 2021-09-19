use std::fs::{create_dir_all, File};
use flate2::read::GzDecoder;
use tar::Archive;
use log::{error, info};
use crate::storage::image::load_manifest;
use crate::storage::{IMAGE_PATH, CONTAINER_PATH};

pub fn extract_image(image: &str, tag: &str, name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let manifest = match load_manifest(image, tag) {
        Ok(manifest) => manifest,
        Err(e) => return Err(e),
    };
    let path = format!("{}/{}", CONTAINER_PATH, name);
    match create_dir_all(path.clone()) {
        Ok(_) => {
            let layers = manifest.layers();
            for layer in layers.iter() {
                let digest = layer.digest();
                let layer_path = format!("{}/{}/{}/layers/{}", IMAGE_PATH, image, tag, digest);
                let layer_file = match File::open(layer_path) {
                    Ok(file) => file,
                    Err(e) => {
                        error!("Failed to open layer file: {}", e);
                        return Err(Box::new(e));
                    }
                };
                let gz_decoder = GzDecoder::new(layer_file);
                let mut tar_archive = Archive::new(gz_decoder);
                match tar_archive.unpack(path.clone()) {
                    Ok(_) => {
                        info!("Successfully extracted layer {}", digest);
                    }
                    Err(e) => {
                        error!("Failed to extract layer {}: {}", digest, e);
                        return Err(Box::new(e));
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to create container directory: {}", e);
            return Err(Box::new(e));
        },
    }
    Ok(path)
}
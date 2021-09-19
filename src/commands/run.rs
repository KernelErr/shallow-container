use crate::commands::pull::pull_image;
use crate::storage::container::extract_image;
use crate::storage::image::load_config;
use crate::storage::IMAGE_PATH;
use clap::ArgMatches;
use log::{error, info};
use nanoid::nanoid;
use std::path::Path;
use unshare;
use unshare::Namespace;

pub async fn run(arg: &ArgMatches) -> bool {
    let image = arg.value_of("image").unwrap();
    let (scope, image, tag) = match sscanf::scanf!(image, "{}/{}:{}", String, String, String) {
        Some((scope, image, tag)) => (scope, image, tag),
        None => {
            error!("Invalid image format: {}", image);
            return false;
        }
    };

    if !Path::new(&format!("{}/{}/{}", IMAGE_PATH, image, tag)).exists() {
        info!("Image not found, trying to pull it");
        if !pull_image(scope, image.clone(), tag.clone()).await {
            return false;
        };
    }

    let config = match load_config(&image, &tag) {
        Ok(config) => config,
        Err(_) => {
            return false;
        }
    };
    let container_name = nanoid!();
    let path = match extract_image(&image, &tag, &container_name) {
        Ok(path) => path,
        Err(_) => {
            return false;
        }
    };

    // commands & envs
    let commands = config.config.Cmd;
    if commands.is_empty() {
        error!("No commands found in config");
        return false;
    }
    let entry_point = config.config.Entrypoint;
    let mut cmd = match entry_point {
        Some(ep) => {
            let mut cmd = unshare::Command::new(ep[0].clone());
            for entry in ep.iter().skip(1) {
                cmd.arg(entry);
            }
            for c in &commands {
                cmd.arg(c);
            }
            cmd
        }
        None => {
            let mut cmd = unshare::Command::new(commands[0].clone());
            for c in commands.iter().skip(1) {
                cmd.arg(c);
            }
            cmd
        }
    };
    cmd.chroot_dir(path);
    for env in config.config.Env {
        let mut split = env.split('=');
        let key = split.next().unwrap();
        let value = split.next().unwrap();
        cmd.env(key, value);
    }

    // namespaces
    let mut namespaces = vec![
        Namespace::Pid,
        Namespace::Uts,
        Namespace::Ipc,
        Namespace::Mount,
        Namespace::User,
    ];
    if !arg.is_present("share_net") {
        namespaces.push(Namespace::Net);
    }
    cmd.unshare(&namespaces);

    cmd.close_fds(..);

    let mut child = match cmd.spawn() {
        Ok(child) => {
            info!("Started container {}", container_name);
            child
        }
        Err(e) => {
            error!("Failed to spawn process: {}", e);
            return false;
        }
    };

    child.wait().unwrap();
    true
}

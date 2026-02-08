//! Builds Docker socket hostPath volume and mount for agent runtime pods.
//!
//! Enables agents to run `docker build` by mounting the host Docker socket.

use k8s_openapi::api::core::v1::{HostPathVolumeSource, Volume, VolumeMount};

/// Default Docker socket path on the host.
const DEFAULT_DOCKER_SOCKET_PATH: &str = "/var/run/docker.sock";

/// Builds Docker socket hostPath volume and mount.
///
/// Returns empty vecs when `enabled` is `false`.
pub fn build(enabled: bool, socket_path: Option<&str>) -> (Vec<Volume>, Vec<VolumeMount>) {
    if !enabled {
        return (vec![], vec![]);
    }

    let host_path = socket_path.unwrap_or(DEFAULT_DOCKER_SOCKET_PATH);

    let volume = Volume {
        name: "docker-socket".to_string(),
        host_path: Some(HostPathVolumeSource {
            path: host_path.to_string(),
            type_: Some("Socket".to_string()),
        }),
        ..Default::default()
    };

    let volume_mount = VolumeMount {
        name: "docker-socket".to_string(),
        mount_path: DEFAULT_DOCKER_SOCKET_PATH.to_string(),
        ..Default::default()
    };

    (vec![volume], vec![volume_mount])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_volume_when_disabled() {
        let (volumes, mounts) = build(false, None);
        assert!(volumes.is_empty());
        assert!(mounts.is_empty());
    }

    #[test]
    fn volume_created_when_enabled() {
        let (volumes, mounts) = build(true, None);
        assert_eq!(volumes.len(), 1);
        assert_eq!(mounts.len(), 1);
    }

    #[test]
    fn volume_name_is_docker_socket() {
        let (volumes, mounts) = build(true, None);
        assert_eq!(volumes[0].name, "docker-socket");
        assert_eq!(mounts[0].name, "docker-socket");
    }

    #[test]
    fn default_host_path() {
        let (volumes, _) = build(true, None);
        let host_path = volumes[0].host_path.as_ref().unwrap();
        assert_eq!(host_path.path, "/var/run/docker.sock");
    }

    #[test]
    fn custom_host_path() {
        let (volumes, _) = build(true, Some("/custom/docker.sock"));
        let host_path = volumes[0].host_path.as_ref().unwrap();
        assert_eq!(host_path.path, "/custom/docker.sock");
    }

    #[test]
    fn host_path_type_is_socket() {
        let (volumes, _) = build(true, None);
        let host_path = volumes[0].host_path.as_ref().unwrap();
        assert_eq!(host_path.type_.as_deref(), Some("Socket"));
    }

    #[test]
    fn mount_path_is_default_socket() {
        let (_, mounts) = build(true, None);
        assert_eq!(mounts[0].mount_path, "/var/run/docker.sock");
    }
}

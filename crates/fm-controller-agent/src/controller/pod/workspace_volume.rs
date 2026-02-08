//! Builds workspace hostPath volume and mount for agent runtime pods.

use k8s_openapi::api::core::v1::{HostPathVolumeSource, Volume, VolumeMount};

/// Builds workspace hostPath volume and mount.
///
/// Host path: `<base_path>/<task_namespace>/` (DirectoryOrCreate).
/// Container mount: `<container_path>`.
///
/// Returns empty vecs when `base_path` is `None` (backwards compatible).
pub fn build(
    base_path: Option<&str>,
    container_path: &str,
    task_namespace: &str,
) -> (Vec<Volume>, Vec<VolumeMount>) {
    let Some(base_path) = base_path else {
        return (vec![], vec![]);
    };

    let host_path = format!("{}/{}", base_path, task_namespace);

    let volume = Volume {
        name: "workspace".to_string(),
        host_path: Some(HostPathVolumeSource {
            path: host_path,
            type_: Some("DirectoryOrCreate".to_string()),
        }),
        ..Default::default()
    };

    let volume_mount = VolumeMount {
        name: "workspace".to_string(),
        mount_path: container_path.to_string(),
        ..Default::default()
    };

    (vec![volume], vec![volume_mount])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_volume_when_base_path_is_none() {
        let (volumes, mounts) = build(None, "/workspace", "task-abc");
        assert!(volumes.is_empty());
        assert!(mounts.is_empty());
    }

    #[test]
    fn volume_created_when_base_path_set() {
        let (volumes, mounts) = build(Some("/host/tasks"), "/workspace", "task-abc");
        assert_eq!(volumes.len(), 1);
        assert_eq!(mounts.len(), 1);
    }

    #[test]
    fn volume_name_is_workspace() {
        let (volumes, mounts) = build(Some("/host/tasks"), "/workspace", "task-abc");
        assert_eq!(volumes[0].name, "workspace");
        assert_eq!(mounts[0].name, "workspace");
    }

    #[test]
    fn host_path_includes_task_namespace() {
        let (volumes, _) = build(Some("/host/tasks"), "/workspace", "task-abc");
        let host_path = volumes[0].host_path.as_ref().unwrap();
        assert_eq!(host_path.path, "/host/tasks/task-abc");
    }

    #[test]
    fn host_path_type_is_directory_or_create() {
        let (volumes, _) = build(Some("/host/tasks"), "/workspace", "task-abc");
        let host_path = volumes[0].host_path.as_ref().unwrap();
        assert_eq!(host_path.type_.as_deref(), Some("DirectoryOrCreate"));
    }

    #[test]
    fn mount_path_matches_container_path() {
        let (_, mounts) = build(Some("/host/tasks"), "/workspace", "task-abc");
        assert_eq!(mounts[0].mount_path, "/workspace");
    }

    #[test]
    fn custom_container_path() {
        let (_, mounts) = build(Some("/data"), "/opt/work", "task-xyz");
        assert_eq!(mounts[0].mount_path, "/opt/work");
    }
}

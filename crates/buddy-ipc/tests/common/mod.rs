use buddy_ipc::Endpoint;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);
pub struct TestEndpoint {
    pub endpoint: Endpoint,
}
impl TestEndpoint {
    pub fn new() -> Self {
        let id = format!(
            "beni-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            let dir = std::env::temp_dir().join(id);
            std::fs::DirBuilder::new().mode(0o700).create(&dir).unwrap();
            Self {
                endpoint: Endpoint(dir.join("buddy.sock").to_string_lossy().into_owned()),
            }
        }
        #[cfg(windows)]
        {
            Self {
                endpoint: Endpoint(format!("{}.{}", Endpoint::for_user().unwrap().0, id)),
            }
        }
    }
}
impl Drop for TestEndpoint {
    fn drop(&mut self) {
        #[cfg(unix)]
        {
            let _ =
                std::fs::remove_dir_all(std::path::Path::new(&self.endpoint.0).parent().unwrap());
        }
    }
}

use interprocess::local_socket::Name;
#[cfg(unix)]
use interprocess::local_socket::{GenericFilePath, ToFsName};
#[cfg(windows)]
use interprocess::local_socket::{GenericNamespaced, ToNsName};
use std::io;
#[derive(Debug, Clone)]
pub struct Endpoint(pub String);
impl Endpoint {
    pub fn for_user() -> io::Result<Self> {
        #[cfg(unix)]
        {
            let dirs = directories::BaseDirs::new()
                .ok_or_else(|| io::Error::other("Benutzerverzeichnis nicht verfügbar"))?;
            #[cfg(target_os = "macos")]
            let directory = dirs.data_local_dir().join("io.witzelfitz.beni-buddy");
            #[cfg(not(target_os = "macos"))]
            let directory = dirs
                .runtime_dir()
                .unwrap_or(dirs.data_local_dir())
                .join("beni-buddy");
            Ok(Self(
                directory
                    .join("ipc/buddy.sock")
                    .to_string_lossy()
                    .into_owned(),
            ))
        }
        #[cfg(windows)]
        {
            Ok(Self(format!("io.witzelfitz.beni-buddy.{}", user_sid()?)))
        }
    }
    pub(crate) fn name(&self) -> io::Result<Name<'_>> {
        #[cfg(unix)]
        {
            self.0.as_str().to_fs_name::<GenericFilePath>()
        }
        #[cfg(windows)]
        {
            self.0.as_str().to_ns_name::<GenericNamespaced>()
        }
    }
}
#[cfg(windows)]
pub(crate) fn user_sid() -> io::Result<String> {
    use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
    use windows_sys::Win32::{
        Foundation::LocalFree,
        Security::{
            Authorization::ConvertSidToStringSidW, GetTokenInformation, TOKEN_QUERY, TOKEN_USER,
            TokenUser,
        },
        System::Threading::{GetCurrentProcess, OpenProcessToken},
    };
    // SAFETY: each output pointer refers to owned, aligned storage. The token and allocated
    // SID string remain alive throughout their use and are released on every return path.
    unsafe {
        let mut token = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return Err(io::Error::last_os_error());
        }
        let token = OwnedHandle::from_raw_handle(token);
        let mut needed = 0;
        GetTokenInformation(
            token.as_raw_handle(),
            TokenUser,
            std::ptr::null_mut(),
            0,
            &mut needed,
        );
        if needed == 0 {
            return Err(io::Error::last_os_error());
        }
        let mut buffer = vec![0usize; (needed as usize).div_ceil(std::mem::size_of::<usize>())];
        if GetTokenInformation(
            token.as_raw_handle(),
            TokenUser,
            buffer.as_mut_ptr().cast(),
            needed,
            &mut needed,
        ) == 0
        {
            return Err(io::Error::last_os_error());
        }
        let user = &*buffer.as_ptr().cast::<TOKEN_USER>();
        let mut sid = std::ptr::null_mut();
        if ConvertSidToStringSidW(user.User.Sid, &mut sid) == 0 {
            return Err(io::Error::last_os_error());
        }
        let mut len = 0;
        while *sid.add(len) != 0 {
            len += 1;
        }
        let value =
            String::from_utf16(std::slice::from_raw_parts(sid, len)).map_err(io::Error::other);
        LocalFree(sid.cast());
        value
    }
}
#[cfg(windows)]
pub(crate) fn listener_options(
    endpoint: &Endpoint,
) -> io::Result<interprocess::local_socket::ListenerOptions<'_>> {
    use interprocess::os::windows::{
        local_socket::ListenerOptionsExt, security_descriptor::SecurityDescriptor,
    };
    let sid = user_sid()?;
    // A protected DACL grants access only to this Windows user; deny network logons.
    let sddl = widestring::U16CString::from_str(format!("O:{sid}D:P(D;;GA;;;NU)(A;;GA;;;{sid})"))
        .map_err(io::Error::other)?;
    let sd = SecurityDescriptor::deserialize(&sddl)?;
    Ok(interprocess::local_socket::ListenerOptions::new()
        .name(endpoint.name()?)
        .security_descriptor(sd))
}
#[cfg(unix)]
pub(crate) fn prepare_directory(endpoint: &Endpoint) -> io::Result<std::fs::File> {
    use std::{
        fs::{self, DirBuilder, OpenOptions},
        os::unix::fs::{DirBuilderExt, FileTypeExt, MetadataExt, OpenOptionsExt},
    };
    let path = std::path::Path::new(&endpoint.0);
    if !path.is_absolute() {
        return Err(io::Error::other("Socket-Pfad muss absolut sein"));
    }
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("Socket-Verzeichnis fehlt"))?;
    let uid = nix::unistd::geteuid().as_raw();
    if !parent.exists() {
        DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(parent)?;
    }
    let metadata = fs::symlink_metadata(parent)?;
    if !metadata.is_dir() || metadata.uid() != uid || metadata.mode() & 0o777 != 0o700 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Socket-Verzeichnis braucht eigenen Benutzer und Rechte 0700",
        ));
    }
    let lock_path = parent.join("buddy.lock");
    if let Ok(m) = fs::symlink_metadata(&lock_path)
        && (!m.is_file() || m.uid() != uid || m.mode() & 0o777 != 0o600)
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Unsichere Socket-Sperrdatei",
        ));
    }
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .mode(0o600)
        .open(lock_path)?;
    lock.try_lock()
        .map_err(|e| io::Error::new(io::ErrorKind::AddrInUse, e))?;
    if let Ok(m) = fs::symlink_metadata(path) {
        if !m.file_type().is_socket() || m.uid() != uid || m.mode() & 0o777 != 0o600 {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Unsichere oder fremde Socket-Datei",
            ));
        }
        match std::os::unix::net::UnixStream::connect(path) {
            Ok(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::AddrInUse,
                    "Beni empfängt bereits Meldungen",
                ));
            }
            Err(e) if e.kind() == io::ErrorKind::ConnectionRefused => fs::remove_file(path)?,
            Err(e) => return Err(e),
        }
    }
    Ok(lock)
}
#[cfg(unix)]
pub(crate) fn check_client_endpoint(endpoint: &Endpoint) -> io::Result<()> {
    use std::os::unix::fs::{FileTypeExt, MetadataExt};
    let path = std::path::Path::new(&endpoint.0);
    let parent = std::fs::symlink_metadata(
        path.parent()
            .ok_or_else(|| io::Error::other("Socket-Verzeichnis fehlt"))?,
    )?;
    let file = std::fs::symlink_metadata(path)?;
    let uid = nix::unistd::geteuid().as_raw();
    if !parent.is_dir()
        || parent.uid() != uid
        || parent.mode() & 0o777 != 0o700
        || !file.file_type().is_socket()
        || file.uid() != uid
        || file.mode() & 0o777 != 0o600
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Unsicherer Socket-Endpunkt",
        ));
    }
    Ok(())
}

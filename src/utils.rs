use std::io::prelude::*;
use std::fs::{File, copy, create_dir, metadata};
use std::path::Path;
use std::os::unix::io::AsRawFd;
use std::io::Error as IOError;
use std::time::{SystemTime, Duration, UNIX_EPOCH};

use errors::{Result, ResultExt};

pub fn create_file<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    let mut file = File::create(&path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

/// Very similar to `create_dir` from the std except it checks if the folder
/// exists before creating it
pub fn create_directory<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    if !path.exists() {
        create_dir(path)
            .chain_err(|| format!("Was not able to create folder {}", path.display()))?;
    }
    Ok(())
}


/// Return the content of a file, with error handling added
pub fn read_file<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = path.as_ref();

    let mut content = String::new();
    File::open(path)
        .chain_err(|| format!("Failed to open '{:?}'", path.display()))?
        .read_to_string(&mut content)?;

    Ok(content)
}


/// Takes a full path to a .md and returns only the components after the `content` directory
/// Will not return the filename as last component
pub fn find_content_components<P: AsRef<Path>>(path: P) -> Vec<String> {
    let path = path.as_ref();
    let mut is_in_content = false;
    let mut components = vec![];

    for section in path.parent().unwrap().components() {
        let component = section.as_ref().to_string_lossy();

        if is_in_content {
            components.push(component.to_string());
            continue;
        }

        if component == "content" {
            is_in_content = true;
        }
    }

    components
}

/// Set modification time and access time of file
fn set_file_times<P: AsRef<Path>>(path: P, accessed: &SystemTime, modified: &SystemTime) -> Result<()> {
    let accessed_since_epoch = accessed.duration_since(UNIX_EPOCH)?;
    let modified_since_epoch = modified.duration_since(UNIX_EPOCH)?;
    futime(path, &accessed_since_epoch, &modified_since_epoch)
}


/// Copy file if size or modification time is different
pub fn copy_file_if_modified<P: AsRef<Path>>(source: P, target: P) -> Result<()> {
    let target = target.as_ref();
    let source = source.as_ref();
    let source_metadata = metadata(source)?;

    if target.exists() {
        let target_metadata = metadata(target)?;

        if target_metadata.len() == source_metadata.len() &&
            target_metadata.modified()? == source_metadata.modified()? {
            return Ok(())
        }
    }

    copy(&source, &target)?;
    let accessed_time = source_metadata.accessed()?;
    let modified_time = source_metadata.modified()?;
    set_file_times(target, &accessed_time, &modified_time)
}


#[cfg(any(target_os="unix", target_os="linux"))]
fn futime<P: AsRef<Path>>(path: P, accessed_since_epoch: &Duration, modified_since_epoch: &Duration) -> Result<()> {
    use libc::{futimens, timespec, time_t, c_long};
    let file = File::open(path)?;

    let atime = timespec {
        tv_sec: accessed_since_epoch.as_secs() as time_t,
        tv_nsec: accessed_since_epoch.subsec_nanos() as c_long
    };
    let mtime = timespec {
        tv_sec: modified_since_epoch.as_secs() as time_t,
        tv_nsec: modified_since_epoch.subsec_nanos() as c_long
    };
    let times = [atime, mtime];
    let ret = unsafe { futimens(file.as_raw_fd(), times.as_ptr()) };

    if ret == 0 {
        Ok(())
    } else {
        bail!(IOError::last_os_error())
    }
}

#[cfg(target_os="macos")]
fn futime<P: AsRef<Path>>(path: P, accessed_since_epoch: &Duration, modified_since_epoch: &Duration) -> Result<()> {
    use libc::{futimes, timeval, time_t, suseconds_t};
    let file = File::open(path)?;

    let atime = timeval {
        tv_sec: accessed_since_epoch.as_secs() as time_t,
        tv_usec: accessed_since_epoch.subsec_nanos() as suseconds_t
    };
    let mtime = timeval {
        tv_sec: modified_since_epoch.as_secs() as time_t,
        tv_usec: modified_since_epoch.subsec_nanos() as suseconds_t
    };
    let times = [atime, mtime];

    let times = [atime, mtime];
    let ret = unsafe { futimes(file.as_raw_fd(), times.as_ptr()) };

    if ret == 0 {
        Ok(())
    } else {
        bail!(IOError::last_os_error())
    }
}


#[cfg(test)]
mod tests {
    use super::{find_content_components};

    #[test]
    fn test_find_content_components() {
        let res = find_content_components("/home/vincent/code/site/content/posts/tutorials/python.md");
        assert_eq!(res, ["posts".to_string(), "tutorials".to_string()]);
    }
}

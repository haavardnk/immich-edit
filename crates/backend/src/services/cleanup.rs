use std::io;
use std::path::Path;

pub fn warn_unless_missing(result: io::Result<()>, path: &Path) {
    if let Err(error) = result
        && error.kind() != io::ErrorKind::NotFound
    {
        tracing::warn!(path = %path.display(), %error, "cache cleanup failed");
    }
}

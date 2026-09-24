use std::io;
use std::path::{Path, PathBuf};

pub fn join(dir: &Path, components: &[&str]) -> io::Result<PathBuf> {
    let mut path = dir.to_path_buf();
    for component in components {
        if component.contains("..") {
            return Err(unsafe_component(component));
        }
        if component.is_empty() || component.contains(['/', '\\', '\0']) {
            return Err(unsafe_component(component));
        }
        path.push(component);
    }
    Ok(path)
}

fn unsafe_component(component: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("unsafe path component {component:?}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_plain_components() {
        let path = join(Path::new("/data"), &["1", "owner", "abc.r8"]).unwrap();
        if path != Path::new("/data/1/owner/abc.r8") {
            panic!("joined {path:?}");
        }
    }

    #[test]
    fn rejects_components_that_leave_the_directory() {
        for component in ["..", "../x", "a/b", "a\\b", "", "a\0b", "x..y"] {
            let Err(error) = join(Path::new("/data"), &["1", component]) else {
                panic!("{component:?} must be rejected");
            };
            if error.kind() != io::ErrorKind::InvalidInput {
                panic!("{component:?} rejected as {error}");
            }
        }
    }
}

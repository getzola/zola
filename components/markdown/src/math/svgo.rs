use std::{io::Write, process::Command};

pub struct Svgo {
    bin_path: String,
}

impl Default for Svgo {
    fn default() -> Self {
        Self { bin_path: "svgo".to_string() }
    }
}

impl Svgo {
    pub fn new<S: Into<String>>(bin_path: S) -> Self {
        Self { bin_path: bin_path.into() }
    }

    pub fn check_bin(&self) -> Result<(), String> {
        let output =
            Command::new(&self.bin_path).arg("--version").output().map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        Ok(())
    }

    pub fn minify(&self, svg: &str, config: Option<&str>) -> Result<String, String> {
        let mut cmd = Command::new(&self.bin_path);
        let mut child = cmd.arg("-i").arg("-").arg("-o").arg("-").arg("--multipass");

        if let Some(config) = config {
            child = child.arg("--config").arg(config);
        }

        let mut child = child
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;
        child.stdin.as_mut().unwrap().write_all(svg.as_bytes()).unwrap();
        let output = child.wait_with_output().map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

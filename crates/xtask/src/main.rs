use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

type TaskResult<T> = Result<T, Box<dyn std::error::Error>>;

fn main() -> TaskResult<()> {
  let command = env::args().nth(1).unwrap_or_default();
  let root = workspace_root()?;

  match command.as_str() {
    "sync-wit" => sync_wit(&root),
    "sdk-rust" => {
      sync_wit(&root)?;
      run_cargo(&root, &["build", "-p", "bud-plugin-sdk"])
    }
    "sum-plugin" => {
      sync_wit(&root)?;
      build_sum_plugin(&root)?;
      copy_sum_plugin_wasm(&root)
    }
    "plugin-dev" => {
      sync_wit(&root)?;
      run_cargo(&root, &["build", "-p", "bud-plugin-sdk"])?;
      build_sum_plugin(&root)?;
      copy_sum_plugin_wasm(&root)
    }
    _ => Err(
      format!(
        "Unknown command '{}'. Available: sync-wit | sdk-rust | sum-plugin | plugin-dev",
        command
      )
      .into(),
    ),
  }
}

fn workspace_root() -> TaskResult<PathBuf> {
  let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  manifest_dir
    .parent()
    .and_then(Path::parent)
    .map(Path::to_path_buf)
    .ok_or_else(|| "Failed to resolve workspace root from crates/xtask".into())
}

fn sync_wit(root: &Path) -> TaskResult<()> {
  let src = root.join("wit/bud.wit");
  let dst = root.join("sdk/rust/wit/bud.wit");

  if !src.is_file() {
    return Err(format!("WIT source does not exist: {}", src.display()).into());
  }

  fs::create_dir_all(
    dst
      .parent()
      .ok_or_else(|| format!("Invalid target path: {}", dst.display()))?,
  )?;

  fs::copy(&src, &dst)?;

  println!("Synced WIT: {} -> {}", src.display(), dst.display());
  Ok(())
}

fn copy_sum_plugin_wasm(root: &Path) -> TaskResult<()> {
  let src = root.join("target/wasm32-wasip2/release/sum_plugin.wasm");
  let dst = root.join("example/sum-plugin/main.wasm");

  if !src.is_file() {
    return Err(format!("Built wasm artifact does not exist: {}", src.display()).into());
  }

  fs::copy(&src, &dst)?;
  println!("Synced wasm: {} -> {}", src.display(), dst.display());
  Ok(())
}

fn build_sum_plugin(root: &Path) -> TaskResult<()> {
  run_cargo(
    root,
    &[
      "build",
      "--manifest-path",
      "example/sum-plugin/Cargo.toml",
      "--target",
      "wasm32-wasip2",
      "--target-dir",
      "target",
      "--release",
    ],
  )
}

fn run_cargo(root: &Path, args: &[&str]) -> TaskResult<()> {
  let status = Command::new("cargo")
    .current_dir(root)
    .args(args)
    .status()?;
  if status.success() {
    Ok(())
  } else {
    Err(format!("cargo command failed: cargo {}", args.join(" ")).into())
  }
}

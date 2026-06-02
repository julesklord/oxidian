// OXIDIAN BEGIN
use anyhow::{Context as _, Result, bail};
use async_trait::async_trait;
use gpui::AsyncApp;
use http_client::github::{GitHubLspBinaryVersion, latest_github_release};
use http_client::github_download::GithubBinaryMetadata;
pub use language::*;
use lsp::{LanguageServerBinary, LanguageServerName, Uri};
use std::{env::consts, future::Future, path::PathBuf, sync::Arc};
use util::{ResultExt, fs::remove_matching, maybe};

pub struct MarksmanLspAdapter;

impl MarksmanLspAdapter {
    const SERVER_NAME: LanguageServerName = LanguageServerName::new_static("marksman");
}

impl LspInstaller for MarksmanLspAdapter {
    type BinaryVersion = GitHubLspBinaryVersion;

    async fn fetch_latest_server_version(
        &self,
        delegate: &Arc<dyn LspAdapterDelegate>,
        pre_release: bool,
        _: &mut AsyncApp,
    ) -> Result<GitHubLspBinaryVersion> {
        let release = latest_github_release(
            "artempyanykh/marksman",
            true,
            pre_release,
            delegate.http_client(),
        )
        .await?;

        let asset_name = match (consts::OS, consts::ARCH) {
            ("macos", _) => "marksman-macos".to_owned(),
            ("linux", "x86_64") => "marksman-linux-x64".to_owned(),
            ("linux", "aarch64") => "marksman-linux-arm64".to_owned(),
            ("windows", _) => "marksman.exe".to_owned(),
            (os, arch) => bail!("Running on unsupported OS/arch: {os}/{arch}"),
        };

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .with_context(|| format!("no asset found matching {asset_name:?}"))?;

        let version = GitHubLspBinaryVersion {
            name: release.tag_name,
            url: asset.browser_download_url.clone(),
            digest: asset.digest.clone(),
        };
        Ok(version)
    }

    async fn check_if_user_installed(
        &self,
        delegate: &Arc<dyn LspAdapterDelegate>,
        _: Option<Toolchain>,
        cx: &AsyncApp,
    ) -> Option<LanguageServerBinary> {
        // 1. Check if configured in active vault
        let marksman_from_vault = cx.update(|cx| {
            cx.try_global::<oxidian_core::MarksmanBinaryPath>()
                .and_then(|p| p.0.clone())
        });

        if let Some(path) = marksman_from_vault {
            if path.exists() {
                return Some(LanguageServerBinary {
                    path,
                    arguments: Vec::new(),
                    env: None,
                });
            }
        }

        // 2. Check if installed on system PATH
        let path = delegate.which(Self::SERVER_NAME.as_ref()).await?;
        Some(LanguageServerBinary {
            path,
            arguments: Vec::new(),
            env: None,
        })
    }

    fn fetch_server_binary(
        &self,
        version: GitHubLspBinaryVersion,
        container_dir: PathBuf,
        delegate: &Arc<dyn LspAdapterDelegate>,
    ) -> impl Send + Future<Output = Result<LanguageServerBinary>> + use<> {
        let delegate = delegate.clone();

        async move {
            let GitHubLspBinaryVersion {
                name,
                url,
                digest: expected_digest,
            } = version;
            let version_dir = container_dir.join(format!("marksman_{name}"));
            let exe_name = format!("marksman{}", consts::EXE_SUFFIX);
            let binary_path = version_dir.join(&exe_name);

            let binary = LanguageServerBinary {
                path: binary_path.clone(),
                env: None,
                arguments: Default::default(),
            };

            let metadata_path = version_dir.join("metadata");
            let metadata = GithubBinaryMetadata::read_from_file(&metadata_path)
                .await
                .ok();
            if let Some(metadata) = metadata {
                let validity_check = async || {
                    delegate
                        .try_exec(LanguageServerBinary {
                            path: binary_path.clone(),
                            arguments: vec!["--version".into()],
                            env: None,
                        })
                        .await
                        .inspect_err(|err| {
                            log::warn!(
                                "Unable to run {binary_path:?} asset, redownloading: {err:#}",
                            )
                        })
                };
                if let (Some(actual_digest), Some(expected_digest)) =
                    (&metadata.digest, &expected_digest)
                {
                    if actual_digest == expected_digest {
                        if validity_check().await.is_ok() {
                            return Ok(binary);
                        }
                    } else {
                        log::info!(
                            "SHA-256 mismatch for {binary_path:?} asset, downloading new asset. Expected: {expected_digest}, Got: {actual_digest}"
                        );
                    }
                } else if validity_check().await.is_ok() {
                    return Ok(binary);
                }
            }

            // We download the raw binary directly
            let temp_download_path = container_dir.join(format!("temp_{exe_name}"));

            let mut response = delegate
                .http_client()
                .get(&url, Default::default(), true)
                .await
                .context("downloading marksman release")?;

            let mut body = Vec::new();
            use futures::AsyncReadExt as _;
            response.body_mut().read_to_end(&mut body).await?;

            if let Some(expected_sha_256) = expected_digest.as_deref() {
                use sha2::{Digest as _, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(&body);
                let asset_sha_256 = format!("{:x}", hasher.finalize());
                anyhow::ensure!(
                    asset_sha_256 == expected_sha_256,
                    "marksman asset got SHA-256 mismatch. Expected: {expected_sha_256}, Got: {asset_sha_256}"
                );
            }

            std::fs::write(&temp_download_path, &body)?;

            std::fs::create_dir_all(&version_dir)?;
            std::fs::rename(&temp_download_path, &binary_path)?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&binary_path)?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&binary_path, perms)?;
            }

            remove_matching(&container_dir, |entry| entry != version_dir).await;
            GithubBinaryMetadata::write_to_file(
                &GithubBinaryMetadata {
                    metadata_version: 1,
                    digest: expected_digest,
                },
                &metadata_path,
            )
            .await?;

            Ok(binary)
        }
    }
    // OXIDIAN END

    async fn cached_server_binary(
        &self,
        container_dir: PathBuf,
        _: &dyn LspAdapterDelegate,
    ) -> Option<LanguageServerBinary> {
        maybe!(async {
            let entries = std::fs::read_dir(&container_dir)?;
            for entry in entries {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    let dir_path = entry.path();
                    let exe_name = format!("marksman{}", consts::EXE_SUFFIX);
                    let binary_path = dir_path.join(&exe_name);
                    if binary_path.exists() {
                        return Ok(LanguageServerBinary {
                            path: binary_path,
                            arguments: Default::default(),
                            env: None,
                        });
                    }
                }
            }
            anyhow::bail!("no cached marksman binary found")
        })
        .await
        .log_err()
    }
}

#[async_trait(?Send)]
impl LspAdapter for MarksmanLspAdapter {
    fn name(&self) -> LanguageServerName {
        Self::SERVER_NAME
    }

    async fn workspace_configuration(
        self: Arc<Self>,
        _: &Arc<dyn LspAdapterDelegate>,
        _: Option<Toolchain>,
        _: Option<Uri>,
        _: &mut AsyncApp,
    ) -> Result<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

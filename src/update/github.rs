use super::*;
use crate::{core::Core, errors};
use futures::StreamExt;
use serde::Deserialize;
use std::env::consts::{ARCH, OS};

pub struct GitHubSource {
    pub repo: String,
    pub artifact_prefix: String,
    pub release_tag_prefix: String,
}

impl Default for GitHubSource {
    fn default() -> Self {
        Self {
            repo: "SierraSoftworks/git-tool".to_string(),
            artifact_prefix: "git-tool-".to_string(),
            release_tag_prefix: "v".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl Source for GitHubSource {
    async fn get_releases(&self, core: &Core) -> Result<Vec<Release>, crate::core::Error> {
        let uri = format!("https://api.github.com/repos/{}/releases", self.repo);
        info!("Making GET request to {} to check for new releases.", uri);

        let resp = self.get(core, &uri).await?;
        debug!(
            "Received HTTP {} {} from GitHub when requesting releases.",
            resp.status().as_u16(),
            resp.status().canonical_reason().unwrap_or("UNKNOWN")
        );

        match resp.status() {
            reqwest::StatusCode::OK => {
                let releases: Vec<GitHubRelease> = resp.json().await?;

                self.get_releases_from_response(releases)
            }
            reqwest::StatusCode::TOO_MANY_REQUESTS | reqwest::StatusCode::FORBIDDEN => {
                Err(errors::user(
                    "GitHub has rate limited requests from your IP address.",
                    "Please wait until GitHub removes this rate limit before trying again.",
                ))
            }
            status => {
                let inner_error = errors::reqwest::ResponseError::with_body(resp).await;
                Err(errors::system_with_internal(
                    &format!("Received an HTTP {} response from GitHub when attempting to list items in the Git-Tool registry.", status),
                    "Please read the error message below and decide if there is something you can do to fix the problem, or report it to us on GitHub.",
                    inner_error))
            }
        }
    }

    async fn get_binary<W: std::io::Write + Send>(
        &self,
        core: &Core,
        release: &Release,
        variant: &ReleaseVariant,
        into: &mut W,
    ) -> Result<(), crate::core::Error> {
        let uri = format!(
            "https://github.com/{}/releases/download/{}/{}",
            self.repo, release.id, variant.id
        );

        self.download_to_file(core, &uri, into).await
    }
}

impl GitHubSource {
    async fn get(&self, core: &Core, url: &str) -> Result<reqwest::Response, errors::Error> {
        let uri: reqwest::Url = url.parse().map_err(|e| {
            errors::system_with_internal(
                &format!("Unable to parse GitHub API URL '{}'.", url),
                "Please report this error to us by opening a ticket in GitHub.",
                e,
            )
        })?;

        // NOTE: This allows us to consume the GITHUB_TOKEN environment variable in the test
        // environment to bypass rate limiting restrictions.
        // TODO: We should probably support using the users github.com token here to avoid rate limiting
        #[allow(unused_mut)]
        let mut req = reqwest::Request::new(reqwest::Method::GET, uri);

        req.headers_mut().append(
            "User-Agent",
            version!("Git-Tool/").parse().map_err(|e| {
                errors::system_with_internal(
                    &format!(
                        "Unable to parse Git-Tool user agent header {}.",
                        version!("Git-Tool/")
                    ),
                    "Please report this error to us by opening a ticket in GitHub.",
                    e,
                )
            })?,
        );

        #[cfg(test)]
        {
            if let Ok(token) = std::env::var("GITHUB_TOKEN") {
                req.headers_mut().append(
                    "Authorization",
                    format!("token {}", token).parse().map_err(|e| {
                        errors::system_with_internal(
                            "Unable to parse GITHUB_TOKEN authorization header.",
                            "Please report this error to us by opening a ticket in GitHub.",
                            e,
                        )
                    })?,
                );
            }
        }

        core.http_client().request(req).await
    }

    fn get_releases_from_response(
        &self,
        releases: Vec<GitHubRelease>,
    ) -> Result<Vec<Release>, errors::Error> {
        let mut output: Vec<Release> = Vec::new();
        output.reserve(releases.len());

        for r in releases {
            if !r.tag_name.starts_with(&self.release_tag_prefix) {
                continue;
            }

            match r.tag_name[self.release_tag_prefix.len()..].parse() {
                Ok(version) => output.push(Release {
                    id: r.tag_name.clone(),
                    changelog: r.body.clone(),
                    version,
                    variants: self.get_variants_from_response(&r),
                }),
                Err(_) => {}
            }
        }

        Ok(output)
    }

    fn get_variants_from_response(&self, release: &GitHubRelease) -> Vec<ReleaseVariant> {
        let mut variants = Vec::new();

        for a in release.assets.iter() {
            if !a.name.starts_with(&self.artifact_prefix) {
                continue;
            }

            let spec_name = a.name[self.artifact_prefix.len()..]
                .trim_end_matches(".exe")
                .to_string();
            let mut parts = spec_name.split('-');

            let arch = match parts.next_back() {
                Some(spec_arch) => spec_arch.to_string(),
                None => ARCH.to_string(),
            };

            let platform = match parts.next_back() {
                Some(os) => os.to_string(),
                None => OS.to_string(),
            };

            variants.push(ReleaseVariant {
                id: a.name.clone(),
                arch,
                platform,
            })
        }

        variants
    }

    async fn download_to_file<W: std::io::Write + Send>(
        &self,
        core: &Core,
        uri: &str,
        into: &mut W,
    ) -> Result<(), errors::Error> {
        let resp = self.get(core, uri).await?;

        match resp.status() {
            http::StatusCode::OK => {
                let mut stream = resp.bytes_stream();

                while let Some(buf) = stream.next().await {
                    let buf = buf?;
                    into.write_all(&buf).map_err(|err| {
                        errors::user_with_internal(
                            &format!(
                                "Could not write data from '{}' to disk due to an OS-level error.",
                                uri
                            ),
                            "Check that Git-Tool has permission to create and write to this file and that the parent directory exists.",
                            err,
                        )
                    })?;
                }

                return Ok(())
            },
            reqwest::StatusCode::TOO_MANY_REQUESTS | reqwest::StatusCode::FORBIDDEN => {
                return Err(errors::user(
                    "GitHub has rate limited requests from your IP address.",
                    "Please wait until GitHub removes this rate limit before trying again."))
            },
            status => {
                return Err(errors::system(
                    &format!("Received an HTTP {} response from GitHub when attempting to download the update for your platform ({}).", status, uri),
                    "Please read the error message below and decide if there is something you can do to fix the problem, or report it to us on GitHub."))
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubRelease {
    pub name: String,
    pub tag_name: String,
    pub body: String,
    pub prerelease: bool,
    pub assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    pub name: String,
}

#[cfg(test)]
pub mod mocks {
    use crate::core::HttpClient;

    pub fn mock_get_releases() {
        HttpClient::mock(vec![
            HttpClient::route(
                "GET",
                "https://api.github.com/repos/SierraSoftworks/git-tool/releases",
                200,
                r#"[
                            {
                                "name": "Version 2.0.0",
                                "tag_name":"v2.0.0",
                                "body": "Example Release",
                                "prerelease": false,
                                "assets": [
                                    { "name": "git-tool-windows-amd64.exe" },
                                    { "name": "git-tool-linux-amd64" },
                                    { "name": "git-tool-linux-arm64" },
                                    { "name": "git-tool-darwin-amd64" },
                                    { "name": "git-tool-darwin-arm64" }
                                ]
                            }
                        ]"#,
            ),
            HttpClient::route(
                "GET",
                "https://github.com/SierraSoftworks/git-tool/releases/download/v2.0.0/git-tool-windows-amd64.exe",
                200,
                r#"testdata"#,
            ),
            HttpClient::route(
                "GET",
                "https://github.com/SierraSoftworks/git-tool/releases/download/v2.0.0/git-tool-linux-amd64",
                200,
                r#"testdata"#,
            ),
            HttpClient::route(
                "GET",
                "https://github.com/SierraSoftworks/git-tool/releases/download/v2.0.0/git-tool-linux-arm64",
                200,
                r#"testdata"#,
            ),
            HttpClient::route(
                "GET",
                "https://github.com/SierraSoftworks/git-tool/releases/download/v2.0.0/git-tool-darwin-amd64",
                200,
                r#"testdata"#,
            ),
            HttpClient::route(
                "GET",
                "https://github.com/SierraSoftworks/git-tool/releases/download/v2.0.0/git-tool-darwin-arm64",
                200,
                r#"testdata"#,
            ),
        ]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::Write,
        sync::{Arc, Mutex},
    };

    #[tokio::test]
    async fn test_get_releases() {
        let source = GitHubSource::default();
        mocks::mock_get_releases();

        let core = Core::builder().build();

        let releases = source.get_releases(&core).await.unwrap();

        assert_eq!(releases.len(), 1);
        for release in releases {
            assert!(
                release.id.contains(&release.version.to_string()),
                "the release version should be derived from the tag"
            );
            assert_ne!(
                &release.changelog, "",
                "the release changelog should not be empty"
            );
        }
    }

    #[tokio::test]
    async fn test_download() {
        let source = GitHubSource::default();
        mocks::mock_get_releases();

        let core = Core::builder().build();

        let releases = source.get_releases(&core).await.unwrap();
        let latest =
            Release::get_latest(releases.iter()).expect("There should be an available release");
        let variant = latest
            .variants
            .first()
            .expect("There should be a variant available");

        let mut target = sink();

        source
            .get_binary(&core, &latest, &variant, &mut target)
            .await
            .unwrap();

        assert!(target.get_length() > 0);
    }

    fn sink() -> Sink {
        Sink {
            length: Arc::new(Mutex::new(0)),
        }
    }

    struct Sink {
        length: Arc<Mutex<usize>>,
    }

    impl Sink {
        pub fn get_length(&self) -> usize {
            self.length.lock().map(|m| *m).unwrap_or_default()
        }
    }

    impl Write for Sink {
        #[inline]
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.length
                .lock()
                .map(|mut m| {
                    *m += buf.len();
                    buf.len()
                })
                .map_err(|_| std::io::ErrorKind::Other.into())
        }

        #[inline]
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
}

use super::*;
use serde::Deserialize;
use hyper::Client;
use crate::errors;
use http::Uri;
use std::env::consts::{OS, ARCH};
use hyper::body::HttpBody;

pub struct GitHubSource {
    pub repo: String,
    pub artifact_prefix: String,
    pub release_tag_prefix: String,
    client: Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>, hyper::Body>
}

impl Default for GitHubSource {
    fn default() -> Self {
        let https = hyper_tls::HttpsConnector::new();
        let client = Client::builder()
            .build(https);

        Self {
            repo: "SierraSoftworks/git-tool".to_string(),
            artifact_prefix: "git-tool-".to_string(),
            release_tag_prefix: "v".to_string(),
            client
        }
    }
}

#[async_trait::async_trait]
impl Source for GitHubSource {
    async fn get_releases(&self) -> Result<Vec<Release>, crate::core::Error> {
        let uri: Uri = format!("https://api.github.com/repos/{}/releases", self.repo).parse()?;

        let req = hyper::Request::get(uri)
            .header("User-Agent", "Git-Tool/".to_string() + env!("CARGO_PKG_VERSION"))
            .body(hyper::Body::empty())
            .map_err(|e| errors::system_with_internal(
                "Unable to construct web request for Git-Tool releases.",
                "Please report this error to us by opening a ticket in GitHub.",
                e))?;

        let resp = self.client.request(req).await?;

        match resp.status() {
            http::StatusCode::OK => {
                let body = hyper::body::to_bytes(resp.into_body()).await?;
                let releases: Vec<GitHubRelease> = serde_json::from_slice(&body)?;

                self.get_releases_from_response(releases)
            },
            http::StatusCode::TOO_MANY_REQUESTS => {
                Err(errors::user(
                    "GitHub has rate limited requests from your IP address.",
                    "Please wait until GitHub removes this rate limit before trying again."))
            },
            status => {
                let inner_error = errors::hyper::HyperResponseError::with_body(resp).await;
                Err(errors::system_with_internal(
                    &format!("Received an HTTP {} response from GitHub when attempting to list items in the Git-Tool registry.", status),
                    "Please read the error message below and decide if there is something you can do to fix the problem, or report it to us on GitHub.",
                    inner_error))
            }
        }
    }

    async fn get_binary<W: std::io::Write + Send>(&self, release: &Release, variant: &ReleaseVariant, into: &mut W) -> Result<(), crate::core::Error> {
        let uri: Uri = format!("https://github.com/{}/releases/download/{}/{}", self.repo, release.id, variant.id).parse()?;

        self.download_to_file(uri, into).await
    }
}

impl GitHubSource {
    fn get_releases_from_response(&self, releases: Vec<GitHubRelease>) -> Result<Vec<Release>, errors::Error> {
        let mut output: Vec<Release> = Vec::new();
        output.reserve(releases.len());

        for r in releases {
            if !r.tag_name.starts_with(&self.release_tag_prefix) {
                continue;
            }

            match r.tag_name[self.release_tag_prefix.len()..].parse() {
                Ok(version) => {
                    output.push(Release {
                        id: r.tag_name.clone(),
                        changelog: r.body.clone(),
                        version,
                        variants: self.get_variants_from_response(&r)
                    })
                },
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

            let spec_name = a.name[self.artifact_prefix.len()..].trim_end_matches(".exe").to_string();
            let mut parts = spec_name.split('-');
            
            let arch = match parts.next_back() {
                Some(spec_arch) => spec_arch.to_string(),
                None => ARCH.to_string()
            };
            
            let platform = match parts.next_back() {
                Some(os) => os.to_string(),
                None => OS.to_string()
            };

            variants.push(ReleaseVariant {
                id: a.name.clone(),
                arch,
                platform
            })
        }

        variants
    }

    async fn download_to_file<W: std::io::Write + Send>(&self, uri: Uri, into: &mut W) -> Result<(), errors::Error> {
        let mut recursion_limit = 5;
        let mut current_uri = uri.clone();

        while recursion_limit > 0 {
            recursion_limit -= 1;

            let req = hyper::Request::get(current_uri)
                .header("User-Agent", "Git-Tool/".to_string() + env!("CARGO_PKG_VERSION"))
                .body(hyper::Body::empty())
                .map_err(|e| errors::system_with_internal(
                    "Unable to construct web request for Git-Tool releases.",
                    "Please report this error to us by opening a ticket in GitHub.",
                    e))?;

            let resp = self.client.request(req).await?;

            match resp.status() {
                http::StatusCode::OK => {
                    let mut body = resp.into_body();

                    while let Some(buf) = body.data().await {
                        let buf = buf?;
                        into.write_all(&buf)?;
                    }

                    return Ok(())
                },
                http::StatusCode::FOUND | http::StatusCode::MOVED_PERMANENTLY => {
                    let new_location = resp.headers().get("Location")
                        .ok_or(errors::system(
                    "GitHub returned a redirect response without an appropriate Location header.",
                    "Please report this issue to us on GitHub so that we can investigate and implement a fix/workaround for the problem."))
                    .and_then(|h| h.to_str()
                        .map_err(|e| errors::system_with_internal(
                            "GitHub returned a redirect response without an appropriate Location header.",
                            "Please report this issue to us on GitHub so that we can investigate and implement a fix/workaround for the problem.",
                            e)))?;

                    current_uri = new_location.parse()?;
                    continue;
                },
                http::StatusCode::TOO_MANY_REQUESTS => {
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

        Err(errors::system(
            &format!("Reached redirect limit when attempting to download the update for your platform ({})", uri),
            "Please report this issue to us on GitHub so that we can investigate and implement a fix/workaround for the problem."))
    }
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    pub name: String,
    pub tag_name: String,
    pub body: String,
    pub prerelease: bool,
    pub assets: Vec<GitHubAsset>
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    pub name: String
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::{Mutex, Arc}, io::Write};

    #[tokio::test]
    async fn test_get_releases() {
        let source = GitHubSource::default();

        let releases = source.get_releases().await.unwrap();

        assert_ne!(releases.len(), 0);
        for release in releases {
            assert!(release.id.contains(&release.version.to_string()), "the release version should be derived from the tag");
            assert_ne!(&release.changelog, "", "the release changelog should not be empty");
        }
    }

    #[tokio::test]
    async fn test_download() {
        let source = GitHubSource::default();

        let releases = source.get_releases().await.unwrap();
        let latest = Release::get_latest(releases.iter()).expect("There should be an available release");
        let variant = latest.variants.first().expect("There should be a variant available");

        let mut target = sink();

        source.get_binary(&latest, &variant, &mut target).await.unwrap();

        assert!(target.get_length() > 0);
    }

    fn sink() -> Sink {
        Sink {
            length: Arc::new(Mutex::new(0))
        }
    }

    struct Sink {
        length: Arc<Mutex<usize>>
    }

    impl Sink {
        pub fn get_length(&self) -> usize {
            self.length.lock().map(|m| *m).unwrap_or_default()
        }
    }

    impl Write for Sink {
        #[inline]
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.length.lock().map(|mut m| {
                *m += buf.len();
                buf.len()
            }).map_err(|_| {
                std::io::ErrorKind::Other.into()
            })
        }
    
        #[inline]
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
}
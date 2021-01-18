use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
struct GHReleaseRes {
    url: String,
    html_url: String,
    assets_url: String,
    upload_url: String,
    tarball_url: Option<String>,
    zipball_url: Option<String>,
    id: i64,
    node_id: String,
    tag_name: String,
    target_commitish: String,
    name: Option<String>,
    body: Option<String>,
    draft: bool,
    prerelease: bool,
    created_at: String,
    published_at: Option<String>,
    author: Option<GHAuthorRes>,
    assets: Vec<GHAssetRes>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GHAuthorRes {
    login: String,
    id: i64,
    node_id: String,
    avatar_url: String,
    gravatar_id: String,
    url: String,
    html_url: String,
    followers_url: String,
    following_url: String,
    gists_url: String,
    starred_url: String,
    subscriptions_url: String,
    organizations_url: String,
    repos_url: String,
    events_url: String,
    received_events_url: String,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    account_type: String,
    site_admin: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct GHAssetRes {
    url: String,
    id: i64,
    node_id: String,
    name: String,
    label: String,
    uploader: GHAuthorRes,
    content_type: String,
    state: String,
    size: i64,
    download_count: i64,
    created_at: String,
    updated_at: String,
    browser_download_url: String,
}

#[derive(Debug)]
struct Res {
    name: String,
    assets: Vec<ResAsset>,
}

#[derive(Debug)]
struct ResAsset {
    name: String,
    url: String,
}

struct LSInfo {
    name: String,
    url: String,
    bin_name: String,
    gh_release: GHRelease,
}

struct GHRelease {
    linux_bin_name: String,
    win_bin_name: String,
    mac_bin_name: String,
}

impl LSInfo {
    fn bin_dir(&self) -> PathBuf {
        let p = env::home_dir().unwrap();
        p.join(".lsm").join(self.name.clone())
    }

    fn bin_path(&self) -> PathBuf {
        self.bin_dir().join(self.bin_name.clone())
    }

    fn create_bin_dir(&self) -> std::io::Result<()> {
        fs::create_dir_all(self.bin_dir())?;
        Ok(())
    }

    fn client(&self) -> Result<reqwest::blocking::Client, reqwest::Error> {
        reqwest::blocking::Client::builder()
            .user_agent("rust")
            .build()
    }

    fn get_release(&self) -> Result<Res, reqwest::Error> {
        let release: GHReleaseRes = self
            .client()?
            .get(&format!("{}/{}", self.url, "latest"))
            .send()?
            .json()?;

        Ok(Res {
            name: release.tag_name,
            assets: release
                .assets
                .iter()
                .map(|a| ResAsset {
                    name: a.name.clone(),
                    url: a.browser_download_url.clone(),
                })
                .collect::<Vec<ResAsset>>(),
        })
    }

    fn get_bin(&self) -> Bytes {
        let url = self.get_download_url();
        self.client()
            .unwrap()
            .get(&url)
            .send()
            .unwrap()
            .bytes()
            .unwrap()
    }

    fn get_download_url(&self) -> String {
        self.get_release()
            .unwrap()
            .assets
            .iter()
            .find(|a| a.name == self.gh_release.bin_name())
            .unwrap()
            .url
            .clone()
    }

    fn get_releases(&self, client: reqwest::blocking::Client) -> Result<Vec<Res>, reqwest::Error> {
        let releases: Vec<GHReleaseRes> = self.client()?.get(&self.url).send()?.json()?;
        Ok(releases
            .iter()
            .map(|r| Res {
                name: r.tag_name.clone(),
                assets: r
                    .assets
                    .iter()
                    .map(|a| ResAsset {
                        name: a.name.clone(),
                        url: a.browser_download_url.clone(),
                    })
                    .collect::<Vec<ResAsset>>(),
            })
            .collect())
    }
}

impl GHRelease {
    #[cfg(target_os = "linux")]
    fn bin_name(&self) -> String {
        self.linux_bin_name.clone()
    }

    #[cfg(target_os = "windows")]
    fn bin_name(&self) -> String {
        self.win_bin_name.clone()
    }

    #[cfg(target_os = "macos")]
    fn bin_name(&self) -> String {
        self.mac_bin_name.clone()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ls = LSInfo {
        name: "awesome-lsp".to_string(),
        bin_name: "awesome-lsp".to_string(),
        url: "https://api.github.com/repos/h-michael/awesome-lsp/releases".to_string(),
        gh_release: GHRelease {
            linux_bin_name: "awesome-lsp-linux".to_string(),
            mac_bin_name: "awesome-lsp-mac".to_string(),
            win_bin_name: "awesome-lsp-windows.exe".to_string(),
        },
    };

    ls.create_bin_dir().unwrap();
    let mut file = File::create(ls.bin_path())?;
    let content = ls.get_bin();
    file.write_all(&content)?;
    Ok(())
}

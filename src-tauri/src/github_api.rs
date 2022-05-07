use std::collections::HashMap;
use reqwest::Client;
use serde_json::Value;
use once_cell::sync::Lazy;
use tokio::sync::Mutex;

static RESPONSE_CACHE : Lazy<Mutex<HashMap<String, (String, Value)>>> = Lazy::new(|| Default::default());

pub async fn make_github_api_request(client : &reqwest::Client, route : String) -> Result<serde_json::Value, String> {
    let mut response_cache = RESPONSE_CACHE.lock().await;
    // Make request with ETag, if possible
    let response = match response_cache.get(&route) {
        Some((e_tag, _)) => match
            client.get(format!("https://api.github.com/{}", route))
                .header("If-None-Match", e_tag)
                .send()
                .await {
            Ok(r) => r,
            Err(e) => return Err(e.to_string())
        },
        None => match client.get(format!("https://api.github.com/{}", route))
                .send()
                .await {
            Ok(r) => r,
            Err(e) => return Err(e.to_string())
        }
    };
    // Return cached value if resource has not changed
    if response.status() == 304 {
        return Ok(response_cache.get(&route).unwrap().1.clone())
    }
    if !response.status().is_success() {
        return Err(format!("Request returned status code {} with content : {}", response.status().as_u16(), response.text().await.unwrap()))
    }
    let e_tag_option = response.headers().get("ETag").map(|v| v.to_str().unwrap().to_string());
    match response.json::<serde_json::Value>().await {
        Ok(j) => {
            // Cache response if ETag is available
            if let Some(e_tag) = e_tag_option {
                response_cache.insert(route, (e_tag, j.clone()));
            }
            Ok(j)
        },
        Err(e) => Err(e.to_string())
    }
}

pub fn get_assets(newest_release: &Value) -> Vec<Value> {
    let mut assets = newest_release.get("assets").unwrap().as_array().unwrap().clone();
    // Only keep assets that are .dll or .zip files
    assets.retain(|v| match v.get("content_type").unwrap().as_str().unwrap() {
        "application/x-msdownload" => true,
        "application/x-zip-compressed" => true,
        "application/zip" => true,
        "application/octet-stream" => true,
        _ => false
    });
    // Sort by download count
    assets.sort_by(|a, b| {
        let a_download_count = a.get("download_count").unwrap().as_u64().unwrap();
        let b_download_count = b.get("download_count").unwrap().as_u64().unwrap();
        b_download_count.cmp(&a_download_count)
    });
    assets
}

pub async fn get_newest_release(client: &Client, username: &String, repository_name: &String) -> Result<Value, String> {
    // Get releases
    let releases_value = make_github_api_request(&client, format!("repos/{}/{}/releases", username, repository_name)).await?;
    let releases = releases_value.as_array().unwrap();
    let newest_release = match releases.iter().find(|v| v.get("draft").unwrap().as_bool().unwrap() == false) {
        Some(r) => r,
        None => return Err("No releases were found".to_string())
    };
    Ok(newest_release.clone())
}

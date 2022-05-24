use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_defaults() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/search/repositories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!(
            {
                "total_count": 0,
                "items": [],
                "other": {}
            }
        )))
        .expect(1)
        .mount(&mock_server)
        .await;

    let url = format!("{}/search/repositories", &mock_server.uri());
    println!("{}", &url);

    let cli = Command::cargo_bin("cli")
        .unwrap()
        .env("GITHUB_ACCESS_TOKEN", "PAT_TOKEN")
        .args(&["-l", "rust"])
        .args(&["-p", "1"])
        .args(&["-g", &url])
        .assert();

    cli.success();
}
#[tokio::test]
async fn test_simple_scenario() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/search/repositories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!(
            {
                "total_count": 555,
                "items": [],
                "other": {}
            }
        )))
        .expect(2)
        .mount(&mock_server)
        .await;

    let url = format!("{}/search/repositories", &mock_server.uri());
    println!("{}", &url);

    let cli = Command::cargo_bin("cli")
        .unwrap()
        .env("GITHUB_ACCESS_TOKEN", "PAT_TOKEN")
        .args(&["-l", "rust"])
        .args(&["-p", "40"])
        .args(&["-g", &url])
        .assert();

    cli.success();
}
#[tokio::test]
async fn test_help() {
    Command::cargo_bin("cli")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::starts_with(
            "bus_factor 0.1.0\nSimple program to fetch GitHub",
        ));
}

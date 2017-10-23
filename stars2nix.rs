// cargo-deps: serde="1", serde_json="1", reqwest="0.8"
extern crate serde;
#[macro_use] extern crate serde_json;
extern crate reqwest;

use std::io::Write;
use std::fs::{ File, create_dir };
use std::path::PathBuf;
use reqwest::{ Client, Response, Request };
use reqwest::header::{ Authorization, Bearer };
use serde_json::Value;

type Result<T> = std::result::Result<T, Box<std::error::Error>>;


const API_URL: &str = "https://api.github.com/graphql";

const QUERY_WHOAMI: &str = r#"{ viewer { login } }"#;

const QUERY_STARS: &str = r#"query($login: String!, $cursor: String) {
  user(login: $login) {
    starredRepositories(first: 100, after: $cursor) {
      pageInfo {
        endCursor
        hasNextPage
      }
      edges {
        starredAt
        node {
          owner {
            login
            url
          }
          name
          url
          primaryLanguage {
            name
          }
          createdAt
          pushedAt
          description
          homepageUrl
        }
      }
    }
  }
}"#;

fn main() {
    let token = match std::env::var("GITHUB_API_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            eprintln!("missing GITHUB_API_TOKEN");
            std::process::exit(1);
        }
    };

    let cursor = std::env::var("START_CURSOR").ok();

    match run(token, cursor) {
        Ok(_) => (),
        Err(e) => eprintln!("{}", e)
    }
}

fn whoami(token: String) -> Result<String> {
    let body = json!({
        "query": QUERY_WHOAMI,
    });

    let client = Client::new();
    let mut result =
        client
        .post(API_URL)
        .header(Authorization(Bearer { token: token }))
        .json(&body)
        .send()?;

    let result = result.json::<serde_json::Value>()?;

    Ok(result["data"]["viewer"]["login"].as_str().map(ToOwned::to_owned).expect("login"))
}

fn get_page(token: String, login: &str, cursor: Option<&str>) -> Result<(bool, Option<String>)> {
    let body = json!({
        "query": QUERY_STARS,
        "variables":  {
            "login": login,
            "cursor": cursor,
        },
    });

    let client = Client::new();
    let mut result =
        client
        .post(API_URL)
        .header(Authorization(Bearer { token: token }))
        .json(&body)
        .send()?;

    let result = result.json::<serde_json::Value>()?;
    let result = &result["data"]["user"]["starredRepositories"];

    let page = (
        result["pageInfo"]["hasNextPage"].as_bool().unwrap(),
        result["pageInfo"]["endCursor"].as_str().map(ToOwned::to_owned),
    );

    let result = result["edges"].as_array().unwrap();
    for star in result {
        write_star(star)?
    }

    Ok(page)
}

fn write_star(star: &Value) -> Result<()> {
    let starred = star["starredAt"].as_str().expect("starredAt");
    let owner = star["node"]["owner"]["login"].as_str().expect("login");
    let name = star["node"]["name"].as_str().expect("name");
    let repo_url = star["node"]["url"].as_str().expect("url");
    let main_url = star["node"]["homepageUrl"].as_str().unwrap_or("");
    let created = star["node"]["createdAt"].as_str().expect("createdAt");
    let description =
        star["node"]["description"].as_str().unwrap_or("")
        .replace("\\", "\\\\")
        .replace("${", "\\${")
        .replace("\"", "\\\"");

    let mut path = PathBuf::new();
    path.push("stars");
    path.push(owner);

    let _ = create_dir(&path);

    path.push(name);
    path.set_extension("nix");

    let contents = format!(
 r#"{{
    repoName = "{name}";
    repoOwner = "{owner}";
    repoUrl = "{repo_url}";
    mainUrl = "{main_url}";
    created = "{created}";
    starred = "{starred}";
    description = "{description}";
}}"#,
        name = name,
        owner = owner,
        repo_url = repo_url,
        main_url = main_url,
        created = created,
        starred = starred,
        description = description,
    );

    println!("# {}", path.to_string_lossy());

    let mut file = File::create(path)?;
    file.write_all(contents.as_bytes())?;

    Ok(())
}

fn run(token: String, mut cursor: Option<String>) -> Result<()> {
    let login = whoami(token.clone())?;

    loop {
        let next = get_page(token.clone(), &login, cursor.as_ref().map(AsRef::as_ref))?;

        cursor = next.1;

        if !next.0 {
            break
        }
    }

    if let Some(cursor) = cursor {
        println!("export START_CURSOR=\"{}\"", cursor);
    }
    Ok(())
}


use clap::Parser;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::path::Path;
use tokio::fs::{self, DirEntry};
use tokio::io;

#[derive(Serialize, Deserialize)]
struct GitlabCommit {
    id: String,
    short_id: String,
    created_at: String,
    parent_ids: Vec<String>,
    title: String,
    message: String,
    author_name: String,
    author_email: String,
    authored_date: String,
    committer_name: String,
    committer_email: String,
    committed_date: String,
    web_url: String,
}

#[derive(Serialize, Deserialize)]
struct PackageGitlab {
    id: i32,
    url: String,
}

#[derive(Serialize, Deserialize)]
struct Package {
    name: String,
    version: String,
    _gitlab: PackageGitlab,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    project: String,

    #[arg(short, long)]
    key: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let key = args.key.as_str();
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "PRIVATE-TOKEN",
        key.parse().unwrap(),
    );
    let http_client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .expect("http 客户端创建失败");

    
    let project_dir = read_project(args).await.unwrap();
    let package = get_package_info(project_dir).await.unwrap();
    get_project_commits(http_client, package).await.unwrap();
}

async fn read_project(args: Args) -> Result<DirEntry, String> {
    let project_path = args.project;
    let path = Path::new(&project_path);
    let mut dir = fs::read_dir(path).await.expect("读取项目文件夹失败");
    let package_file_name = OsString::from("package.json");

    let empty_package = String::from("未找到 package.json");
    while let Some(entry) = dir.next_entry().await.unwrap() {
        if package_file_name == entry.file_name() {
            println!("找到 package.json ");
            return Ok(entry);
        }
    }
    Err(empty_package)
}

/**
 * 获取node项目 package 信息
 */
async fn get_package_info(entry: DirEntry) -> io::Result<Package> {
    let file_content = fs::read_to_string(entry.path())
        .await
        .expect("读取 Package.json 失败");
    let json_data = file_content.as_str();
    let v: Package = serde_json::from_str(json_data)?;
    Ok(v)
}

async fn get_project_commits(
    http_client: Client,
    package: Package,
) -> Result<(), Box<dyn std::error::Error>> {
    // println!("get_project_commits 发送请求");
    let url = format!(
        "http://git.mcrd2.com/api/v4/projects/{}/repository/commits/",
        package._gitlab.id
    );
    println!("{:?} 发送请求", url);
    let resp = http_client.get(url).query(&[("ref_name", "stable")]).send().await;
    match resp {
        Ok(resp) => {
            if resp.status().as_u16() == 200 {
                let data = resp.json::<Vec<GitlabCommit>>().await;
                match data {
                    Ok(d) => {
                        println!("{:?}", d[0].title);
                    }
                    Err(e) => {
                        println!("GitlabCommit 解析失败 , {:?}", e)
                    }
                }
            } else {
                println!("请求错误 {:?}", resp.text().await)
            }
        }
        Err(e) => {
            println!("请求错误 {:?}", e.to_string());
        }
    };

    Ok(())
}

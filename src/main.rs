use std::{ffi::OsString, fs::DirEntry};
use std::fs;
use std::path::Path;
use clap::Parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};


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
    url: String
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
}




fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let project_path = args.project;
    let path = Path::new(&project_path);
    
    let dir = fs::read_dir(path)?;

    let mut package:Option<Package> = None;

    let http_client = reqwest::Client::new();
    
    for dir_entry in dir {
        let entry = dir_entry?;
        let file_namne = entry.file_name();
        let target_file_name = OsString::from("package.json");
        if file_namne == target_file_name {
            let info = get_package_info(entry).unwrap();
            package = Some(info);
        }
    }

    match package {
        Some(p) => {
            println!("package info name={:?} version={:?}", p.name, p.version);
            println!("gitlab info id={:?} url={:?}", p._gitlab.id, p._gitlab.url);
            
            // TODO: 解决 await 如何在同步方法中执行
            let commits = get_project_commits(http_client, p).await;

            match commits {
                Ok(_) => {

                }
                Err(_) => {
                    
                }
            };
            
        }
        None => {
            println!("未找到 package 信息")
        }
    }
    

    

    Ok(())
}

/**
 * 获取node项目 package 信息
 */
fn get_package_info(entry: DirEntry,) -> std::io::Result<Package> {
    let file_content = fs::read_to_string(entry.path()).unwrap();
    let json_data = file_content.as_str();
    let v: Package = serde_json::from_str(json_data)?;

    Ok(v)
}

async fn get_project_commits(http_client:Client, package: Package) -> Result<(), Box<dyn std::error::Error>>  {
    println!("get_project_commits 发送请求");
    let url = format!("http://git.mcrd2.com/api/v4/projects/{}/repository/commits/", package._gitlab.id);
    
    let resp = http_client.post(url).send().await;
    match resp {
        Ok(resp) => {
            let data = resp.json::<Vec<GitlabCommit>>().await;
            match data {
                Ok(d) => {
                    println!("{:?}", d[0].title);
                }
                Err(e) => {
                    println!("GitlabCommit 解析失败 , {:?}", e)
                }
            }
        },
        Err(e) => {
            println!("请求错误 {:?}", e.to_string());
        }
    };

    Ok(())
}
use anyhow::{anyhow, Ok, Result};
use clap::{Parser, Subcommand};
use reqwest::{header, Client, Response, Url};
use std::str::FromStr;
#[derive(Parser, Debug)]
#[clap(
    name = "httpie",
    version = "1.0",
    author = "luowen",
    about = "一个简单的 Rust 版 httpie 实现"
)]
struct Opts {
    #[clap(subcommand)]
    subcmd: Subcommands,
}

#[derive(Subcommand, Debug)]
enum Subcommands {
    /// 发送 GET 请求  
    #[clap(name = "get", about = "发送 GET 请求到指定 URL")]
    Get(GetOpts),
    /// 发送 POST 请求  
    #[clap(name = "post", about = "发送 POST 请求到指定 URL，并包含请求体")]
    Post(PostOpts),
}

#[derive(Parser, Debug)]
struct GetOpts {
    /// HTTP 请求的 URL  
    #[clap(required = true)]
    #[arg(value_parser = parse_url)]
    url: Url,
}

#[derive(Parser, Debug)]
struct PostOpts {
    /// HTTP 请求的 URL  
    #[clap(required = true)]
    #[arg(value_parser = parse_url)]
    url: Url,
    /// HTTP 请求的 body（多行字符串将合并）  
    #[clap(name = "body", required = false)]
    #[arg(value_parser = parse_kv_pair)]
    body: Vec<KvPair>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct KvPair {
    k: String,
    v: String,
}

impl FromStr for KvPair {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split("=");
        let err = || anyhow!(format!("Failed to parse {}", s));
        Ok(Self {
            // 从迭代器中取第一个结果作为 key，迭代器返回 Some(T)/None
            // 我们将其转换成 Ok(T)/Err(E)，然后用 ? 处理错误
            k: (split.next().ok_or_else(err)?).to_string(),
            // 从迭代器中取第二个结果作为 value
            v: (split.next().ok_or_else(err)?).to_string(),
        })
    }
}

fn parse_url(s: &str) -> Result<Url, anyhow::Error> {
    Ok(s.parse()?)
}

fn parse_kv_pair(s: &str) -> Result<KvPair> {
    Ok(s.parse()?)
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    // let client = Client::new();
    let result = match opts.subcmd {
        Subcommands::Get(get) => {
            println!("GET request to: {}", get.url);
            // ... 处理 GET 请求
        }
        Subcommands::Post(post) => {
            println!("POST body:{:?}", post.body);
            // println!("POST request to: {}", post.url);
            // println!("Body:\n{}", post.body.join("\n"));
            // ... 处理 POST 请求
        }
    };

    Ok(result)
}

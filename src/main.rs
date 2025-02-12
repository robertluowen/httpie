use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use colored::*;
use mime::Mime;
use reqwest::{header, Client, Response, Url};
use std::{collections::HashMap, str::FromStr};
// use syntect::easy::HighlightLines;
// use syntect::highlighting::{Style, ThemeSet};
// use syntect::parsing::SyntaxSet;
// use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};
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
#[derive(Debug, Clone, PartialEq)]
struct KvPair {
    k: String,
    v: String,
}

/// 当我们实现 FromStr trait 后，可以用 str.parse() 方法将字符串解析成 KvPair
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

/// 因为我们为 KvPair 实现了 FromStr，这里可以直接 s.parse() 得到 KvPair
fn parse_kv_pair(s: &str) -> Result<KvPair> {
    Ok(s.parse()?)
}

async fn get(client: Client, args: &GetOpts) -> Result<()> {
    let resp = client.get(args.url.as_str()).send().await?;
    Ok(print_resp(resp).await?)
}

async fn post(client: Client, args: &PostOpts) -> Result<()> {
    let mut body = HashMap::new();
    for pair in args.body.iter() {
        body.insert(&pair.k, &pair.v);
    }
    let resp = client.post(args.url.as_str()).json(&body).send().await?;
    Ok(print_resp(resp).await?)
}

// 打印服务器版本号 + 状态码
fn print_status(resp: &Response) {
    let status = format!("{:?} {}", resp.version(), resp.status()).blue();
    println!("{}\n", status);
}

// 打印服务器返回的 HTTP header
fn print_headers(resp: &Response) {
    for (name, value) in resp.headers() {
        // print_highlighting(name.to_string());

        println!("{}: {:?}", name.to_string().green(), value);
    }

    print!("\n");
}

// fn print_highlighting(s: String) {
//     let ps = SyntaxSet::load_defaults_newlines();
//     let ts = ThemeSet::load_defaults();

//     let syntax = ps.find_syntax_by_extension("rs").unwrap();
//     let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
//     for line in LinesWithEndings::from(&s) {
//         let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
//         let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
//         print!("{:?}", escaped);
//     }
// }

/// 打印服务器返回的 HTTP body
fn print_body(m: Option<Mime>, body: &String) {
    match m {
        // 对于 "application/json" 我们 pretty print
        Some(v) if v == mime::APPLICATION_JSON => {
            println!("{}", jsonxf::pretty_print(body).unwrap().cyan())
        }
        // 其它 mime type，我们就直接输出
        _ => println!("{}", body),
    }
}

/// 打印整个响应
async fn print_resp(resp: Response) -> Result<()> {
    print_status(&resp);
    print_headers(&resp);
    let mime = get_content_type(&resp);
    let body = resp.text().await?;
    print_body(mime, &body);
    Ok(())
}

/// 将服务器返回的 content-type 解析成 Mime 类型
fn get_content_type(resp: &Response) -> Option<Mime> {
    resp.headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().parse().unwrap())
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let client = Client::new();
    let result = match opts.subcmd {
        Subcommands::Get(ref args) => get(client, args).await?,
        Subcommands::Post(ref args) => post(client, args).await?,
    };

    Ok(result)
}

// 仅在 cargo test 时才编译
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_url_works() {
        assert!(parse_url("abc").is_err());
        assert!(parse_url("http://abc.xyz").is_ok());
        assert!(parse_url("https://httpbin.org/post").is_ok());
    }

    #[test]
    fn parse_kv_pair_works() {
        assert!(parse_kv_pair("a").is_err());
        assert_eq!(
            parse_kv_pair("a=1").unwrap(),
            KvPair {
                k: "a".into(),
                v: "1".into()
            }
        );

        assert_eq!(
            parse_kv_pair("b=").unwrap(),
            KvPair {
                k: "b".into(),
                v: "".into()
            }
        );
    }
}

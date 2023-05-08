use anyhow::{Ok, Result};
use clap::{Args, Parser, Subcommand};
use reqwest::Url;

/// 一个用Rust实现的原生HTTPie工具
#[derive(Parser, Debug)]
#[command(
    name = "HTTPie",
    author = "Baisen Qiu <abosen@qq.com>",
    version = "1.0"
)]
struct Opts {
    #[command(subcommand)]
    subcmd: SubCommand,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    Get(Get),
    Post(Post),
}

#[derive(Args, Debug)]
struct Get {
    #[arg(value_parser=parse_url)]
    url: String,
}
#[derive(Args, Debug)]
struct Post {
    #[arg(value_parser=parse_url)]
    url: String,
    body: Vec<String>,
}

fn parse_url(url: &str) -> Result<String> {
    url.parse::<Url>()?;
    Ok(url.into())
}

/* 测试一下
cargo run post httpbin.org/post a=1 b=2
Opts { subcmd: Post(Post { url: "httpbin.org/post", body: ["a=1", "b=2"] }) }
 */
fn main() {
    let opts = Opts::parse();
    println!("{:?}", opts)
}

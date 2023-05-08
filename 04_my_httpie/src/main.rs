use std::str::FromStr;

use anyhow::{anyhow, Ok, Result};
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
    #[arg(value_parser=parse_kv_pair)]
    body: Vec<KvPair>,
}

fn parse_url(url: &str) -> Result<String> {
    url.parse::<Url>()?;
    Ok(url.into())
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
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
            k: split.next().ok_or_else(err)?.to_string(),
            v: split.next().ok_or_else(err)?.to_string(),
        })
    }
}

fn parse_kv_pair(s: &str) -> Result<KvPair> {
    s.parse()
}

fn main() {
    let opts = Opts::parse();
    println!("{:?}", opts)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn error_if_post_body_not_in_pair() {
        let result = Opts::try_parse_from(vec![
            "my_httpie",
            "post",
            "https://httpbin.org/post",
            "a=1",
            "b",
        ]);
        assert!(result.is_err());
        assert!(result
            .err()
            .unwrap()
            .to_string()
            .starts_with("error: invalid value 'b' for '[BODY]...': Failed to parse b"));
    }
    #[test]
    fn error_if_url_illegal() {
        let result = Opts::try_parse_from(vec!["my_httpie", "post", "abc", "a=1"]);
        assert!(result.is_err());
        assert!(result
            .err()
            .unwrap()
            .to_string()
            .starts_with("error: invalid value 'abc' for '<URL>': relative URL without a base"));
    }
    #[test]
    fn success_parse() {
        let result = Opts::try_parse_from(vec![
            "my_httpie",
            "post",
            "https://httpbin.org/post",
            "a=1",
            "b=2",
        ]);
        assert!(result.is_ok());
        match result.unwrap().subcmd {
            SubCommand::Post(post) => {
                assert_eq!(post.url, "https://httpbin.org/post");
                assert_eq!(
                    post.body,
                    vec![
                        KvPair {
                            k: "a".into(),
                            v: "1".into()
                        },
                        KvPair {
                            k: "b".into(),
                            v: "2".into()
                        }
                    ]
                );
            }
            _ => panic!("解析错误"),
        };
    }
}

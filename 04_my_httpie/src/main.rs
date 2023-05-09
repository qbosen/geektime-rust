use std::{collections::HashMap, str::FromStr};

use anyhow::{anyhow, Ok, Result};
use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use mime::Mime;
use reqwest::{header, Client, Response, Url};
use syntect::{
    easy::HighlightLines,
    highlighting::{Style, ThemeSet},
    parsing::SyntaxSet,
    util::{as_24_bit_terminal_escaped, LinesWithEndings},
};

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

/// feed get with an url and retrieve the response
#[derive(Args, Debug)]
struct Get {
    /// 请求 URL
    #[arg(value_parser=parse_url)]
    url: String,
}
/// feed post with and url and optional key=value pairs.
/// post data as JSON and retrieve the response
#[derive(Args, Debug)]
struct Post {
    /// 请求 URL
    #[arg(value_parser=parse_url)]
    url: String,
    /// key=value 样式的body
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

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opts::parse();
    let mut headers = header::HeaderMap::new();
    headers.insert("X-POWERED-BY", "Rust".parse()?);
    headers.insert(header::USER_AGENT, "Rust Httpie".parse()?);
    let client = Client::builder().default_headers(headers).build()?;
    let result = match opts.subcmd {
        SubCommand::Get(ref args) => get(client, args).await?,
        SubCommand::Post(ref args) => post(client, args).await?,
    };
    Ok(result)
}

async fn get(client: Client, args: &Get) -> Result<()> {
    // args是一个不可变引用,无法移动args.url的所有权; 这里传递&String,有对应的IntoUrl实现 impl<'a> IntoUrl for &'a String {}
    let resp = client.get(&args.url).send().await?;
    Ok(print_resp(resp).await?)
}

async fn post(client: Client, args: &Post) -> Result<()> {
    let mut body = HashMap::new();
    for pair in args.body.iter() {
        body.insert(&pair.k, &pair.v);
    }
    let resp = client.post(&args.url).json(&body).send().await?;
    Ok(print_resp(resp).await?)
}

/// 打印服务器版本号+状态码
fn print_status(resp: &Response) {
    let status = format!("{:?} {}", resp.version(), resp.status()).blue();
    println!("{}\n", status);
}
/// 打印响应头
fn print_header(resp: &Response) {
    for (k, v) in resp.headers() {
        println!("{}: {:?}", k.to_string().green(), v);
    }
    println!();
}

/// 打印HTTP body
fn print_body(m: Option<Mime>, body: &String) {
    match m {
        Some(v) if v == mime::APPLICATION_JSON => print_syntect(body, "json"),
        Some(v) if v == mime::TEXT_HTML => print_syntect(body, "html"),
        _ => println!("{}", body),
    }
}

fn get_content_type(resp: &Response) -> Option<Mime> {
    resp.headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().parse().unwrap())
}

fn print_syntect(s: &str, ext: &str) {
    // Load these once at the start of your program
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let syntax = ps.find_syntax_by_extension(ext).unwrap();
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
    for line in LinesWithEndings::from(s) {
        let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
        let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
        print!("{}", escaped);
    }
}

/// 打印整个响应
async fn print_resp(resp: Response) -> Result<()> {
    print_status(&resp);
    print_header(&resp);
    let mime = get_content_type(&resp);
    let body = resp.text().await?;
    print_body(mime, &body);
    Ok(())
}

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
    #[cfg(test)]
    mod tests_clap {
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
            assert!(result.err().unwrap().to_string().starts_with(
                "error: invalid value 'abc' for '<URL>': relative URL without a base"
            ));
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
}

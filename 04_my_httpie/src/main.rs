use clap::{Args, Parser, Subcommand};

/// 一个用Rust实现的原生HTTPie工具
#[derive(Parser, Debug)]
#[command(name = "HTTPie", author = "Baisen Qiu <abosen@qq.com>", version = "1.0")]
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
    url: String,
}
#[derive(Args, Debug)]
struct Post {
    url: String,
    body: Vec<String>,
}
/* 测试一下
cargo run post httpbin.org/post a=1 b=2
Opts { subcmd: Post(Post { url: "httpbin.org/post", body: ["a=1", "b=2"] }) }
 */
fn main() {
    let opts = Opts::parse();
    println!("{:?}", opts)
}

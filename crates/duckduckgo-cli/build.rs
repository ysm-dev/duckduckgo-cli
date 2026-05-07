fn main() {
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_owned());
    println!("cargo:rustc-env=DUCKDUCKGO_TARGET={target}");
    let out = std::env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo");
    let command = clap::Command::new("duckduckgo")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Agent-first DuckDuckGo search CLI")
        .arg(clap::Arg::new("query").num_args(0..).value_name("QUERY"))
        .arg(clap::Arg::new("num").short('n').long("num").value_name("N"))
        .arg(
            clap::Arg::new("region")
                .short('r')
                .long("region")
                .value_name("CODE"),
        )
        .arg(clap::Arg::new("page").long("page").value_name("N"))
        .arg(clap::Arg::new("json").long("json"))
        .arg(
            clap::Arg::new("completion")
                .long("completion")
                .value_name("SHELL"),
        );
    clap_mangen::Man::new(command)
        .title("DUCKDUCKGO")
        .section("1")
        .date("2026-05-07")
        .manual("duckduckgo-cli Manual")
        .generate_to(out)
        .expect("generate man page");
}

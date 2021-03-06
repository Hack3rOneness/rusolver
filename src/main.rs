use {
    clap::{value_t, App, Arg},
    futures::stream::StreamExt,
    rand::Rng,
    std::net::IpAddr,
    tokio::{
        self,
        io::{self, AsyncReadExt},
    },
    trust_dns_resolver::{
        config::{NameServerConfigGroup, ResolverConfig, ResolverOpts},
        TokioAsyncResolver,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Eval args
    let matches = App::new("Rusolver")
        .version("0.1.0")
        .author("Eduard Tolosa <edu4rdshl@protonmail.com>")
        .about("Fast DNS resolver written in Rust.")
        .arg(
            Arg::with_name("threads")
                .short("t")
                .long("threads")
                .takes_value(true)
                .help("Number of threads. Default: 500"),
        )
        .arg(
            Arg::with_name("timeout")
                .long("timeout")
                .takes_value(true)
                .help("Timeout in seconds. Default: 1"),
        )
        .arg(
            Arg::with_name("ip")
                .short("i")
                .long("ip")
                .takes_value(false)
                .help("Show the discovered IP addresses. Default: false"),
        )
        .get_matches();

    // Assign values or use defaults
    let show_ip_adress = matches.is_present("ip");
    let threads = value_t!(matches.value_of("threads"), usize).unwrap_or_else(|_| 500);
    let timeout = value_t!(matches.value_of("timeout"), u64).unwrap_or_else(|_| 1);

    // Resolver opts
    let mut options = ResolverOpts::default();
    options.timeout = std::time::Duration::from_secs(timeout);

    // Read stdin
    let mut buffer = String::new();
    let mut stdin = io::stdin();
    stdin.read_to_string(&mut buffer).await?;
    let hosts: Vec<String> = buffer.lines().map(str::to_owned).collect();

    futures::stream::iter(hosts.into_iter().map(|host| async move {
        let dns_ips = vec![
            // Cloudflare
            "1.1.1.1",
            "1.0.0.1",
            // Google
            "8.8.8.8",
            "8.8.4.4",
            // Quad9
            "9.9.9.9",
            "149.112.112.112",
            // OpenDNS
            "208.67.222.222",
            "208.67.220.220",
            // Verisign
            "64.6.64.6",
            "64.6.65.6",
            // UncensoredDNS
            "91.239.100.100",
            "89.233.43.71",
            // dns.watch
            "84.200.69.80",
            "84.200.70.40",
        ];
        if let Ok(ip) = TokioAsyncResolver::tokio(
            ResolverConfig::from_parts(
                None,
                vec![],
                NameServerConfigGroup::from_ips_clear(
                    &[IpAddr::V4(
                        dns_ips[rand::thread_rng().gen_range(0, dns_ips.len())]
                            .parse()
                            .unwrap(),
                    )],
                    53,
                ),
            ),
            options,
        )
        .await
        .unwrap()
        .ipv4_lookup(host.clone() + ".")
        .await
        {
            if show_ip_adress {
                println!(
                    "{};{:?}",
                    host,
                    ip.into_iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                )
            } else {
                println!("{}", host)
            }
        }
    }))
    .buffer_unordered(threads)
    .collect::<Vec<()>>()
    .await;
    Ok(())
}

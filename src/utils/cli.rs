use clap::{Arg, ArgMatches, Command};

/// 解析命令行参数的函数。
pub fn parse_args() -> (ArgMatches, Command<'static>) {
    let app = Command::new("spider")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
        .about("A simple web crawler")
        .subcommand(
            Command::new("create")
                .about("Creates a new crawling task")
                .arg(
                    Arg::new("file")
                        .short('f')
                        .long("file")
                        .takes_value(true)
                        .required(true)
                        .help("The file containing a list of URLs to crawl"),
                )
                .arg(
                    Arg::new("name")
                        .short('n')
                        .long("name")
                        .takes_value(true)
                        .required(true)
                        .help("The name of the crawling task"),
                )
                .arg(
                    Arg::new("deep")
                        .short('d')
                        .long("deep")
                        .takes_value(true)
                        .required(true)
                        .value_parser(clap::value_parser!(u32))
                        .help("The deep of the spider cycle"),
                ),
        )
        .subcommand(
            Command::new("list")
                .about("Lists all crawling tasks")
        )
        .subcommand(
            Command::new("resume")
                .about("Resumes a crawling task")
                .arg(
                    Arg::new("id")
                        .short('i')
                        .long("id")
                        .takes_value(true)
                        .required(true)
                        .value_parser(clap::value_parser!(u32))
                        .help("The ID of the task to resume"),
                ),
        )
        .subcommand(
            Command::new("restart")
                .about("Restarts a crawling task")
                .arg(
                    Arg::new("id")
                        .short('i')
                        .long("id")
                        .takes_value(true)
                        .required(true)
                        .value_parser(clap::value_parser!(u32))
                        .help("The ID of the task to restart"),
                ),
        );

    let matches = app.clone().get_matches();

    (matches, app)
}

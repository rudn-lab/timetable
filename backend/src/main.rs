use backend::*;
use clap::{Parser, ValueEnum};
use std::net::Ipv4Addr;

#[derive(ValueEnum, Debug, Clone, Copy)]
enum AddressOption {
    Local,
    Public,
}

impl From<AddressOption> for Ipv4Addr {
    fn from(value: AddressOption) -> Self {
        match value {
            AddressOption::Local => Self::new(127, 0, 0, 1),
            AddressOption::Public => Self::new(0, 0, 0, 0),
        }
    }
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(value_enum, short, long, default_value_t = AddressOption::Local)]
    address: AddressOption,

    #[arg(short, long, default_value_t = 8080)]
    port: u16,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // access logs are printed with the INFO level so ensure it is enabled by default
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let args = Args::parse();
    let ip = Into::<Ipv4Addr>::into(args.address);

    run_scheduler().await;

    run_server(ip, args.port).await
}

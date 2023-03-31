use backend::run;
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

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let ip = Into::<Ipv4Addr>::into(args.address);

    run(ip, args.port).await
}

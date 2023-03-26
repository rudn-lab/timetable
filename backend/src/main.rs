use backend::run;
use clap::{Parser, ValueEnum};
use std::net::Ipv4Addr;

#[derive(ValueEnum, Debug, Clone, Copy)]
enum AddressOption {
    Local,
    Public,
}

impl Into<Ipv4Addr> for AddressOption {
    fn into(self) -> Ipv4Addr {
        match self {
            AddressOption::Local => Ipv4Addr::new(127, 0, 0, 1),
            AddressOption::Public => Ipv4Addr::new(0, 0, 0, 0),
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
    let args = Args::parse();
    let ip = Into::<Ipv4Addr>::into(args.address);

    run(ip, args.port).await
}

use clap::{Parser, Subcommand};
use koad_proto::kernel::kernel_service_client::KernelServiceClient;
use koad_proto::kernel::CommandRequest;
use tonic::transport::{Endpoint, Uri};
use tower::service_fn;
use tokio::net::UnixStream;
use hyper_util::rt::tokio::TokioIo;

#[derive(Parser)]
#[command(name = "koad")]
#[command(about = "KoadOS v3 Command Line Interface", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Socket path for the kspine Kernel
    #[arg(long, default_value = "/home/ideans/.koad-os/kspine.sock")]
    socket: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Execute a command in the Kernel
    Run {
        /// The name of the command or tool
        name: String,
        /// Optional arguments as key=value pairs
        #[arg(short, long)]
        args: Vec<String>,
    },
    /// Check the status of the Kernel
    Status,
    /// Execute a tool from the DoodSkills Toolbox (Admiral Only)
    Dood {
        /// The name of the tool
        name: String,
        /// Optional arguments as key=value pairs
        #[arg(short, long)]
        args: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Connect to the Kernel via UDS
    let socket_path = cli.socket.clone();
    let channel = Endpoint::try_from("http://[::]:50051")?
        .connect_with_connector(service_fn(move |_: Uri| {
            let path = socket_path.clone();
            async move { 
                let stream = UnixStream::connect(path).await?;
                Ok::<_, std::io::Error>(TokioIo::new(stream))
            }
        }))
        .await?;

    let mut client = KernelServiceClient::new(channel);

    match cli.command {
        Commands::Run { name, args } => {
            let mut arg_map = std::collections::HashMap::new();
            for arg in args {
                if let Some((k, v)) = arg.split_once('=') {
                    arg_map.insert(k.to_string(), v.to_string());
                }
            }

            let request = tonic::Request::new(CommandRequest {
                command_id: uuid::Uuid::new_v4().to_string(),
                name,
                args: arg_map,
                identity: "admin".to_string(), // Placeholder for real identity
            });

            let response = client.execute(request).await?.into_inner();
            if response.success {
                println!("{}", response.output);
            } else {
                eprintln!("Error: {}", response.error);
            }
        }
        Commands::Status => {
            let request = tonic::Request::new(koad_proto::kernel::Empty {});
            let _ = client.heartbeat(request).await?;
            println!("Kernel is online and responsive.");
        }
        Commands::Dood { name, args } => {
            let mut arg_map = std::collections::HashMap::new();
            for arg in args {
                if let Some((k, v)) = arg.split_once('=') {
                    arg_map.insert(k.to_string(), v.to_string());
                }
            }

            let request = tonic::Request::new(CommandRequest {
                command_id: uuid::Uuid::new_v4().to_string(),
                name,
                args: arg_map,
                identity: "admiral".to_string(),
            });

            let response = client.execute(request).await?.into_inner();
            if response.success {
                println!("{}", response.output);
            } else {
                eprintln!("Error: {}", response.error);
            }
        }
    }

    Ok(())
}

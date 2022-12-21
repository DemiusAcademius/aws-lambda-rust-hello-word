use aws_config::meta::region::RegionProviderChain;
use http::Response;
use lambda_http::{http::StatusCode, run, service_fn, Error, IntoResponse, Request, RequestExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_max_level(tracing_subscriber::filter::LevelFilter::INFO)
        .init();

    let Opt { region, verbose } = Opt::from_args();

    let region_provider = RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-east-1"));
    println!();

    if verbose {
        println!("Cognito client version: {}", PKG_VERSION);
        println!(
            "Region:                 {}",
            region_provider.region().await.unwrap().as_ref()
        );

        println!();
    }

    let shared_config = aws_config::from_env().region(region_provider).load().await;

    run(service_fn(function_handler)).await
}

pub async fn function_handler(event: Request) -> Result<impl IntoResponse, Error> {
    let body = event.payload::<MyPayload>()?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(
            json!({
              "message": "Hello World",
              "payload": body,
              "payload-exists": body.is_some()
            })
            .to_string(),
        )
        .map_err(Box::new)?;

    Ok(response)
}


#[derive(Debug, StructOpt)]
struct Opt {
    /// The AWS Region.
    #[structopt(short, long)]
    region: Option<String>,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MyPayload {
    pub prop1: String,
    pub prop2: String,
}

use aws_sdk_cognitoidentityprovider::{Client, Region, PKG_VERSION};
use aws_smithy_types_convert::date_time::DateTimeExt;

// Lists your user pools.
// snippet-start:[cognitoidentityprovider.rust.list-user-pools]
async fn show_pools(client: &Client) -> Result<(), Error> {
    let response = client.list_user_pools().max_results(10).send().await?;
    if let Some(pools) = response.user_pools() {
        println!("User pools:");
        for pool in pools {
            println!("  ID:              {}", pool.id().unwrap_or_default());
            println!("  Name:            {}", pool.name().unwrap_or_default());
            println!("  Status:          {:?}", pool.status());
            println!("  Lambda Config:   {:?}", pool.lambda_config().unwrap());
            println!(
                "  Last modified:   {}",
                pool.last_modified_date().unwrap().to_chrono_utc()?
            );
            println!(
                "  Creation date:   {:?}",
                pool.creation_date().unwrap().to_chrono_utc()
            );
            println!();
        }
    }
    println!("Next token: {}", response.next_token().unwrap_or_default());

    Ok(())
}
// snippet-end:[cognitoidentityprovider.rust.list-user-pools]

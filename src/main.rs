use std::collections::HashMap;

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_cognitoidentityprovider::{model::AuthFlowType, Client, Region, PKG_VERSION};
use aws_smithy_types_convert::date_time::DateTimeExt;

use lambda_http::{http::StatusCode, run, service_fn, Error, IntoResponse, Request, RequestExt};

use http::Response;
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

    // println!("verbose is {}", verbose);
    println!("Cognito client version: {}", PKG_VERSION);
    println!(
        "Region:                 {}",
        region_provider.region().await.unwrap().as_ref()
    );
    println!();
    /*
    if verbose {
        println!("Cognito client version: {}", PKG_VERSION);
        println!(
            "Region:                 {}",
            region_provider.region().await.unwrap().as_ref()
        );

        println!();
    }
    */

    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);

    let pool_id = show_pools(&client).await?;

    run(service_fn(|event: Request| {
        function_handler(event, pool_id.clone(), &client)
    }))
    .await
}

pub async fn function_handler(
    event: Request,
    pool_id: Option<String>,
    client: &Client,
) -> Result<impl IntoResponse, Error> {
    if pool_id.is_none() {
        let response = Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("UsersPoolNotDOunf".to_string())
            .map_err(Box::new)?;
        return Ok(response);
    }

    let method = event.method();
    let body = event.payload::<AuthenticationPayload>()?;

    let Some(payload) = body else {
        let response = Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("PayloadDoesntExist".to_string())
            .map_err(Box::new)?;
        return Ok(response);
    };

    if method == http::Method::GET {
        let user = client
            .admin_get_user()
            .set_user_pool_id(pool_id.clone())
            .set_username(Some(payload.username))
            .send()
            .await?;

        let status = user.user_status();

        let response = Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(
                json!({
                  "message": "Hello World",
                  "pool-id": pool_id,
                  "user-enabled": user.enabled(),
                  "user-status": format!("{:?}", status)
                })
                .to_string(),
            )
            .map_err(Box::new)?;

        return Ok(response);
    }

    if method == http::Method::POST {
        let Some(password) = payload.password else {
            let response = Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("NotFoundPasswordForAUth".to_string())
                .map_err(Box::new)?;
            return Ok(response);
        };
        // authenticate
        let auth_params = HashMap::from([
            ("USERNAME".to_string(), payload.username),
            ("PASSWORD".to_string(), password),
        ]);
        // USERNAME, PASSWORD
        let auth_initiate = client
            .admin_initiate_auth()
            .set_user_pool_id(pool_id.clone())
            .set_auth_flow(Some(AuthFlowType::UserPasswordAuth))
            .set_auth_parameters(Some(auth_params))
            .send()
            .await?;

        let response = Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(
                json!({
                  "auth-result": format!("{:?}", auth_initiate.authentication_result()),
                  "challenge-name": format!("{:?}", auth_initiate.challenge_name()),
                  "session": auth_initiate.session().to_owned()
                })
                .to_string(),
            )
            .map_err(Box::new)?;

        return Ok(response);
    }

    let response = Response::builder()
        .status(StatusCode::METHOD_NOT_ALLOWED)
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
pub struct AuthenticationPayload {
    pub username: String,
    pub password: Option<String>,
}

// Lists your user pools.
// snippet-start:[cognitoidentityprovider.rust.list-user-pools]
async fn show_pools(client: &Client) -> Result<Option<String>, Error> {
    let mut pool_id: Option<String> = None;
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
            pool_id = pool.id().map(|it| it.to_string());
        }
    } else {
        println!("User pools not exists");
    }
    println!("Next token: {}", response.next_token().unwrap_or_default());

    Ok(pool_id)
}
// snippet-end:[cognitoidentityprovider.rust.list-user-pools]

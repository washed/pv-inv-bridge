use axum::headers;
use axum::{
    debug_handler,
    extract::State,
    response::sse::{Event, Sse},
    routing::get,
    Router, TypedHeader,
};
use edgedb_derive::Queryable;
use edgedb_protocol::model::Uuid;
use futures::stream::Stream;
use std::env;
use std::{convert::Infallible, time::Duration};
use tokio::sync::broadcast;
use tokio::task::JoinSet;
use tokio::time;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt as _;

mod inverter;
use inverter::{PVInverter, PVInverterData};

#[derive(Debug, Queryable)]
pub struct QueryableInsertResponse {
    pub id: Uuid,
}

#[derive(Clone)]
pub struct AppState {
    pub modbus_broadcast_tx: broadcast::Sender<PVInverterData>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (modbus_broadcast_tx, _modbus_broadcast_rx) = broadcast::channel::<PVInverterData>(16);

    let state = AppState {
        modbus_broadcast_tx: modbus_broadcast_tx,
    };

    let mut join_set = JoinSet::new();

    start_insert_inverter_task(&mut join_set, state.modbus_broadcast_tx.subscribe());
    start_get_inverter_task(&mut join_set, state.modbus_broadcast_tx.clone());

    start_server(state).await?;

    while let Some(res) = join_set.join_next().await {
        println!("Task finished unexpectedly!");
    }

    Ok(())
}

async fn start_server(state: AppState) -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/stream", get(sse_handler))
        .with_state(state);

    let bind_address: std::net::SocketAddr = env::var("BIND_ADDRESS")?.parse()?;
    axum::Server::bind(&bind_address)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[debug_handler]
async fn sse_handler(
    State(state): State<AppState>,
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    println!("`{}` connected", user_agent.as_str());
    let modbus_broadcast_sse_rx = state.modbus_broadcast_tx.subscribe();
    let stream = BroadcastStream::new(modbus_broadcast_sse_rx)
        .chunks_timeout(1, Duration::from_secs(10))
        .map(|x| {
            let mut battery_charge_power_sum: f64 = 0.0;

            for datum in x.iter() {
                let foo = datum.as_ref().unwrap();
                battery_charge_power_sum += f64::from(foo.battery_charge_power);
            }

            let mut res = x.last().unwrap().clone().unwrap();
            res.battery_charge_power = (battery_charge_power_sum / x.len() as f64).round() as i16;
            res
        })
        .filter_map(|x| match { Event::default().json_data(x) } {
            Ok(foo) => Some(foo),
            Err(_) => None,
        })
        .map(Ok);

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

fn start_insert_inverter_task(
    join_set: &mut JoinSet<()>,
    mut modbus_broadcast_rx: broadcast::Receiver<PVInverterData>,
) {
    join_set.spawn(async move {
        let db_conn = match db_connect().await {
            Ok(db_conn) => db_conn,
            Err(e) => {
                eprintln!("Error connecting to database: {e}");
                return;
            }
        };

        {
            loop {
                match modbus_broadcast_rx.recv().await {
                    Ok(data) => match insert(&db_conn, data).await {
                        Ok(res) => println!("Inserted object {res}"),
                        Err(e) => eprintln!("Error inserting object into db: {e}"),
                    },
                    Err(e) => eprintln!("Error receiving modbus data from stream! {e}"),
                }
            }
        }
    });
}

fn start_get_inverter_task(
    join_set: &mut JoinSet<()>,
    modbus_broadcast_tx: broadcast::Sender<PVInverterData>,
) {
    join_set.spawn(async move {
        let mut pv_inverter = match PVInverter::new().await {
            Ok(pv_inverter) => pv_inverter,
            Err(e) => {
                eprintln!("Error instantiating PVInverter! {e}");
                return;
            }
        };

        let interval = match env::var("INTERVAL_S") {
            Ok(res) => match res.parse::<u64>() {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("Error parsing INTERVAL_S: {e}");
                    return;
                }
            },
            Err(e) => {
                println!("Error getting INTERVAL_S from env var: {e}");
                return;
            }
        };
        let mut interval = time::interval(time::Duration::from_secs(interval));

        loop {
            interval.tick().await;
            let data: PVInverterData = match pv_inverter.get_inverter_data().await {
                Ok(data) => {
                    println!("modbus rx");
                    data
                }
                Err(e) => {
                    eprintln!("Error getting modbus data: {e}");
                    return;
                }
            };

            match modbus_broadcast_tx.send(data) {
                Ok(_res) => println!("sent modbus data into stream"),
                Err(e) => eprintln!("error sending data into stream: {e}"),
            }
        }
    });
}

async fn insert(
    db_conn: &edgedb_tokio::Client,
    data: PVInverterData,
) -> Result<i32, Box<dyn std::error::Error>> {
    let query = format!("INSERT PVInverter {{
        timestamp := <datetime>\"{}\",
        device := (insert Device {{ device_id := <str>\"{}\" }} unless conflict on .device_id else (select Device)),
        grid_voltage := <float32>{},
        grid_current := <float32>{},
        grid_power := <float32>{},
        grid_frequency := <float32>{},
        pv_power_1 := <int32>{},
        pv_power_2 := <int32>{},
        feedin_power := <int32>{},
        battery_charge_power := <int16>{},
        battery_soc := <int16>{},
        radiator_temperature := <int16>{},
        battery_temperature := <int16>{}
    }}",
    data.timestamp.0.to_rfc3339(), data.device_id, data.grid_voltage, data.grid_current,
    data.grid_power, data.grid_frequency, data.pv_power_1, data.pv_power_2,
    data.feedin_power, data.battery_charge_power, data.battery_soc,
    data.radiator_temperature, data.battery_temperature);
    db_conn.execute(&query, &()).await?;
    println!("Inserted {:#?}", data);
    Ok(0)
}

async fn db_connect() -> Result<edgedb_tokio::Client, Box<dyn std::error::Error>> {
    let edgedb_dsn = env::var("EDGEDB_DSN")?;
    let mut builder = edgedb_tokio::Builder::new();
    builder.dsn(edgedb_dsn.as_str())?;
    let config = builder.build_env().await?;
    let conn = edgedb_tokio::Client::new(&config);
    conn.ensure_connected().await?;
    Ok(conn)
}

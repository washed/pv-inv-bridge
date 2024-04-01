use axum::headers;
use axum::{
    debug_handler,
    extract::State,
    response::sse::{Event, Sse},
    routing::get,
    Router, TypedHeader,
};

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

mod db;
use db::DBInserter;

mod heatpump;
use heatpump::Heatpump;

#[derive(Clone)]
pub struct AppState {
    pub modbus_broadcast_tx: broadcast::Sender<PVInverterData>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (modbus_broadcast_tx, _modbus_broadcast_rx) = broadcast::channel::<PVInverterData>(16);

    let state = AppState {
        modbus_broadcast_tx,
    };

    let mut join_set = JoinSet::new();

    start_insert_inverter_task(&mut join_set, state.modbus_broadcast_tx.subscribe());
    start_send_pv_data_heatpump_task(&mut join_set, state.modbus_broadcast_tx.subscribe());
    start_get_inverter_task(&mut join_set, state.modbus_broadcast_tx.clone());

    start_server(state).await?;

    while let Some(_res) = join_set.join_next().await {
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
        .filter_map(|x| match Event::default().json_data(x) {
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
        let db_inserter = match DBInserter::new_retrying().await {
            Ok(db_inserter) => db_inserter,
            Err(e) => {
                eprintln!("Error connecting to database: {e}");
                return;
            }
        };

        loop {
            match modbus_broadcast_rx.recv().await {
                Ok(data) => match db_inserter.insert_pv_inverter_data(data).await {
                    Ok(res) => println!("Inserted object {res}"),
                    Err(e) => eprintln!("Error inserting object into db: {e}"),
                },
                Err(e) => eprintln!("Error receiving modbus data from stream! {e}"),
            }
        }
    });
}

fn start_get_inverter_task(
    join_set: &mut JoinSet<()>,
    modbus_broadcast_tx: broadcast::Sender<PVInverterData>,
) {
    join_set.spawn(async move {
        println!("start_get_inverter_task");
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

fn start_send_pv_data_heatpump_task(
    join_set: &mut JoinSet<()>,
    mut modbus_broadcast_rx: broadcast::Receiver<PVInverterData>,
) {
    join_set.spawn(async move {
        println!("start_get_pv_surplus_task");
        let mut heatpump = match Heatpump::new().await {
            Ok(heatpump) => heatpump,
            Err(e) => {
                eprintln!("Error instantiating Heatpump! {e}");
                return;
            }
        };

        loop {
            match modbus_broadcast_rx.recv().await {
                Ok(data) => {
                    let pv_power: f32 =
                        ((data.pv_power_1 + data.pv_power_2) as f64 / 1000.0) as f32;
                    let pv_surplus = (data.feedin_power as f32).clamp(0.0, f32::MAX) / 1000.0;
                    let battery_charge_power =
                        (data.battery_charge_power as f32 / 1000.0).clamp(0.0, f32::MAX);
                    let pv_surplus_incl_battery = pv_surplus + battery_charge_power;
                    //dbg!(pv_power);
                    //dbg!(pv_surplus);
                    //dbg!(batter_charge_power);
                    dbg!(pv_surplus_incl_battery);

                    heatpump
                        .set_pv_surplus(pv_surplus_incl_battery)
                        .await
                        .unwrap();
                    heatpump.set_pv_power(pv_power).await.unwrap();
                }
                Err(e) => eprintln!("Error receiving modbus data from stream! {e}"),
            }
        }
    });
}

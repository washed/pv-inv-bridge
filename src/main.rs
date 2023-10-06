use chrono::prelude::*;
use edgedb_derive::Queryable;
use edgedb_protocol::model::Uuid;
use std::env;
use tokio::sync::broadcast;
use tokio::task;
use tokio::task::JoinSet;
use tokio::time;
use tokio_modbus::prelude::*;

#[derive(Clone, Debug)]
struct PVInverterData {
    timestamp: DateTime<Utc>,
    device_id: String,
    grid_voltage: f32,
    grid_current: f32,
    grid_power: i16,
    grid_frequency: f32,
    pv_power_1: i32,
    pv_power_2: i32,
    feedin_power: i32,
    battery_charge_power: i16,
    battery_soc: i16,
    radiator_temperature: i16,
    battery_temperature: i16,
}

#[derive(Debug, Queryable)]
pub struct QueryableInsertResponse {
    pub id: Uuid,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut join_set = JoinSet::new();

    let (modbus_broadcast_tx, modbus_broadcast_rx_inserter) =
        broadcast::channel::<PVInverterData>(16);

    start_insert_inverter_tak(&mut join_set, modbus_broadcast_rx_inserter);
    start_get_inverter_task(&mut join_set, modbus_broadcast_tx);

    while let Some(res) = join_set.join_next().await {
        println!("Task finished unexpectedly!");
    }

    Ok(())
}

fn start_insert_inverter_tak(
    join_set: &mut JoinSet<()>,
    mut modbus_broadcast_rx: broadcast::Receiver<PVInverterData>,
) {
    join_set.spawn(async move {
        let db_conn = db_connect().await.unwrap();

        loop {
            let data = modbus_broadcast_rx.recv().await.unwrap();
            insert(&db_conn, data).await.unwrap();
        }
    });
}

fn start_get_inverter_task(
    join_set: &mut JoinSet<()>,
    modbus_broadcast_tx: broadcast::Sender<PVInverterData>,
) {
    join_set.spawn(async move {
        let mut modbus_ctx = modbus_connect().await.unwrap();

        let interval = env::var("INTERVAL_S").unwrap().parse().unwrap();
        let mut interval = time::interval(time::Duration::from_secs(interval));

        loop {
            interval.tick().await;
            let data: PVInverterData = get_modbus_stuff(&mut modbus_ctx).await.unwrap();
            println!("modbus rx");
            modbus_broadcast_tx.send(data).unwrap();
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
    data.timestamp.to_rfc3339(), data.device_id, data.grid_voltage, data.grid_current,
    data.grid_power, data.grid_frequency, data.pv_power_1, data.pv_power_2,
    data.feedin_power, data.battery_charge_power, data.battery_soc,
    data.radiator_temperature, data.battery_temperature);
    db_conn.execute(&query, &()).await?;
    println!("Inserted {:#?}", data);
    Ok(0)
}

async fn modbus_connect() -> Result<client::Context, Box<dyn std::error::Error>> {
    let addr = env::var("PV_INV_MODBUS_TCP_ADDRESS")?;
    let socket_addr = addr.parse()?;
    let modbus_ctx: client::Context = tcp::connect_slave(socket_addr, Slave(1)).await?;
    Ok(modbus_ctx)
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

async fn get_modbus_stuff(
    modbus_ctx: &mut client::Context,
) -> Result<PVInverterData, std::io::Error> {
    Ok(PVInverterData {
        timestamp: Utc::now(),
        device_id: get_serial_number(modbus_ctx).await?,
        grid_voltage: get_grid_voltage(modbus_ctx).await?,
        grid_current: get_grid_current(modbus_ctx).await?,
        grid_power: get_grid_power(modbus_ctx).await?,
        grid_frequency: get_grid_frequency(modbus_ctx).await?,
        pv_power_1: get_pv_power_1(modbus_ctx).await?.into(),
        pv_power_2: get_pv_power_2(modbus_ctx).await?.into(),
        feedin_power: get_feedin_power(modbus_ctx).await?,
        battery_charge_power: get_battery_charge_power(modbus_ctx).await?,
        battery_soc: get_battery_soc(modbus_ctx).await?,
        radiator_temperature: get_radiator_temperature(modbus_ctx).await?,
        battery_temperature: get_battery_temperature(modbus_ctx).await?,
    })
}

async fn get_battery_charge_voltage(
    modbus_ctx: &mut client::Context,
) -> Result<f32, std::io::Error> {
    let value = modbus_ctx.read_input_registers(0x0014, 1).await?[0] as i16;
    let value = f32::from(value);
    Ok(value / 10.0)
}

async fn get_battery_charge_current(
    modbus_ctx: &mut client::Context,
) -> Result<f32, std::io::Error> {
    let value = modbus_ctx.read_input_registers(0x0015, 1).await?[0] as i16;
    let value = f32::from(value);
    Ok(value / 10.0)
}

async fn get_battery_charge_power(modbus_ctx: &mut client::Context) -> Result<i16, std::io::Error> {
    Ok(modbus_ctx.read_input_registers(0x0016, 1).await?[0] as i16)
}

async fn get_battery_soc(modbus_ctx: &mut client::Context) -> Result<i16, std::io::Error> {
    Ok(modbus_ctx.read_input_registers(0x001c, 1).await?[0] as i16)
}

async fn get_battery_discharge_max_current(
    modbus_ctx: &mut client::Context,
) -> Result<i16, std::io::Error> {
    Ok(modbus_ctx.read_input_registers(0x0025, 1).await?[0] as i16)
}

async fn get_battery_power_all(modbus_ctx: &mut client::Context) -> Result<i32, std::io::Error> {
    let result = modbus_ctx.read_input_registers(0x01FA, 2).await?;
    Ok(((result[1] as i32) << 16) | result[0] as i32)
}

async fn get_battery_charge_discharge_power(
    modbus_ctx: &mut client::Context,
) -> Result<i32, std::io::Error> {
    let result = modbus_ctx.read_input_registers(0x0114, 2).await?;
    Ok(((result[1] as i32) << 16) | result[0] as i32)
}

async fn get_feedin_power(modbus_ctx: &mut client::Context) -> Result<i32, std::io::Error> {
    let result = modbus_ctx.read_input_registers(0x0046, 2).await?;
    Ok(((result[1] as i32) << 16) | result[0] as i32)
}

async fn get_battery_temperature(modbus_ctx: &mut client::Context) -> Result<i16, std::io::Error> {
    Ok(modbus_ctx.read_input_registers(0x0018, 1).await?[0] as i16)
}

async fn get_user_soc(modbus_ctx: &mut client::Context) -> Result<u16, std::io::Error> {
    Ok(modbus_ctx.read_input_registers(0x00BE, 1).await?[0])
}

async fn get_radiator_temperature(modbus_ctx: &mut client::Context) -> Result<i16, std::io::Error> {
    Ok(modbus_ctx.read_input_registers(0x0008, 1).await?[0] as i16)
}

async fn get_pv_power_1(modbus_ctx: &mut client::Context) -> Result<u16, std::io::Error> {
    Ok(modbus_ctx.read_input_registers(0x000A, 1).await?[0])
}

async fn get_pv_power_2(modbus_ctx: &mut client::Context) -> Result<u16, std::io::Error> {
    Ok(modbus_ctx.read_input_registers(0x000B, 1).await?[0])
}

async fn get_pv_voltage_1(modbus_ctx: &mut client::Context) -> Result<f32, std::io::Error> {
    let value = modbus_ctx.read_input_registers(0x0003, 1).await?[0];
    let value = f32::from(value);
    Ok(value / 10.0)
}

async fn get_pv_voltage_2(modbus_ctx: &mut client::Context) -> Result<f32, std::io::Error> {
    let value = modbus_ctx.read_input_registers(0x0004, 1).await?[0];
    let value = f32::from(value);
    Ok(value / 10.0)
}

async fn get_pv_current_1(modbus_ctx: &mut client::Context) -> Result<f32, std::io::Error> {
    let value = modbus_ctx.read_input_registers(0x0005, 1).await?[0];
    let value = f32::from(value);
    Ok(value / 10.0)
}

async fn get_pv_current_2(modbus_ctx: &mut client::Context) -> Result<f32, std::io::Error> {
    let value = modbus_ctx.read_input_registers(0x0006, 1).await?[0];
    let value = f32::from(value);
    Ok(value / 10.0)
}

async fn get_grid_current(modbus_ctx: &mut client::Context) -> Result<f32, std::io::Error> {
    let value = modbus_ctx.read_input_registers(0x0001, 1).await?[0];
    let value = f32::from(value as i16);
    Ok(value / 10.0)
}

async fn get_grid_power(modbus_ctx: &mut client::Context) -> Result<i16, std::io::Error> {
    Ok(modbus_ctx.read_input_registers(0x0002, 1).await?[0] as i16)
}

async fn get_grid_frequency(modbus_ctx: &mut client::Context) -> Result<f32, std::io::Error> {
    let value = modbus_ctx.read_input_registers(0x0007, 1).await?[0];
    let value = f32::from(value);
    Ok(value / 100.0)
}

async fn get_grid_voltage(modbus_ctx: &mut client::Context) -> Result<f32, std::io::Error> {
    let value = modbus_ctx.read_input_registers(0x0000, 1).await?[0];
    let value = f32::from(value);
    Ok(value / 10.0)
}

async fn get_machine_type(modbus_ctx: &mut client::Context) -> Result<u16, std::io::Error> {
    Ok(modbus_ctx.read_holding_registers(0x0105, 1).await?[0])
}

async fn get_serial_number(modbus_ctx: &mut client::Context) -> Result<String, std::io::Error> {
    let result = modbus_ctx.read_holding_registers(0x0, 7).await?;
    Ok(modbus_to_str(&result))
}

fn modbus_to_str(data: &Vec<u16>) -> String {
    let mut chars = String::new();

    for word in data {
        chars.push(char::try_from((word >> 8) as u8).expect("boing"));
        chars.push(char::try_from((word & 0x00FF) as u8).expect("boing"));
    }

    chars
}

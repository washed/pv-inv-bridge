use std::thread::sleep;
use std::time::{Duration, Instant};
use tokio_modbus::prelude::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket_addr = "192.168.178.202:502".parse().unwrap();

    let mut ctx: client::Context = tcp::connect_slave(socket_addr, Slave(1)).await?;

    println!("power");
    let data = ctx.read_input_registers(0x0016, 1).await?;

    let power = data[0] as i16;
    println!("power: '{power}'");

    let interval = Duration::from_secs(1);
    let mut next_time = Instant::now() + interval;

    loop {
        get_modbus_stuff(&mut ctx).await;
        sleep(next_time - Instant::now());
        next_time += interval;
    }
}

async fn get_modbus_stuff(ctx: &mut client::Context) {
    let sn = get_serial_number(ctx).await;
    println!("Inverter SN: {sn}");

    let machine_type = get_machine_type(ctx).await;
    println!("Machine Type: {machine_type}");

    let grid_voltage = get_grid_voltage(ctx).await;
    println!("Grid voltage: {grid_voltage}");

    let grid_current = get_grid_current(ctx).await;
    println!("Grid current: {grid_current}");

    let grid_power = get_grid_power(ctx).await;
    println!("Grid power: {grid_power}");

    let grid_frequency = get_grid_frequency(ctx).await;
    println!("Grid frequency: {grid_frequency}");

    let radiator_temperature = get_radiator_temperature(ctx).await;
    println!("Radiator Temperature: {radiator_temperature}");

    let feedin_power = get_feedin_power(ctx).await;
    println!("feedin_power: {feedin_power}");

    let battery_charge_power = get_battery_charge_power(ctx).await;
    println!("battery_charge_power: {battery_charge_power}");

    let battery_temperature = get_battery_temperature(ctx).await;
    println!("battery_temperature: {battery_temperature}");

    let battery_soc = get_battery_soc(ctx).await;
    println!("battery_soc: {battery_soc}");

    let battery_charge_voltage = get_battery_charge_voltage(ctx).await;
    println!("battery_charge_voltage: {battery_charge_voltage}");

    let battery_charge_current = get_battery_charge_current(ctx).await;
    println!("battery_charge_current: {battery_charge_current}");

    let battery_charge_power = get_battery_charge_power(ctx).await;
    println!("battery_charge_power: {battery_charge_power}");
    println!();
}

async fn get_battery_charge_voltage(ctx: &mut client::Context) -> f32 {
    let value = ctx.read_input_registers(0x0014, 1).await.expect("nope")[0] as i16;
    let value = f32::from(value);
    value / 10.0
}

async fn get_battery_charge_current(ctx: &mut client::Context) -> f32 {
    let value = ctx.read_input_registers(0x0015, 1).await.expect("nope")[0] as i16;
    let value = f32::from(value);
    value / 10.0
}

async fn get_battery_charge_power(ctx: &mut client::Context) -> i16 {
    ctx.read_input_registers(0x0016, 1).await.expect("nope")[0] as i16
}

async fn get_battery_soc(ctx: &mut client::Context) -> u16 {
    ctx.read_input_registers(0x001c, 1).await.expect("nope")[0]
}

async fn get_battery_discharge_max_current(ctx: &mut client::Context) -> i16 {
    ctx.read_input_registers(0x0025, 1).await.expect("nope")[0] as i16
}

async fn get_battery_power_all(ctx: &mut client::Context) -> i32 {
    let result = ctx.read_input_registers(0x01FA, 2).await.expect("nope");
    ((result[1] as i32) << 16) | result[0] as i32
}

async fn get_battery_charge_discharge_power(ctx: &mut client::Context) -> i32 {
    let result = ctx.read_input_registers(0x0114, 2).await.expect("nope");
    ((result[1] as i32) << 16) | result[0] as i32
}

async fn get_feedin_power(ctx: &mut client::Context) -> i32 {
    let result = ctx.read_input_registers(0x0046, 2).await.expect("nope");
    ((result[1] as i32) << 16) | result[0] as i32
}

async fn get_battery_temperature(ctx: &mut client::Context) -> i16 {
    ctx.read_input_registers(0x0018, 1).await.expect("nope")[0] as i16
}

async fn get_user_soc(ctx: &mut client::Context) -> u16 {
    ctx.read_input_registers(0x00BE, 1).await.expect("nope")[0]
}

async fn get_radiator_temperature(ctx: &mut client::Context) -> i16 {
    ctx.read_input_registers(0x0008, 1).await.expect("nope")[0] as i16
}

async fn get_dc_power_1(ctx: &mut client::Context) -> u16 {
    ctx.read_input_registers(0x000A, 1).await.expect("nope")[0]
}

async fn get_dc_power_2(ctx: &mut client::Context) -> u16 {
    ctx.read_input_registers(0x000B, 1).await.expect("nope")[0]
}

async fn get_pv_voltage_1(ctx: &mut client::Context) -> f32 {
    let value = ctx.read_input_registers(0x0003, 1).await.expect("nope")[0];
    let value = f32::from(value);
    value / 10.0
}

async fn get_pv_voltage_2(ctx: &mut client::Context) -> f32 {
    let value = ctx.read_input_registers(0x0004, 1).await.expect("nope")[0];
    let value = f32::from(value);
    value / 10.0
}

async fn get_pv_current_1(ctx: &mut client::Context) -> f32 {
    let value = ctx.read_input_registers(0x0005, 1).await.expect("nope")[0];
    let value = f32::from(value);
    value / 10.0
}

async fn get_pv_current_2(ctx: &mut client::Context) -> f32 {
    let value = ctx.read_input_registers(0x0006, 1).await.expect("nope")[0];
    let value = f32::from(value);
    value / 10.0
}

async fn get_grid_current(ctx: &mut client::Context) -> f32 {
    let value = ctx.read_input_registers(0x0001, 1).await.expect("nope")[0];
    let value = f32::from(value as i16);
    value / 10.0
}

async fn get_grid_power(ctx: &mut client::Context) -> i16 {
    ctx.read_input_registers(0x0002, 1).await.expect("nope")[0] as i16
}

async fn get_grid_frequency(ctx: &mut client::Context) -> f32 {
    let value = ctx.read_input_registers(0x0007, 1).await.expect("nope")[0];
    let value = f32::from(value);
    value / 100.0
}

async fn get_grid_voltage(ctx: &mut client::Context) -> f32 {
    let value = ctx.read_input_registers(0x0000, 1).await.expect("nope")[0];
    let value = f32::from(value);
    value / 10.0
}

async fn get_machine_type(ctx: &mut client::Context) -> u16 {
    ctx.read_holding_registers(0x0105, 1).await.expect("nope")[0]
}

async fn get_serial_number(ctx: &mut client::Context) -> String {
    let result = ctx.read_holding_registers(0x0, 7).await.expect("nope");
    modbus_to_str(&result)
}

fn modbus_to_str(data: &Vec<u16>) -> String {
    let mut chars = String::new();

    for word in data {
        chars.push(char::try_from((word >> 8) as u8).expect("boing"));
        chars.push(char::try_from((word & 0x00FF) as u8).expect("boing"));
    }

    chars
}

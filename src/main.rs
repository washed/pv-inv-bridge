use modbus::tcp;
use modbus::Client;
use modbus::Transport;

use std::thread::sleep;
use std::time::{Duration, Instant};

fn main() {
    let mut cfg = tcp::Config::default();
    let mut client = tcp::Transport::new_with_cfg("192.168.178.202", cfg).unwrap();

    let interval = Duration::from_secs(1);
    let mut next_time = Instant::now() + interval;

    loop {
        get_modbus_stuff(&mut client);
        sleep(next_time - Instant::now());
        next_time += interval;
    }
}

fn get_modbus_stuff(client: &mut Transport) {
    let sn = get_serial_number(client);
    println!("Inverter SN: {sn}");

    let machine_type = get_machine_type(client);
    println!("Machine Type: {machine_type}");

    let grid_voltage = get_grid_voltage(client);
    println!("Grid voltage: {grid_voltage}");

    let grid_current = get_grid_current(client);
    println!("Grid current: {grid_current}");

    let grid_power = get_grid_power(client);
    println!("Grid power: {grid_power}");

    let grid_frequency = get_grid_frequency(client);
    println!("Grid frequency: {grid_frequency}");

    let radiator_temperature = get_radiator_temperature(client);
    println!("Radiator Temperature: {radiator_temperature}");

    let feedin_power = get_feedin_power(client);
    println!("feedin_power: {feedin_power}");

    let battery_charge_power = get_battery_charge_power(client);
    println!("battery_charge_power: {battery_charge_power}");

    let battery_temperature = get_battery_temperature(client);
    println!("battery_temperature: {battery_temperature}");

    let battery_soc = get_battery_soc(client);
    println!("battery_soc: {battery_soc}");

    let battery_charge_voltage = get_battery_charge_voltage(client);
    println!("battery_charge_voltage: {battery_charge_voltage}");

    let battery_charge_current = get_battery_charge_current(client);
    println!("battery_charge_current: {battery_charge_current}");

    let battery_charge_power = get_battery_charge_power(client);
    println!("battery_charge_power: {battery_charge_power}");
    println!();
}

fn get_battery_charge_voltage(client: &mut Transport) -> f32 {
    let value = client.read_input_registers(0x0014, 1).expect("oh no")[0] as i16;
    let value = f32::from(value);
    value / 10.0
}

fn get_battery_charge_current(client: &mut Transport) -> f32 {
    let value = client.read_input_registers(0x0015, 1).expect("oh no")[0] as i16;
    let value = f32::from(value);
    value / 10.0
}

fn get_battery_charge_power(client: &mut Transport) -> i16 {
    client.read_input_registers(0x0016, 1).expect("oh no")[0] as i16
}

fn get_battery_soc(client: &mut Transport) -> u16 {
    client.read_input_registers(0x001c, 1).expect("oh no")[0]
}

fn get_battery_discharge_max_current(client: &mut Transport) -> i16 {
    client.read_input_registers(0x0025, 1).expect("oh no")[0] as i16
}

fn get_battery_power_all(client: &mut Transport) -> i32 {
    let result = client.read_input_registers(0x01FA, 2).expect("oh no");
    ((result[1] as i32) << 16) | result[0] as i32
}

fn get_battery_charge_discharge_power(client: &mut Transport) -> i32 {
    let result = client.read_input_registers(0x0114, 2).expect("oh no");
    ((result[1] as i32) << 16) | result[0] as i32
}

fn get_feedin_power(client: &mut Transport) -> i32 {
    let result = client.read_input_registers(0x0046, 2).expect("oh no");
    ((result[1] as i32) << 16) | result[0] as i32
}

fn get_battery_temperature(client: &mut Transport) -> i16 {
    client.read_input_registers(0x0018, 1).expect("oh no")[0] as i16
}

fn get_user_soc(client: &mut Transport) -> u16 {
    client.read_input_registers(0x00BE, 1).expect("oh no")[0]
}

fn get_radiator_temperature(client: &mut Transport) -> i16 {
    client.read_input_registers(0x0008, 1).expect("oh no")[0] as i16
}

fn get_dc_power_1(client: &mut Transport) -> u16 {
    client.read_input_registers(0x000A, 1).expect("oh no")[0]
}

fn get_dc_power_2(client: &mut Transport) -> u16 {
    client.read_input_registers(0x000B, 1).expect("oh no")[0]
}

fn get_pv_voltage_1(client: &mut Transport) -> f32 {
    let value = client.read_input_registers(0x0003, 1).expect("oh no")[0];
    let value = f32::from(value);
    value / 10.0
}

fn get_pv_voltage_2(client: &mut Transport) -> f32 {
    let value = client.read_input_registers(0x0004, 1).expect("oh no")[0];
    let value = f32::from(value);
    value / 10.0
}

fn get_pv_current_1(client: &mut Transport) -> f32 {
    let value = client.read_input_registers(0x0005, 1).expect("oh no")[0];
    let value = f32::from(value);
    value / 10.0
}

fn get_pv_current_2(client: &mut Transport) -> f32 {
    let value = client.read_input_registers(0x0006, 1).expect("oh no")[0];
    let value = f32::from(value);
    value / 10.0
}

fn get_grid_current(client: &mut Transport) -> f32 {
    let value = client.read_input_registers(0x0001, 1).expect("oh no")[0];
    let value = f32::from(value as i16);
    value / 10.0
}

fn get_grid_power(client: &mut Transport) -> i16 {
    client.read_input_registers(0x0002, 1).expect("oh no")[0] as i16
}

fn get_grid_frequency(client: &mut Transport) -> f32 {
    let value = client.read_input_registers(0x0007, 1).expect("oh no")[0];
    let value = f32::from(value);
    value / 100.0
}

fn get_grid_voltage(client: &mut Transport) -> f32 {
    let value = client.read_input_registers(0x0000, 1).expect("oh no")[0];
    let value = f32::from(value);
    value / 10.0
}

fn get_machine_type(client: &mut Transport) -> u16 {
    client.read_holding_registers(0x0105, 1).expect("oh no")[0]
}

fn get_serial_number(client: &mut Transport) -> String {
    let result = client.read_holding_registers(0x0, 7).expect("oh no");
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

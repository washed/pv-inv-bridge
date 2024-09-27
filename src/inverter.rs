use crate::robust_modbus::RobustContext;
use anyhow::{anyhow, Result};
use chrono::prelude::*;
use serde::Serialize;
use std::env;
use tokio_modbus::prelude::*;

#[derive(Clone, Serialize, Debug)]
pub struct PVInverterData {
    pub timestamp: DateTime<Utc>,
    pub device_id: String,
    pub grid_voltage: f32,
    pub grid_current: f32,
    pub grid_power: i16,
    pub grid_frequency: f32,
    pub pv_power_1: i32,
    pub pv_power_2: i32,
    pub feedin_power: i32,
    pub battery_charge_power: i16,
    pub battery_soc: i16,
    pub radiator_temperature: i16,
    pub battery_temperature: i16,
}

pub struct PVInverter {
    modbus_ctx: RobustContext,
}

impl PVInverter {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let host = env::var("PV_INV_MODBUS_TCP_ADDRESS")?;
        let modbus_ctx: RobustContext = RobustContext::new(host.as_str(), Slave(1)).await?;
        Ok(Self { modbus_ctx })
    }

    pub async fn get_inverter_data(&mut self) -> Result<()> {
        let grid_voltage = self.get_grid_voltage().await?;
        println!("{grid_voltage}");

        Ok(())
    }

    async fn get_battery_charge_voltage(&mut self) -> Result<f32> {
        let value = self.modbus_ctx.read_input_registers(0x0014, 1).await??[0] as i16;
        let value = f32::from(value);
        Ok(value / 10.0)
    }

    async fn get_battery_charge_current(&mut self) -> Result<f32> {
        let value = self.modbus_ctx.read_input_registers(0x0015, 1).await??[0] as i16;
        let value = f32::from(value);
        Ok(value / 10.0)
    }

    async fn get_battery_charge_power(&mut self) -> Result<i16> {
        Ok(self.modbus_ctx.read_input_registers(0x0016, 1).await??[0] as i16)
    }

    async fn get_battery_soc(&mut self) -> Result<i16> {
        Ok(self.modbus_ctx.read_input_registers(0x001c, 1).await??[0] as i16)
    }

    async fn get_battery_discharge_max_current(&mut self) -> Result<i16> {
        Ok(self.modbus_ctx.read_input_registers(0x0025, 1).await??[0] as i16)
    }

    async fn get_battery_power_all(&mut self) -> Result<i32> {
        let result = self.modbus_ctx.read_input_registers(0x01FA, 2).await??;
        Ok(((result[1] as i32) << 16) | result[0] as i32)
    }

    async fn get_battery_charge_discharge_power(&mut self) -> Result<i32> {
        let result = self.modbus_ctx.read_input_registers(0x0114, 2).await??;
        Ok(((result[1] as i32) << 16) | result[0] as i32)
    }

    async fn get_feedin_power(&mut self) -> Result<i32> {
        let result = self.modbus_ctx.read_input_registers(0x0046, 2).await??;
        Ok(((result[1] as i32) << 16) | result[0] as i32)
    }

    async fn get_battery_temperature(&mut self) -> Result<i16> {
        Ok(self.modbus_ctx.read_input_registers(0x0018, 1).await??[0] as i16)
    }

    async fn get_user_soc(&mut self) -> Result<u16> {
        Ok(self.modbus_ctx.read_input_registers(0x00BE, 1).await??[0])
    }

    async fn get_radiator_temperature(&mut self) -> Result<i16> {
        Ok(self.modbus_ctx.read_input_registers(0x0008, 1).await??[0] as i16)
    }

    async fn get_pv_power_1(&mut self) -> Result<u16> {
        Ok(self.modbus_ctx.read_input_registers(0x000A, 1).await??[0])
    }

    async fn get_pv_power_2(&mut self) -> Result<u16> {
        Ok(self.modbus_ctx.read_input_registers(0x000B, 1).await??[0])
    }

    async fn get_pv_voltage_1(&mut self) -> Result<f32> {
        let value = self.modbus_ctx.read_input_registers(0x0003, 1).await??[0];
        let value = f32::from(value);
        Ok(value / 10.0)
    }

    async fn get_pv_voltage_2(&mut self) -> Result<f32> {
        let value = self.modbus_ctx.read_input_registers(0x0004, 1).await??[0];
        let value = f32::from(value);
        Ok(value / 10.0)
    }

    async fn get_pv_current_1(&mut self) -> Result<f32> {
        let value = self.modbus_ctx.read_input_registers(0x0005, 1).await??[0];
        let value = f32::from(value);
        Ok(value / 10.0)
    }

    async fn get_pv_current_2(&mut self) -> Result<f32> {
        let value = self.modbus_ctx.read_input_registers(0x0006, 1).await??[0];
        let value = f32::from(value);
        Ok(value / 10.0)
    }

    async fn get_grid_current(&mut self) -> Result<f32> {
        let value = self.modbus_ctx.read_input_registers(0x0001, 1).await??[0];
        let value = f32::from(value as i16);
        Ok(value / 10.0)
    }

    async fn get_grid_power(&mut self) -> Result<i16> {
        Ok(self.modbus_ctx.read_input_registers(0x0002, 1).await??[0] as i16)
    }

    async fn get_grid_frequency(&mut self) -> Result<f32> {
        let value = self.modbus_ctx.read_input_registers(0x0007, 1).await??[0];
        let value = f32::from(value);
        Ok(value / 100.0)
    }

    async fn get_grid_voltage(&mut self) -> Result<f32> {
        let value = match self.modbus_ctx.read_input_registers(0x0000, 1).await {
            Ok(res) => match res {
                Ok(res_inner) => res_inner[0],
                Err(e) => {
                    eprintln!("error inner: {e}");
                    return Err(anyhow!("inner error"));
                }
            },
            Err(e) => {
                eprintln!("error outer: {e}");
                return Err(anyhow!("outer error"));
            }
        };
        let value = f32::from(value);
        Ok(value / 10.0)
    }

    async fn get_machine_type(&mut self) -> Result<u16> {
        Ok(self.modbus_ctx.read_holding_registers(0x0105, 1).await??[0])
    }

    /*
    async fn get_serial_number(&mut self) -> Result<String> {
        let result = self.modbus_ctx.read_holding_registers(0x0, 7).await??;
        Ok(PVInverter::modbus_to_str(&result))
    }
    */

    fn modbus_to_str(data: &Vec<u16>) -> String {
        let mut chars = String::new();

        for word in data {
            chars.push(char::from((word >> 8) as u8));
            chars.push(char::from((word & 0x00FF) as u8));
        }

        chars
    }
}

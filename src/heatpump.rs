use anyhow::anyhow;
use std::env;
use std::net::{SocketAddr, ToSocketAddrs};
use tokio_modbus::prelude::*;

pub struct Heatpump {
    modbus_ctx: client::Context,
}

impl Heatpump {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let modbus_tcp_address = env::var("HEATPUMP_MODBUS_TCP_ADDRESS")?
            .to_socket_addrs()
            .unwrap()
            .next()
            .unwrap();

        let modbus_ctx: client::Context = tcp::connect_slave(modbus_tcp_address, Slave(1)).await?;

        Ok(Self { modbus_ctx })
    }

    pub async fn get_heatpump_power(&mut self) -> anyhow::Result<f32> {
        let value = self.modbus_ctx.read_input_registers(4122, 2).await??[0];
        let value = f32::from(value);
        Ok(value)
    }

    pub async fn set_pv_surplus(&mut self, pv_surplus_power_watts: f32) -> anyhow::Result<()> {
        dbg!(pv_surplus_power_watts);
        let data = idm_float_to_u16(pv_surplus_power_watts);
        self.modbus_ctx
            .write_multiple_registers(74, &data)
            .await
            .map_err(|e| anyhow!(e))?;
        Ok(())
    }

    pub async fn set_pv_power(&mut self, pv_power: f32) -> anyhow::Result<()> {
        dbg!(pv_power);
        let data = idm_float_to_u16(pv_power);
        self.modbus_ctx.write_multiple_registers(78, &data).await?;

        Ok(())
    }
}

pub fn idm_float_to_u16(value: f32) -> [u16; 2] {
    let value: [u8; 4] = value.to_be_bytes();
    [
        value[3] as u16 + ((value[2] as u16) << 8),
        value[1] as u16 + ((value[0] as u16) << 8),
    ]
}

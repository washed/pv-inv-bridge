use edgedb_derive::Queryable;
use edgedb_protocol::model::Uuid;
use std::env;
use std::time::Duration;
use tokio_retry::strategy::ExponentialBackoff;
use tokio_retry::Retry;
use tracing::info;

use crate::inverter::PVInverterData;

#[derive(Debug, Queryable)]
pub struct QueryableInsertResponse {
    pub id: Uuid,
}

pub struct DBInserter {
    db_conn: edgedb_tokio::Client,
}

impl DBInserter {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let edgedb_dsn = env::var("EDGEDB_DSN")?;
        let mut builder = edgedb_tokio::Builder::new();
        builder.dsn(edgedb_dsn.as_str())?;
        let config = builder.build_env().await?;
        let db_conn = edgedb_tokio::Client::new(&config);
        db_conn.ensure_connected().await?;
        Ok(Self { db_conn })
    }

    pub async fn new_retrying() -> Result<Self, Box<dyn std::error::Error>> {
        let retry_strategy =
            ExponentialBackoff::from_millis(100).max_delay(Duration::from_secs(10));

        Retry::spawn(retry_strategy, Self::new).await
    }

    pub async fn insert_pv_inverter_data(
        &self,
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
        self.db_conn.execute(&query, &()).await?;
        info!("Inserted {:#?}", data);
        Ok(0)
    }
}

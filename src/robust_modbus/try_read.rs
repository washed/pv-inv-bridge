use crate::robust_modbus::{
    registers::{
        Coils, DiscreteInputs, HoldingRegisters, InputRegisters, ReadWriteMultipleRegisters,
    },
    Coil, RobustContext, Word,
};
use std::io;
use tokio_modbus::{prelude::*, Error as ModbusError, Result as ModbusResult};

pub(crate) trait TryRead {
    type Result;

    async fn try_read(self, robust_ctx: &RobustContext) -> ModbusResult<Vec<Self::Result>>;
}

impl TryRead for Coils {
    type Result = Coil;
    async fn try_read(self, robust_ctx: &RobustContext) -> ModbusResult<Vec<Self::Result>> {
        let ctx = robust_ctx.ctx.clone();
        let res = {
            let mut ctx_guard = ctx.lock().await;
            let ctx = ctx_guard
                .as_mut()
                .map_err(|e| ModbusError::Transport(io::Error::new(e.kind(), e.to_string())));

            match ctx {
                Ok(ctx) => ctx.read_coils(self.addr, self.cnt).await,
                Err(e) => Err(e),
            }
        };

        if res.is_err() {
            RobustContext::refresh_context(
                ctx.clone(),
                robust_ctx.socket_address,
                robust_ctx.slave,
            )
            .await;
        }

        res
    }
}

impl TryRead for DiscreteInputs {
    type Result = Coil;
    async fn try_read(self, robust_ctx: &RobustContext) -> ModbusResult<Vec<Self::Result>> {
        let ctx = robust_ctx.ctx.clone();
        let res = {
            let mut ctx_guard = ctx.lock().await;
            let ctx = ctx_guard
                .as_mut()
                .map_err(|e| ModbusError::Transport(io::Error::new(e.kind(), e.to_string())));

            match ctx {
                Ok(ctx) => ctx.read_discrete_inputs(self.addr, self.cnt).await,
                Err(e) => Err(e),
            }
        };

        if res.is_err() {
            RobustContext::refresh_context(
                ctx.clone(),
                robust_ctx.socket_address,
                robust_ctx.slave,
            )
            .await;
        }

        res
    }
}

impl TryRead for HoldingRegisters {
    type Result = Word;
    async fn try_read(self, robust_ctx: &RobustContext) -> ModbusResult<Vec<Self::Result>> {
        let ctx = robust_ctx.ctx.clone();
        let res = {
            let mut ctx_guard = ctx.lock().await;
            let ctx = ctx_guard
                .as_mut()
                .map_err(|e| ModbusError::Transport(io::Error::new(e.kind(), e.to_string())));

            match ctx {
                Ok(ctx) => ctx.read_holding_registers(self.addr, self.cnt).await,
                Err(e) => Err(e),
            }
        };

        if res.is_err() {
            RobustContext::refresh_context(
                ctx.clone(),
                robust_ctx.socket_address,
                robust_ctx.slave,
            )
            .await;
        }

        res
    }
}

impl TryRead for InputRegisters {
    type Result = Word;
    async fn try_read(self, robust_ctx: &RobustContext) -> ModbusResult<Vec<Self::Result>> {
        let ctx = robust_ctx.ctx.clone();
        let res = {
            let mut ctx_guard = ctx.lock().await;
            let ctx = ctx_guard
                .as_mut()
                .map_err(|e| ModbusError::Transport(io::Error::new(e.kind(), e.to_string())));

            match ctx {
                Ok(ctx) => ctx.read_input_registers(self.addr, self.cnt).await,
                Err(e) => Err(e),
            }
        };

        if res.is_err() {
            RobustContext::refresh_context(
                ctx.clone(),
                robust_ctx.socket_address,
                robust_ctx.slave,
            )
            .await;
        }

        res
    }
}

impl<'a> TryRead for ReadWriteMultipleRegisters<'a> {
    type Result = Word;
    async fn try_read(self, robust_ctx: &RobustContext) -> ModbusResult<Vec<Self::Result>> {
        let ctx = robust_ctx.ctx.clone();
        let res = {
            let mut ctx_guard = ctx.lock().await;
            let ctx = ctx_guard
                .as_mut()
                .map_err(|e| ModbusError::Transport(io::Error::new(e.kind(), e.to_string())));

            match ctx {
                Ok(ctx) => {
                    ctx.read_write_multiple_registers(
                        self.read_addr,
                        self.read_count,
                        self.write_addr,
                        self.write_data,
                    )
                    .await
                }
                Err(e) => Err(e),
            }
        };

        if res.is_err() {
            RobustContext::refresh_context(
                ctx.clone(),
                robust_ctx.socket_address,
                robust_ctx.slave,
            )
            .await;
        }

        res
    }
}

use crate::robust_modbus::{Coil, RobustContext, Word};
use std::io;
use tokio_modbus::{prelude::*, Address, Error as ModbusError, Result as ModbusResult};

pub(crate) trait TryWrite {
    async fn try_write(self, robust_ctx: &RobustContext) -> ModbusResult<()>;
}

pub(crate) struct CoilWrite {
    pub addr: Address,
    pub coil: Coil,
}

pub(crate) struct RegisterWrite {
    pub addr: Address,
    pub word: Word,
}

pub(crate) struct MultipleCoilsWrite<'a> {
    pub addr: Address,
    pub coils: &'a [Coil],
}

pub(crate) struct MultipleRegistersWrite<'a> {
    pub addr: Address,
    pub words: &'a [Word],
}

pub(crate) struct RegisterMaskedWrite {
    pub addr: Address,
    pub and_mask: Word,
    pub or_mask: Word,
}

impl TryWrite for CoilWrite {
    async fn try_write(self, robust_ctx: &RobustContext) -> ModbusResult<()> {
        let ctx = robust_ctx.ctx.clone();
        let res = {
            let mut ctx_guard = ctx.lock().await;
            let ctx = ctx_guard
                .as_mut()
                .map_err(|e| ModbusError::Transport(io::Error::new(e.kind(), e.to_string())));

            match ctx {
                Ok(ctx) => ctx.write_single_coil(self.addr, self.coil).await,
                Err(e) => Err(e),
            }
        };

        if res.is_err() {
            RobustContext::refresh_context(ctx.clone(), &robust_ctx.host, robust_ctx.slave).await;
        }

        res
    }
}

impl TryWrite for RegisterWrite {
    async fn try_write(self, robust_ctx: &RobustContext) -> ModbusResult<()> {
        let ctx = robust_ctx.ctx.clone();
        let res = {
            let mut ctx_guard = ctx.lock().await;
            let ctx = ctx_guard
                .as_mut()
                .map_err(|e| ModbusError::Transport(io::Error::new(e.kind(), e.to_string())));

            match ctx {
                Ok(ctx) => ctx.write_single_register(self.addr, self.word).await,
                Err(e) => Err(e),
            }
        };

        if res.is_err() {
            RobustContext::refresh_context(ctx.clone(), &robust_ctx.host, robust_ctx.slave).await;
        }

        res
    }
}

impl<'a> TryWrite for MultipleCoilsWrite<'a> {
    async fn try_write(self, robust_ctx: &RobustContext) -> ModbusResult<()> {
        let ctx = robust_ctx.ctx.clone();
        let res = {
            let mut ctx_guard = ctx.lock().await;
            let ctx = ctx_guard
                .as_mut()
                .map_err(|e| ModbusError::Transport(io::Error::new(e.kind(), e.to_string())));

            match ctx {
                Ok(ctx) => ctx.write_multiple_coils(self.addr, self.coils).await,
                Err(e) => Err(e),
            }
        };

        if res.is_err() {
            RobustContext::refresh_context(ctx.clone(), &robust_ctx.host, robust_ctx.slave).await;
        }

        res
    }
}

impl<'a> TryWrite for MultipleRegistersWrite<'a> {
    async fn try_write(self, robust_ctx: &RobustContext) -> ModbusResult<()> {
        let ctx = robust_ctx.ctx.clone();
        let res = {
            let mut ctx_guard = ctx.lock().await;
            let ctx = ctx_guard
                .as_mut()
                .map_err(|e| ModbusError::Transport(io::Error::new(e.kind(), e.to_string())));

            match ctx {
                Ok(ctx) => ctx.write_multiple_registers(self.addr, self.words).await,
                Err(e) => Err(e),
            }
        };

        if res.is_err() {
            RobustContext::refresh_context(ctx.clone(), &robust_ctx.host, robust_ctx.slave).await;
        }

        res
    }
}

impl TryWrite for RegisterMaskedWrite {
    async fn try_write(self, robust_ctx: &RobustContext) -> ModbusResult<()> {
        let ctx = robust_ctx.ctx.clone();
        let res = {
            let mut ctx_guard = ctx.lock().await;
            let ctx = ctx_guard
                .as_mut()
                .map_err(|e| ModbusError::Transport(io::Error::new(e.kind(), e.to_string())));

            match ctx {
                Ok(ctx) => {
                    ctx.masked_write_register(self.addr, self.and_mask, self.or_mask)
                        .await
                }
                Err(e) => Err(e),
            }
        };

        if res.is_err() {
            RobustContext::refresh_context(ctx.clone(), &robust_ctx.host, robust_ctx.slave).await;
        }

        res
    }
}

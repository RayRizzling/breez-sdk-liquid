use std::borrow::Cow::{self, Owned};
use std::sync::Arc;

use anyhow::Result;
use clap::{arg, Parser};
use rustyline::highlight::Highlighter;
use rustyline::history::DefaultHistory;
use rustyline::Editor;
use rustyline::{hint::HistoryHinter, Completer, Helper, Hinter, Validator};

use breez_sdk_liquid::{ReceivePaymentRequest, Wallet};
use serde::Serialize;
use serde_json::to_string_pretty;

#[derive(Parser, Debug, Clone, PartialEq)]
pub(crate) enum Command {
    /// Send lbtc and receive btc through a swap
    SendPayment { bolt11: String },
    /// Receive lbtc and send btc through a swap
    ReceivePayment {
        #[arg(short, long)]
        onchain_amount_sat: Option<u64>,

        #[arg(short, long)]
        invoice_amount_sat: Option<u64>,
    },
    /// List incoming and outgoing payments
    ListPayments,
    /// Get the balance of the currently loaded wallet
    GetInfo,
    /// Empties the encrypted wallet transaction cache
    EmptyCache,
}

#[derive(Helper, Completer, Hinter, Validator)]
pub(crate) struct CliHelper {
    #[rustyline(Hinter)]
    pub(crate) hinter: HistoryHinter,
}

impl Highlighter for CliHelper {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }
}

#[derive(Serialize)]
pub(crate) struct CommandResult<T: Serialize> {
    pub success: bool,
    pub message: T,
}

macro_rules! command_result {
    ($expr:expr) => {{
        to_string_pretty(&CommandResult {
            success: true,
            message: $expr,
        })?
    }};
}

pub(crate) fn handle_command(
    _rl: &mut Editor<CliHelper, DefaultHistory>,
    wallet: &Arc<Wallet>,
    command: Command,
) -> Result<String> {
    Ok(match command {
        Command::ReceivePayment {
            onchain_amount_sat,
            invoice_amount_sat,
        } => {
            let response = wallet.receive_payment(ReceivePaymentRequest {
                invoice_amount_sat,
                onchain_amount_sat,
            })?;
            qr2term::print_qr(response.invoice.clone())?;
            command_result!(response)
        }
        Command::SendPayment { bolt11 } => {
            let prepare_response = wallet.prepare_payment(&bolt11)?;
            let response = wallet.send_payment(&prepare_response)?;
            command_result!(response)
        }
        Command::GetInfo => {
            command_result!(wallet.get_info(true)?)
        }
        Command::ListPayments => {
            command_result!(wallet.list_payments(true, true)?)
        }
        Command::EmptyCache => {
            wallet.empty_wallet_cache()?;
            command_result!("Cache emptied successfully")
        }
    })
}

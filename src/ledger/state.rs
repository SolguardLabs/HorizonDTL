use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::{
    AccountId, AccountState, Amount, AssetConfig, AssetId, DeliveryNote, Digest, FeeLedger,
    FeeSchedule, HorizonError, HorizonResult, IndexObservation, JournalEntry, JournalOp, NoteId,
    NoteRecord, OperatorRegistry, OperatorRole, OracleBook, ProtocolConfig, PublicIdentity,
    RedemptionTicket, RiskEngine, RiskLimits, RouteBook, SettlementRoute, SignedIngressOrder,
    SignedRedemptionTicket, TxId, VaultConfig, VaultId, VaultState,
};

#[derive(Clone, Debug, Serialize)]
pub struct HorizonLedger {
    network_id: u32,
    assets: BTreeMap<AssetId, AssetConfig>,
    accounts: BTreeMap<AccountId, AccountState>,
    total_supply: BTreeMap<AssetId, Amount>,
    vaults: BTreeMap<VaultId, VaultState>,
    notes: BTreeMap<NoteId, DeliveryNote>,
    note_records: BTreeMap<NoteId, NoteRecord>,
    processed_tickets: BTreeSet<crate::TicketId>,
    seen_transactions: BTreeSet<TxId>,
    operators: OperatorRegistry,
    oracle_book: OracleBook,
    route_book: RouteBook,
    fee_ledger: FeeLedger,
    risk_engine: RiskEngine,
    journal: Vec<JournalEntry>,
}

#[derive(Serialize)]
struct LedgerDigestView<'a> {
    network_id: u32,
    assets: &'a BTreeMap<AssetId, AssetConfig>,
    accounts: &'a BTreeMap<AccountId, AccountState>,
    total_supply: &'a BTreeMap<AssetId, Amount>,
    vaults: &'a BTreeMap<VaultId, VaultState>,
    notes: &'a BTreeMap<NoteId, DeliveryNote>,
    note_records: &'a BTreeMap<NoteId, NoteRecord>,
    processed_tickets: &'a BTreeSet<crate::TicketId>,
    seen_transactions: &'a BTreeSet<TxId>,
    operators: &'a OperatorRegistry,
    oracle_book: &'a OracleBook,
    route_book: &'a RouteBook,
    fee_ledger: &'a FeeLedger,
    risk_engine: &'a RiskEngine,
    journal_len: usize,
}

impl HorizonLedger {
    pub fn new(network_id: u32) -> Self {
        Self {
            network_id,
            assets: BTreeMap::new(),
            accounts: BTreeMap::new(),
            total_supply: BTreeMap::new(),
            vaults: BTreeMap::new(),
            notes: BTreeMap::new(),
            note_records: BTreeMap::new(),
            processed_tickets: BTreeSet::new(),
            seen_transactions: BTreeSet::new(),
            operators: OperatorRegistry::default(),
            oracle_book: OracleBook::default(),
            route_book: RouteBook::default(),
            fee_ledger: FeeLedger::default(),
            risk_engine: RiskEngine::default(),
            journal: Vec::new(),
        }
    }

    pub const fn network_id(&self) -> u32 {
        self.network_id
    }

    pub fn configure_protocol(&mut self, config: ProtocolConfig) -> HorizonResult<TxId> {
        self.account(config.admin)?;
        self.operators.configure(config);
        let tx_id = TxId::from_serializable(
            "horizon-protocol-config-tx-v1",
            &(self.network_id, config, self.journal.len() as u64),
        )?;
        self.append_journal(tx_id, JournalOp::ProtocolConfigured { config })?;
        Ok(tx_id)
    }

    pub fn grant_operator_role(
        &mut self,
        account: AccountId,
        role: OperatorRole,
    ) -> HorizonResult<TxId> {
        self.account(account)?;
        self.operators.grant_role(account, role);
        let tx_id = TxId::from_serializable(
            "horizon-operator-role-tx-v1",
            &(self.network_id, account, role, self.journal.len() as u64),
        )?;
        self.append_journal(tx_id, JournalOp::OperatorRoleGranted { account, role })?;
        Ok(tx_id)
    }

    pub fn set_risk_limits(&mut self, limits: RiskLimits) {
        self.risk_engine.set_limits(limits);
    }

    pub fn configure_fee_schedule(&mut self, schedule: FeeSchedule) -> HorizonResult<TxId> {
        self.account(schedule.treasury)?;
        self.operators
            .require_role(schedule.treasury, OperatorRole::Treasury)?;
        self.fee_ledger.configure(schedule);
        let tx_id = TxId::from_serializable(
            "horizon-fee-schedule-tx-v1",
            &(self.network_id, schedule, self.journal.len() as u64),
        )?;
        self.append_journal(
            tx_id,
            JournalOp::FeeScheduleConfigured {
                treasury: schedule.treasury,
                protocol_fee_bps: schedule.protocol_fee_bps,
                reserve_fee_bps: schedule.reserve_fee_bps,
            },
        )?;
        Ok(tx_id)
    }

    pub fn register_asset(&mut self, config: AssetConfig) -> HorizonResult<()> {
        if self.assets.contains_key(&config.id) {
            return Err(HorizonError::AssetAlreadyExists(config.id));
        }
        self.total_supply
            .entry(config.id)
            .or_insert_with(Amount::zero);
        self.assets.insert(config.id, config);
        Ok(())
    }

    pub fn register_account(&mut self, identity: PublicIdentity) -> HorizonResult<()> {
        identity.verify_consistency()?;
        if self.accounts.contains_key(&identity.account) {
            return Err(HorizonError::AccountAlreadyExists(identity.account));
        }
        self.accounts
            .insert(identity.account, AccountState::new(identity));
        Ok(())
    }

    pub fn register_vault(&mut self, config: VaultConfig) -> HorizonResult<TxId> {
        self.account(config.controller)?;
        self.account(config.index_authority)?;
        self.asset_config(config.asset)?;
        if self.vaults.contains_key(&config.vault_id) {
            return Err(HorizonError::VaultAlreadyExists(config.vault_id));
        }
        self.vaults.insert(config.vault_id, VaultState::new(config));
        let tx_id = TxId::from_serializable(
            "horizon-register-vault-tx-v1",
            &(self.network_id, config, self.journal.len() as u64),
        )?;
        self.append_journal(
            tx_id,
            JournalOp::VaultRegistered {
                vault_id: config.vault_id,
                controller: config.controller,
                asset: config.asset,
            },
        )?;
        Ok(tx_id)
    }

    pub fn register_route(&mut self, route: SettlementRoute) -> HorizonResult<TxId> {
        self.vault(route.source_vault)?;
        self.vault(route.payout_vault)?;
        self.asset_config(route.asset)?;
        let route_id = self.route_book.register_route(route)?;
        let tx_id = TxId::from_serializable(
            "horizon-route-tx-v1",
            &(self.network_id, route, self.journal.len() as u64),
        )?;
        self.append_journal(
            tx_id,
            JournalOp::RouteRegistered {
                route_id,
                source_vault: route.source_vault,
                payout_vault: route.payout_vault,
                asset: route.asset,
            },
        )?;
        Ok(tx_id)
    }

    pub fn publish_index(&mut self, observation: IndexObservation) -> HorizonResult<TxId> {
        self.account(observation.observer)?;
        self.operators
            .require_role(observation.observer, OperatorRole::Oracle)?;
        let vault = self.vault(observation.vault_id)?;
        if vault.config.asset != observation.asset {
            return Err(HorizonError::Policy(
                "observation asset mismatch".to_owned(),
            ));
        }
        if vault.config.index_authority != observation.observer {
            return Err(HorizonError::UnauthorizedSigner {
                expected: vault.config.index_authority,
                received: observation.observer,
            });
        }
        self.vault_mut(observation.vault_id)?
            .reprice(observation.share_index);
        self.oracle_book.publish(observation);
        let tx_id = TxId::from_serializable(
            "horizon-index-observation-tx-v1",
            &(self.network_id, observation, self.journal.len() as u64),
        )?;
        self.append_journal(
            tx_id,
            JournalOp::VaultIndexPublished {
                vault_id: observation.vault_id,
                observer: observation.observer,
                share_index: observation.share_index,
                epoch: observation.epoch,
            },
        )?;
        Ok(tx_id)
    }

    pub fn credit_genesis(
        &mut self,
        account: AccountId,
        asset: AssetId,
        amount: Amount,
    ) -> HorizonResult<TxId> {
        if amount.is_zero() {
            return Err(HorizonError::ZeroAmount);
        }
        self.asset_config(asset)?;
        let mut candidate = self.clone();
        candidate.credit(account, asset, amount)?;
        let supply = candidate.total_supply_of(asset).checked_add(amount)?;
        candidate.total_supply.insert(asset, supply);
        let tx_id = TxId::from_serializable(
            "horizon-genesis-credit-tx-v1",
            &(
                self.network_id,
                account,
                asset,
                amount,
                self.journal.len() as u64,
            ),
        )?;
        candidate.append_journal(
            tx_id,
            JournalOp::GenesisCredit {
                account,
                asset,
                amount,
            },
        )?;
        candidate.verify_conservation(asset)?;
        *self = candidate;
        Ok(tx_id)
    }

    pub fn deposit_vault(
        &mut self,
        owner: AccountId,
        vault_id: VaultId,
        amount: Amount,
    ) -> HorizonResult<TxId> {
        if amount.is_zero() {
            return Err(HorizonError::ZeroAmount);
        }
        let mut candidate = self.clone();
        let asset = candidate.vault(vault_id)?.config.asset;
        candidate.debit(owner, asset, amount)?;
        candidate.vault_mut(vault_id)?.deposit(amount)?;
        let tx_id = TxId::from_serializable(
            "horizon-vault-deposit-tx-v1",
            &(
                self.network_id,
                owner,
                vault_id,
                amount,
                self.journal.len() as u64,
            ),
        )?;
        candidate.append_journal(
            tx_id,
            JournalOp::VaultDeposit {
                vault_id,
                owner,
                amount,
            },
        )?;
        candidate.verify_conservation(asset)?;
        *self = candidate;
        Ok(tx_id)
    }

    pub fn issue_note(&mut self, signed: &SignedIngressOrder) -> HorizonResult<TxId> {
        let mut candidate = self.clone();
        let tx_id = candidate.issue_note_inner(signed)?;
        let asset = signed.order.asset;
        candidate.verify_conservation(asset)?;
        *self = candidate;
        Ok(tx_id)
    }

    pub fn settle_ticket(&mut self, signed: &SignedRedemptionTicket) -> HorizonResult<TxId> {
        let mut candidate = self.clone();
        let tx_id = candidate.settle_ticket_inner(signed)?;
        let note = candidate
            .notes
            .get(&signed.ticket.note_id)
            .copied()
            .ok_or(HorizonError::NoteNotFound(signed.ticket.note_id))?;
        candidate.verify_conservation(note.asset)?;
        *self = candidate;
        Ok(tx_id)
    }

    pub fn sweep_vault_surplus(
        &mut self,
        controller: AccountId,
        vault_id: VaultId,
        amount: Amount,
    ) -> HorizonResult<TxId> {
        let mut candidate = self.clone();
        let vault = candidate.vault(vault_id)?;
        let asset = vault.config.asset;
        let expected_controller = vault.config.controller;
        if expected_controller != controller {
            return Err(HorizonError::UnauthorizedSigner {
                expected: expected_controller,
                received: controller,
            });
        }
        candidate.vault_mut(vault_id)?.sweep(amount)?;
        candidate.credit(controller, asset, amount)?;
        let tx_id = TxId::from_serializable(
            "horizon-vault-surplus-sweep-tx-v1",
            &(
                self.network_id,
                controller,
                vault_id,
                amount,
                self.journal.len() as u64,
            ),
        )?;
        candidate.append_journal(
            tx_id,
            JournalOp::VaultSurplusSwept {
                vault_id,
                controller,
                amount,
            },
        )?;
        candidate.verify_conservation(asset)?;
        *self = candidate;
        Ok(tx_id)
    }

    pub fn balance_of(&self, account: AccountId, asset: AssetId) -> HorizonResult<Amount> {
        Ok(self.account(account)?.balance_of(asset))
    }

    pub fn ingress_nonce(&self, account: AccountId) -> HorizonResult<u64> {
        Ok(self.account(account)?.next_ingress_nonce)
    }

    pub fn ticket_nonce(&self, account: AccountId) -> HorizonResult<u64> {
        Ok(self.account(account)?.next_ticket_nonce)
    }

    pub fn total_supply_of(&self, asset: AssetId) -> Amount {
        self.total_supply
            .get(&asset)
            .copied()
            .unwrap_or_else(Amount::zero)
    }

    pub fn vault(&self, vault_id: VaultId) -> HorizonResult<&VaultState> {
        self.vaults
            .get(&vault_id)
            .ok_or(HorizonError::VaultNotFound(vault_id))
    }

    pub fn note(&self, note_id: NoteId) -> HorizonResult<DeliveryNote> {
        self.notes
            .get(&note_id)
            .copied()
            .ok_or(HorizonError::NoteNotFound(note_id))
    }

    pub fn note_count(&self) -> usize {
        self.notes.len()
    }

    pub fn processed_ticket_count(&self) -> usize {
        self.processed_tickets.len()
    }

    pub fn route_count(&self) -> usize {
        self.route_book.route_count()
    }

    pub fn observation_count(&self) -> usize {
        self.oracle_book.observation_count()
    }

    pub fn operator_count(&self) -> usize {
        self.operators.operator_count()
    }

    pub fn fee_asset_count(&self) -> usize {
        self.fee_ledger.accrued_asset_count()
    }

    pub fn journal(&self) -> &[JournalEntry] {
        &self.journal
    }

    pub fn state_digest(&self) -> HorizonResult<Digest> {
        Digest::from_serializable(
            "horizon-ledger-state-v1",
            &LedgerDigestView {
                network_id: self.network_id,
                assets: &self.assets,
                accounts: &self.accounts,
                total_supply: &self.total_supply,
                vaults: &self.vaults,
                notes: &self.notes,
                note_records: &self.note_records,
                processed_tickets: &self.processed_tickets,
                seen_transactions: &self.seen_transactions,
                operators: &self.operators,
                oracle_book: &self.oracle_book,
                route_book: &self.route_book,
                fee_ledger: &self.fee_ledger,
                risk_engine: &self.risk_engine,
                journal_len: self.journal.len(),
            },
        )
    }

    pub fn is_conserved(&self, asset: AssetId) -> HorizonResult<bool> {
        self.verify_conservation(asset)?;
        Ok(true)
    }

    fn issue_note_inner(&mut self, signed: &SignedIngressOrder) -> HorizonResult<TxId> {
        signed.verify()?;
        let order = signed.order;
        self.operators.ensure_not_paused()?;
        self.operators
            .require_role(order.issuer, OperatorRole::Issuer)?;
        self.operators
            .require_role(order.beneficiary, OperatorRole::Beneficiary)?;
        if order.network_id != self.network_id {
            return Err(HorizonError::Policy("network mismatch".to_owned()));
        }
        if self.ingress_nonce(order.issuer)? != order.owner_nonce {
            return Err(HorizonError::NonceMismatch {
                account: order.issuer,
                expected: self.ingress_nonce(order.issuer)?,
                received: order.owner_nonce,
            });
        }
        let vault = self.vault(order.source_vault)?;
        if vault.config.asset != order.asset {
            return Err(HorizonError::Policy("note asset mismatch".to_owned()));
        }
        if !self.asset_config(order.asset)?.settlement_enabled {
            return Err(HorizonError::Policy("asset settlement disabled".to_owned()));
        }
        let note = order.note(vault.share_index)?;
        if self.notes.contains_key(&note.note_id) {
            return Err(HorizonError::NoteAlreadyExists(note.note_id));
        }
        let tx_id = signed.tx_id()?;
        if self.seen_transactions.contains(&tx_id) {
            return Err(HorizonError::DuplicateTransaction(tx_id));
        }
        self.debit(order.issuer, order.asset, order.amount)?;
        self.vault_mut(order.source_vault)?.deposit(order.amount)?;
        self.vault_mut(order.source_vault)?
            .lock_notional(order.amount)?;
        self.account_mut(order.issuer)?.advance_ingress_nonce()?;
        self.notes.insert(note.note_id, note);
        self.note_records.insert(
            note.note_id,
            NoteRecord {
                note_id: note.note_id,
                source_vault: order.source_vault,
                issuer: order.issuer,
                settled: false,
            },
        );
        self.seen_transactions.insert(tx_id);
        self.append_journal(
            tx_id,
            JournalOp::NoteIssued {
                note_id: note.note_id,
                source_vault: order.source_vault,
                issuer: order.issuer,
                beneficiary: order.beneficiary,
                amount: order.amount,
            },
        )?;
        Ok(tx_id)
    }

    fn settle_ticket_inner(&mut self, signed: &SignedRedemptionTicket) -> HorizonResult<TxId> {
        signed.verify()?;
        let ticket: RedemptionTicket = signed.ticket;
        self.operators.ensure_not_paused()?;
        self.operators
            .require_role(ticket.beneficiary, OperatorRole::Beneficiary)?;
        self.operators
            .require_role(ticket.relayer, OperatorRole::Relayer)?;
        if ticket.network_id != self.network_id {
            return Err(HorizonError::Policy("network mismatch".to_owned()));
        }
        if self.processed_tickets.contains(&ticket.ticket_id) {
            return Err(HorizonError::TicketProcessed(ticket.ticket_id));
        }
        let note = self.note(ticket.note_id)?;
        let mut record = *self
            .note_records
            .get(&ticket.note_id)
            .ok_or(HorizonError::NoteNotFound(ticket.note_id))?;
        if record.settled {
            return Err(HorizonError::NoteSettled(ticket.note_id));
        }
        if ticket.beneficiary != note.beneficiary {
            return Err(HorizonError::UnauthorizedSigner {
                expected: note.beneficiary,
                received: ticket.beneficiary,
            });
        }
        if ticket.note_digest != note.digest()? {
            return Err(HorizonError::Policy("note digest mismatch".to_owned()));
        }
        if ticket.settlement_epoch < note.maturity_epoch {
            return Err(HorizonError::Policy("note is not mature".to_owned()));
        }
        if ticket.settlement_epoch < self.risk_engine.limits().current_epoch {
            return Err(HorizonError::Policy("stale redemption ticket".to_owned()));
        }
        if self.ticket_nonce(ticket.beneficiary)? != ticket.ticket_nonce {
            return Err(HorizonError::NonceMismatch {
                account: ticket.beneficiary,
                expected: self.ticket_nonce(ticket.beneficiary)?,
                received: ticket.ticket_nonce,
            });
        }
        let payout_vault = self.vault(ticket.payout_vault)?;
        if payout_vault.config.asset != note.asset {
            return Err(HorizonError::Policy(
                "payout vault asset mismatch".to_owned(),
            ));
        }
        let gross_amount = payout_vault
            .share_index
            .amount_from_units(note.locked_units)?;
        let route = self.route_book.resolve_route(
            record.source_vault,
            ticket.payout_vault,
            note.asset,
            note.amount,
            ticket.relayer_fee,
        )?;
        let risk = self.risk_engine.evaluate_ticket(
            ticket.ticket_id,
            payout_vault,
            gross_amount,
            ticket.relayer_fee,
        )?;
        let protocol_fee = self.fee_ledger.protocol_fee(ticket.relayer_fee)?;
        let reserve_fee = self.fee_ledger.reserve_fee(ticket.relayer_fee)?;
        let tx_id = signed.tx_id()?;
        if self.seen_transactions.contains(&tx_id) {
            return Err(HorizonError::DuplicateTransaction(tx_id));
        }
        let beneficiary_amount = gross_amount.checked_sub(ticket.relayer_fee)?;
        self.vault_mut(ticket.payout_vault)?.pay(gross_amount)?;
        self.vault_mut(record.source_vault)?
            .release_notional(note.amount)?;
        self.credit(ticket.beneficiary, note.asset, beneficiary_amount)?;
        if !ticket.relayer_fee.is_zero() {
            self.credit(ticket.relayer, note.asset, ticket.relayer_fee)?;
        }
        self.fee_ledger.accrue(note.asset, protocol_fee)?;
        self.fee_ledger
            .add_reserve_buffer(note.asset, reserve_fee)?;
        self.account_mut(ticket.beneficiary)?
            .advance_ticket_nonce()?;
        record.settled = true;
        self.note_records.insert(ticket.note_id, record);
        self.processed_tickets.insert(ticket.ticket_id);
        self.seen_transactions.insert(tx_id);
        self.append_journal(
            tx_id,
            JournalOp::TicketSettled {
                ticket_id: ticket.ticket_id,
                note_id: ticket.note_id,
                route_id: route.route_id,
                source_vault: record.source_vault,
                payout_vault: ticket.payout_vault,
                beneficiary: ticket.beneficiary,
                beneficiary_amount,
                relayer: ticket.relayer,
                relayer_fee: ticket.relayer_fee,
                protocol_fee,
                reserve_fee,
                risk: Box::new(risk),
            },
        )?;
        Ok(tx_id)
    }

    fn append_journal(&mut self, tx_id: TxId, op: JournalOp) -> HorizonResult<()> {
        let entry = JournalEntry {
            sequence: self.journal.len() as u64,
            tx_id,
            op,
            state_digest: self.state_digest()?,
        };
        self.journal.push(entry);
        Ok(())
    }

    fn asset_config(&self, asset: AssetId) -> HorizonResult<AssetConfig> {
        self.assets
            .get(&asset)
            .copied()
            .ok_or(HorizonError::AssetNotFound(asset))
    }

    fn account(&self, account: AccountId) -> HorizonResult<&AccountState> {
        self.accounts
            .get(&account)
            .ok_or(HorizonError::AccountNotFound(account))
    }

    fn account_mut(&mut self, account: AccountId) -> HorizonResult<&mut AccountState> {
        self.accounts
            .get_mut(&account)
            .ok_or(HorizonError::AccountNotFound(account))
    }

    fn vault_mut(&mut self, vault_id: VaultId) -> HorizonResult<&mut VaultState> {
        self.vaults
            .get_mut(&vault_id)
            .ok_or(HorizonError::VaultNotFound(vault_id))
    }

    fn credit(&mut self, account: AccountId, asset: AssetId, amount: Amount) -> HorizonResult<()> {
        self.account_mut(account)?.credit(asset, amount)
    }

    fn debit(&mut self, account: AccountId, asset: AssetId, amount: Amount) -> HorizonResult<()> {
        self.account_mut(account)?.debit(account, asset, amount)
    }

    fn verify_conservation(&self, asset: AssetId) -> HorizonResult<()> {
        let account_total = self
            .accounts
            .values()
            .try_fold(Amount::zero(), |accumulator, account| {
                accumulator.checked_add(account.balance_of(asset))
            })?;
        let vault_total = self
            .vaults
            .values()
            .filter(|vault| vault.config.asset == asset)
            .try_fold(Amount::zero(), |accumulator, vault| {
                accumulator.checked_add(vault.reserve_balance)
            })?;
        let observed = account_total.checked_add(vault_total)?;
        let expected = self.total_supply_of(asset);
        if observed != expected {
            return Err(HorizonError::Conservation {
                asset,
                expected,
                observed,
            });
        }
        Ok(())
    }
}

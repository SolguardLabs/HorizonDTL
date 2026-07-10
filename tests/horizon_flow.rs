use horizon_dtl::{
    Amount, AssetConfig, Bps, DeliveryNote, Digest, FeeSchedule, HorizonError, HorizonLedger,
    HorizonResult, IndexObservation, IngressOrder, KeyPair, OperatorRole, ProtocolConfig,
    RedemptionTicket, RiskLimits, SettlementRoute, ShareIndex, SignedIngressOrder,
    SignedRedemptionTicket, TxId, VaultConfig, VaultId,
};

const NETWORK_ID: u32 = 72_901;
const SETTLEMENT_EPOCH: u64 = 500;

struct Fixture {
    ledger: HorizonLedger,
    issuer: KeyPair,
    beneficiary: KeyPair,
    relayer: KeyPair,
    asset: AssetConfig,
    primary_vault: VaultId,
    reserve_vault: VaultId,
    primary_controller: KeyPair,
}

fn fixture() -> HorizonResult<Fixture> {
    let issuer = keyed(11);
    let beneficiary = keyed(22);
    let relayer = keyed(33);
    let liquidity_provider = keyed(44);
    let primary_controller = keyed(55);
    let reserve_controller = keyed(66);
    let asset = AssetConfig::new("HUSD", 6, Bps::new(80)?)?;
    let mut ledger = HorizonLedger::new(NETWORK_ID);

    ledger.set_risk_limits(RiskLimits::new(
        SETTLEMENT_EPOCH,
        Amount::new(50_000_000_000)?,
        Bps::new(100)?,
        Amount::zero(),
    )?);
    ledger.register_asset(asset)?;

    for identity in [
        issuer.public_identity(),
        beneficiary.public_identity(),
        relayer.public_identity(),
        liquidity_provider.public_identity(),
        primary_controller.public_identity(),
        reserve_controller.public_identity(),
    ] {
        ledger.register_account(identity)?;
    }

    ledger.configure_protocol(ProtocolConfig::new(
        primary_controller.public_identity().account,
        SETTLEMENT_EPOCH,
        2,
        Digest::from_parts("horizon-config-salt-v1", &[b"primary"]),
    )?)?;

    for (account, role) in [
        (issuer.public_identity().account, OperatorRole::Issuer),
        (
            beneficiary.public_identity().account,
            OperatorRole::Beneficiary,
        ),
        (relayer.public_identity().account, OperatorRole::Relayer),
        (
            primary_controller.public_identity().account,
            OperatorRole::VaultController,
        ),
        (
            reserve_controller.public_identity().account,
            OperatorRole::VaultController,
        ),
        (
            primary_controller.public_identity().account,
            OperatorRole::Oracle,
        ),
        (
            primary_controller.public_identity().account,
            OperatorRole::Treasury,
        ),
    ] {
        ledger.grant_operator_role(account, role)?;
    }

    ledger.credit_genesis(
        issuer.public_identity().account,
        asset.id,
        Amount::new(20_000_000_000)?,
    )?;
    ledger.credit_genesis(
        liquidity_provider.public_identity().account,
        asset.id,
        Amount::new(120_000_000_000)?,
    )?;

    let primary_config = VaultConfig::new(
        primary_controller.public_identity().account,
        primary_controller.public_identity().account,
        asset.id,
        1,
        Amount::zero(),
        Bps::new(10_000)?,
        Digest::from_parts("horizon-vault-salt-v1", &[b"primary"]),
    );
    let reserve_config = VaultConfig::new(
        reserve_controller.public_identity().account,
        primary_controller.public_identity().account,
        asset.id,
        2,
        Amount::new(5_000_000_000)?,
        Bps::new(9_000)?,
        Digest::from_parts("horizon-vault-salt-v1", &[b"reserve"]),
    );
    let primary_vault = primary_config.vault_id;
    let reserve_vault = reserve_config.vault_id;
    ledger.register_vault(primary_config)?;
    ledger.register_vault(reserve_config)?;

    ledger.configure_fee_schedule(FeeSchedule::new(
        primary_controller.public_identity().account,
        Bps::new(500)?,
        Bps::new(250)?,
    )?)?;

    for vault_id in [primary_vault, reserve_vault] {
        ledger.publish_index(IndexObservation::new(
            vault_id,
            asset.id,
            primary_controller.public_identity().account,
            ShareIndex::one(),
            Bps::new(5)?,
            SETTLEMENT_EPOCH,
        )?)?;
    }

    ledger.register_route(SettlementRoute::new(
        primary_vault,
        primary_vault,
        asset.id,
        Amount::new(100_000_000)?,
        Amount::new(20_000_000_000)?,
        Bps::new(100)?,
        2,
        Digest::from_parts("horizon-route-salt-v1", &[b"primary-local"]),
    )?)?;
    ledger.register_route(SettlementRoute::new(
        primary_vault,
        reserve_vault,
        asset.id,
        Amount::new(100_000_000)?,
        Amount::new(20_000_000_000)?,
        Bps::new(100)?,
        2,
        Digest::from_parts("horizon-route-salt-v1", &[b"reserve-route"]),
    )?)?;

    ledger.deposit_vault(
        liquidity_provider.public_identity().account,
        reserve_vault,
        Amount::new(90_000_000_000)?,
    )?;

    Ok(Fixture {
        ledger,
        issuer,
        beneficiary,
        relayer,
        asset,
        primary_vault,
        reserve_vault,
        primary_controller,
    })
}

fn issue_note(
    fixture: &mut Fixture,
    amount: u128,
    label: &'static str,
) -> HorizonResult<(DeliveryNote, TxId)> {
    let issuer = fixture.issuer.public_identity().account;
    let beneficiary = fixture.beneficiary.public_identity().account;
    let order = IngressOrder::new(
        fixture.ledger.network_id(),
        fixture.primary_vault,
        issuer,
        beneficiary,
        fixture.asset.id,
        Amount::new(amount)?,
        fixture.ledger.ingress_nonce(issuer)?,
        SETTLEMENT_EPOCH,
        Digest::from_parts("horizon-test-route-v1", &[label.as_bytes()]),
    )?;
    let note = order.note(ShareIndex::one())?;
    let signed = SignedIngressOrder::sign(order, &fixture.issuer)?;
    let tx = fixture.ledger.issue_note(&signed)?;
    Ok((note, tx))
}

fn settle_note(
    fixture: &mut Fixture,
    note: DeliveryNote,
    payout_vault: VaultId,
) -> HorizonResult<TxId> {
    let beneficiary = fixture.beneficiary.public_identity().account;
    let relayer = fixture.relayer.public_identity().account;
    let ticket = RedemptionTicket::new(
        fixture.ledger.network_id(),
        payout_vault,
        note.note_id,
        beneficiary,
        relayer,
        Amount::new(5_000_000)?,
        fixture.ledger.ticket_nonce(beneficiary)?,
        SETTLEMENT_EPOCH,
        note.digest()?,
    )?;
    let signed = SignedRedemptionTicket::sign(ticket, &fixture.beneficiary)?;
    fixture.ledger.settle_ticket(&signed)
}

#[test]
fn fixture_inicializa_superficie_operativa() {
    let fixture = fixture().expect("fixture should initialize");

    assert_eq!(fixture.ledger.network_id(), NETWORK_ID);
    assert_eq!(fixture.ledger.route_count(), 2);
    assert_eq!(fixture.ledger.observation_count(), 2);
    assert_eq!(fixture.ledger.operator_count(), 5);
    assert_eq!(fixture.ledger.note_count(), 0);
    assert_eq!(fixture.ledger.processed_ticket_count(), 0);
    assert_eq!(fixture.ledger.fee_asset_count(), 0);
    assert!(fixture.ledger.is_conserved(fixture.asset.id).unwrap());
}

#[test]
fn issue_note_bloquea_notional_en_vault_origen() {
    let mut fixture = fixture().expect("fixture should initialize");
    let (note, tx) =
        issue_note(&mut fixture, 2_500_000_000, "issue").expect("note issuance should succeed");
    let vault = fixture.ledger.vault(fixture.primary_vault).unwrap();

    assert_eq!(note.amount, Amount::new(2_500_000_000).unwrap());
    assert_eq!(fixture.ledger.journal().last().unwrap().tx_id, tx);
    assert_eq!(
        fixture
            .ledger
            .balance_of(fixture.issuer.public_identity().account, fixture.asset.id)
            .unwrap(),
        Amount::new(17_500_000_000).unwrap()
    );
    assert_eq!(vault.reserve_balance, Amount::new(2_500_000_000).unwrap());
    assert_eq!(vault.locked_notional, Amount::new(2_500_000_000).unwrap());
    assert_eq!(fixture.ledger.note_count(), 1);
    assert!(fixture.ledger.is_conserved(fixture.asset.id).unwrap());
}

#[test]
fn settle_local_distribuye_valor_y_cierra_notional() {
    let mut fixture = fixture().expect("fixture should initialize");
    let (note, _) =
        issue_note(&mut fixture, 2_500_000_000, "settle").expect("note issuance should succeed");
    let payout_vault = fixture.primary_vault;

    settle_note(&mut fixture, note, payout_vault).expect("local settlement should succeed");

    let vault = fixture.ledger.vault(fixture.primary_vault).unwrap();
    assert_eq!(
        fixture
            .ledger
            .balance_of(
                fixture.beneficiary.public_identity().account,
                fixture.asset.id
            )
            .unwrap(),
        Amount::new(2_495_000_000).unwrap()
    );
    assert_eq!(
        fixture
            .ledger
            .balance_of(fixture.relayer.public_identity().account, fixture.asset.id)
            .unwrap(),
        Amount::new(5_000_000).unwrap()
    );
    assert_eq!(vault.reserve_balance, Amount::zero());
    assert_eq!(vault.locked_notional, Amount::zero());
    assert_eq!(fixture.ledger.processed_ticket_count(), 1);
    assert_eq!(fixture.ledger.fee_asset_count(), 1);
    assert!(fixture.ledger.is_conserved(fixture.asset.id).unwrap());
}

#[test]
fn settle_enrutado_actualiza_vault_de_reserva() {
    let mut fixture = fixture().expect("fixture should initialize");
    let (note, _) =
        issue_note(&mut fixture, 2_500_000_000, "routed").expect("note issuance should succeed");
    let payout_vault = fixture.reserve_vault;

    settle_note(&mut fixture, note, payout_vault).expect("routed settlement should succeed");
    fixture
        .ledger
        .sweep_vault_surplus(
            fixture.primary_controller.public_identity().account,
            fixture.primary_vault,
            Amount::new(2_500_000_000).unwrap(),
        )
        .expect("surplus sweep should succeed");

    let primary = fixture.ledger.vault(fixture.primary_vault).unwrap();
    let reserve = fixture.ledger.vault(fixture.reserve_vault).unwrap();
    assert_eq!(primary.reserve_balance, Amount::zero());
    assert_eq!(primary.locked_notional, Amount::zero());
    assert_eq!(
        reserve.reserve_balance,
        Amount::new(87_500_000_000).unwrap()
    );
    assert_eq!(
        fixture
            .ledger
            .balance_of(
                fixture.primary_controller.public_identity().account,
                fixture.asset.id
            )
            .unwrap(),
        Amount::new(2_500_000_000).unwrap()
    );
    assert!(fixture.ledger.is_conserved(fixture.asset.id).unwrap());
}

#[test]
fn settle_enrutado_usa_indice_actual_del_vault_pagador() {
    let mut fixture = fixture().expect("fixture should initialize");
    let (note, _) =
        issue_note(&mut fixture, 2_500_000_000, "indexed").expect("note issuance should succeed");
    let payout_vault = fixture.reserve_vault;
    fixture
        .ledger
        .publish_index(
            IndexObservation::new(
                fixture.reserve_vault,
                fixture.asset.id,
                fixture.primary_controller.public_identity().account,
                ShareIndex::new(1_200_000_000_000).unwrap(),
                Bps::new(5).unwrap(),
                SETTLEMENT_EPOCH + 1,
            )
            .unwrap(),
        )
        .expect("index update should succeed");

    settle_note(&mut fixture, note, payout_vault).expect("indexed settlement should succeed");

    assert_eq!(
        fixture
            .ledger
            .balance_of(
                fixture.beneficiary.public_identity().account,
                fixture.asset.id
            )
            .unwrap(),
        Amount::new(2_995_000_000).unwrap()
    );
    assert_eq!(
        fixture
            .ledger
            .vault(fixture.reserve_vault)
            .unwrap()
            .reserve_balance,
        Amount::new(87_000_000_000).unwrap()
    );
    assert!(fixture.ledger.is_conserved(fixture.asset.id).unwrap());
}

#[test]
fn ticket_repetido_es_rechazado() {
    let mut fixture = fixture().expect("fixture should initialize");
    let (note, _) =
        issue_note(&mut fixture, 2_500_000_000, "duplicate").expect("note issuance should succeed");
    let beneficiary = fixture.beneficiary.public_identity().account;
    let ticket = RedemptionTicket::new(
        fixture.ledger.network_id(),
        fixture.primary_vault,
        note.note_id,
        beneficiary,
        fixture.relayer.public_identity().account,
        Amount::new(5_000_000).unwrap(),
        fixture.ledger.ticket_nonce(beneficiary).unwrap(),
        SETTLEMENT_EPOCH,
        note.digest().unwrap(),
    )
    .unwrap();
    let signed = SignedRedemptionTicket::sign(ticket, &fixture.beneficiary).unwrap();

    fixture
        .ledger
        .settle_ticket(&signed)
        .expect("first settlement should succeed");
    let error = fixture
        .ledger
        .settle_ticket(&signed)
        .expect_err("second settlement should fail");

    assert_eq!(error, HorizonError::TicketProcessed(ticket.ticket_id));
}

#[test]
fn ticket_antiguo_es_rechazado() {
    let mut fixture = fixture().expect("fixture should initialize");
    let (note, _) =
        issue_note(&mut fixture, 2_500_000_000, "stale").expect("note issuance should succeed");
    let beneficiary = fixture.beneficiary.public_identity().account;
    let ticket = RedemptionTicket::new(
        fixture.ledger.network_id(),
        fixture.primary_vault,
        note.note_id,
        beneficiary,
        fixture.relayer.public_identity().account,
        Amount::new(5_000_000).unwrap(),
        fixture.ledger.ticket_nonce(beneficiary).unwrap(),
        SETTLEMENT_EPOCH - 1,
        note.digest().unwrap(),
    )
    .unwrap();
    let signed = SignedRedemptionTicket::sign(ticket, &fixture.beneficiary).unwrap();
    let error = fixture
        .ledger
        .settle_ticket(&signed)
        .expect_err("stale settlement should fail");

    assert_eq!(error, HorizonError::Policy("note is not mature".to_owned()));
}

fn keyed(byte: u8) -> KeyPair {
    KeyPair::from_seed([byte; 32])
}

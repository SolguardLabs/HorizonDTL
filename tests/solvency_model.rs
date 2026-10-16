use horizon_dtl::{
    Amount, Bps, HorizonError, SolvencyBand, SolvencyInput, SolvencyModel, SolvencyPolicy,
};

fn amount(value: u128) -> Amount {
    Amount::new(value).unwrap()
}

fn policy() -> SolvencyPolicy {
    SolvencyPolicy::new(
        Bps::new(1_000).unwrap(),
        Bps::new(8_000).unwrap(),
        Bps::new(5_000).unwrap(),
        Bps::new(8_000).unwrap(),
        Bps::new(1_000).unwrap(),
        Bps::new(5_000).unwrap(),
        amount(100),
    )
    .unwrap()
}

fn input(payout_reserve: u128, largest_route: u128) -> SolvencyInput {
    SolvencyInput::new(
        amount(payout_reserve),
        amount(500),
        amount(800),
        amount(200),
        amount(100),
        amount(100),
        amount(largest_route),
    )
}

#[test]
fn clasifica_una_reserva_con_holgura_como_normal() {
    let projection = SolvencyModel::evaluate(policy(), input(1_000, 300)).unwrap();

    assert_eq!(projection.band, SolvencyBand::Normal);
    assert_eq!(projection.stressed_payout_reserve, amount(900));
    assert_eq!(projection.eligible_source_reserve, amount(400));
    assert_eq!(projection.eligible_backstop, amount(50));
    assert_eq!(projection.required_liquidity, amount(1_140));
    assert_eq!(projection.target_liquidity, amount(1_254));
    assert_eq!(projection.capital_buffer, amount(210));
    assert_eq!(projection.capital_shortfall, Amount::zero());
    assert_eq!(projection.route_concentration_bps, 2_727);
}

#[test]
fn activa_vigilancia_cuando_no_alcanza_el_buffer_objetivo() {
    let mut guarded_policy = policy();
    guarded_policy.reserve_shock_bps = Bps::new(1_500).unwrap();

    let projection = SolvencyModel::evaluate(guarded_policy, input(900, 300)).unwrap();

    assert_eq!(projection.eligible_liquidity, amount(1_215));
    assert_eq!(projection.band, SolvencyBand::Guarded);
    assert_eq!(projection.capital_shortfall, Amount::zero());
}

#[test]
fn restringe_una_cartera_con_ruta_demasiado_concentrada() {
    let projection = SolvencyModel::evaluate(policy(), input(1_000, 700)).unwrap();

    assert_eq!(projection.band, SolvencyBand::Restricted);
    assert_eq!(projection.route_concentration_bps, 6_363);
}

#[test]
fn declara_critica_una_proyeccion_con_deficit() {
    let projection = SolvencyModel::evaluate(policy(), input(300, 300)).unwrap();

    assert_eq!(projection.band, SolvencyBand::Critical);
    assert_eq!(projection.eligible_liquidity, amount(720));
    assert_eq!(projection.capital_shortfall, amount(420));
    assert_eq!(projection.capital_buffer, Amount::zero());
}

#[test]
fn una_cartera_vacia_conserva_cobertura_neutral() {
    let empty_policy = SolvencyPolicy::new(
        Bps::new(0).unwrap(),
        Bps::new(0).unwrap(),
        Bps::new(0).unwrap(),
        Bps::new(10_000).unwrap(),
        Bps::new(1_000).unwrap(),
        Bps::new(5_000).unwrap(),
        Amount::zero(),
    )
    .unwrap();
    let empty = SolvencyInput::new(
        Amount::zero(),
        Amount::zero(),
        Amount::zero(),
        Amount::zero(),
        Amount::zero(),
        Amount::zero(),
        Amount::zero(),
    );

    let projection = SolvencyModel::evaluate(empty_policy, empty).unwrap();

    assert_eq!(projection.band, SolvencyBand::Normal);
    assert_eq!(projection.coverage_bps, 10_000);
    assert_eq!(projection.route_concentration_bps, 0);
}

#[test]
fn rechaza_una_ruta_mayor_que_las_obligaciones() {
    let error = SolvencyModel::evaluate(policy(), input(1_000, 1_101)).unwrap_err();

    assert_eq!(
        error,
        HorizonError::Policy("largest route exceeds total obligations".to_owned())
    );
}

#[test]
fn rechaza_un_limite_de_concentracion_nulo() {
    let error = SolvencyPolicy::new(
        Bps::new(0).unwrap(),
        Bps::new(0).unwrap(),
        Bps::new(0).unwrap(),
        Bps::new(0).unwrap(),
        Bps::new(0).unwrap(),
        Bps::new(0).unwrap(),
        Amount::zero(),
    )
    .unwrap_err();

    assert_eq!(
        error,
        HorizonError::Policy("route concentration limit must be positive".to_owned())
    );
}

#[test]
fn falla_cerrado_si_la_suma_de_liquidez_desborda() {
    let overflow_policy = SolvencyPolicy::new(
        Bps::new(0).unwrap(),
        Bps::new(10_000).unwrap(),
        Bps::new(0).unwrap(),
        Bps::new(0).unwrap(),
        Bps::new(0).unwrap(),
        Bps::new(10_000).unwrap(),
        Amount::zero(),
    )
    .unwrap();
    let overflow_input = SolvencyInput::new(
        amount(u128::MAX),
        amount(1),
        Amount::zero(),
        Amount::zero(),
        Amount::zero(),
        Amount::zero(),
        Amount::zero(),
    );

    assert_eq!(
        SolvencyModel::evaluate(overflow_policy, overflow_input).unwrap_err(),
        HorizonError::AmountOverflow
    );
}

use std::ops::Div;

use ark_ff::{FftField, to_bytes};
use ark_poly::{univariate::DensePolynomial, UVPolynomial, EvaluationDomain, evaluations::univariate::Evaluations};
use ark_poly_commit::{LabeledPolynomial, PolynomialCommitment, LabeledCommitment, QuerySet, evaluate_query_set};
use ark_std::rand::RngCore;

use ark_std::vec::Vec;

use crate::{
    data_structures::{Proof, Statement},
    error::Error,
    rng::FiatShamirRng,
    PROTOCOL_NAME,
};

pub fn prove<
    F: FftField,
    PC: PolynomialCommitment<F, DensePolynomial<F>>,
    FS: FiatShamirRng,
    R: RngCore,
>(
    ck: &PC::CommitterKey,
    statement: &Statement<F, PC>,
    f: &LabeledPolynomial<F, DensePolynomial<F>>,
    f_rand: &PC::Randomness,
    rng: &mut R,
) -> Result<Proof<F, PC>, Error<PC::Error>> {

    // Part 1 - finding the adversarial polynomials

    // Basic idea: put s to be some random polynomial diffent from f with the same sum, then
    // perform the univariant sum-check for p=f+s. 
    // I believe randomness is important so that the verifier can't tell what we are doing by running it multiple times.
    
    let seed =  b"GEOMETRY-SUMCHECK, double spend attack";

    let mut s_rng = FS::initialize(&to_bytes![&seed, statement].unwrap());

    let mut evaluations = Vec::new();
    let mut sum = F::zero();
    for h in statement.domain.elements() {
        let random = F::rand(&mut s_rng);
        sum = sum + random;
        evaluations.push(-f.evaluate(&h) + random);
    }
    evaluations[0] = evaluations[0] - sum;
    
    let evals = Evaluations::from_vec_and_domain(evaluations, statement.domain);
    
    
    let s = evals.interpolate(); // interpolate shuffled values of f

    let p =  f.polynomial().clone() + s.clone();
    let (h, r) = p.divide_by_vanishing_poly(statement.domain).unwrap();
    let x =  DensePolynomial::from_coefficients_vec(vec![F::zero(), F::one()]);
    let g = r.div(&x);


    // Part 2 - Preparing the proof

    let g = LabeledPolynomial::new("g".into(), g.clone(), Some(statement.domain.size() - 2), Some(1));
    let h = LabeledPolynomial::new("h".into(), h.clone(), None, Some(1));
    let s = LabeledPolynomial::new("s".into(), s.clone(), None, Some(1));

    let (g_commitment, g_rand) = PC::commit(&ck, &[g.clone()], Some(rng)).unwrap(); 
    let (h_commitment, h_rand) = PC::commit(&ck, &[h.clone()], Some(rng)).unwrap(); 
    let (s_commitment, s_rand) = PC::commit(&ck, &[s.clone()], Some(rng)).unwrap(); 

    let f_c = LabeledCommitment::new("f".into(), statement.f.clone(), None);
    let s_c = LabeledCommitment::new("s".into(), s_commitment[0].commitment().clone(), None);
    let h_c = LabeledCommitment::new("h".into(), h_commitment[0].commitment().clone(), None);
    let g_c = LabeledCommitment::new(
        "g".into(),
        g_commitment[0].commitment().clone(),
        Some(statement.domain.size() - 2),
    );

    // Initialized Fiat-Shamir values
    let mut fs_rng = FS::initialize(&to_bytes![&PROTOCOL_NAME, statement].unwrap());

    fs_rng.absorb(&to_bytes![
        s_c.commitment().clone(), 
        h_c.commitment().clone(), 
        g_c.commitment().clone()].unwrap());

    let xi = F::rand(&mut fs_rng);
    let opening_challenge = F::rand(&mut fs_rng);

    // Make quary set and proof

    let point_label = String::from("xi");
    let query_set = QuerySet::from([
        ("f".into(), (point_label.clone(), xi)),
        ("h".into(), (point_label.clone(), xi)),
        ("g".into(), (point_label.clone(), xi)),
        ("s".into(), (point_label, xi)),
    ]);

    let evaluations = evaluate_query_set(
        [f, &h, &g, &s],
        &query_set,
    );
    
    let pc_proof = PC::batch_open(
        ck, 
        [f, &h, &g, &s],
        [&f_c, &h_c, &g_c, &s_c],
        &query_set,
        opening_challenge,
        [f_rand, &h_rand[0], &g_rand[0], &s_rand[0]],
        Some(rng),
    ).unwrap();

    Ok(Proof{
        f_opening : evaluations[&("f".into(), xi)],
        s : s_c.commitment().clone(),
       s_opening : evaluations[&("s".into(), xi)],
        g : g_c.commitment().clone(),
        g_opening :evaluations[&("g".into(), xi)],
        h: h_c.commitment().clone(),
       h_opening : evaluations[&("h".into(), xi)],
        pc_proof : pc_proof,
    })
}

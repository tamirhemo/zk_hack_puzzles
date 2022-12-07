use ark_bls12_cheon::{Fr, G1Projective as G1, G2Projective as G2};
use ark_ff::Field;
use crate::utils::{pow_sp, pow_sp2, bigInt_to_u128};

// Imports for pairing
use ark_bls12_cheon::Bls12Cheon;
use ark_ec::pairing::{Pairing, PairingOutput};


const A: u64 = 1089478584172543;
const B : u64 = 1089547303649280;
const D : u32 = 702047372;
const QD :u64 = 1587011986761171; //(q-1)/D
const FACTOR :u64 = 262144; // floor(sqrt(n))
const FACTORONE : u64 = 26497; // floor(sqrt(D))+1

/// A baby-step giant-step calculation
/// Contains commented print statements indicating progress. 
fn baby_steps_giant_steps<F: Field>(g_d : F, g : F, eta_1: Fr, factor: u64, m: u64) -> Option<(u64, u64)> {
    let eta_1_inv = eta_1.inverse().unwrap();
    let eta_1_factor = pow_sp(eta_1, factor.into(), 64);

    let mut lookup_baby_steps = pow_sp2(g_d, eta_1_inv, m+1);
    lookup_baby_steps.insert(g_d, 0);
    let mut lookup_giant_steps = pow_sp2(g, eta_1_factor, m+1);
    lookup_giant_steps.insert(g, 0);

    //println!("Tables collected");
    
    let mut u_log = None;
    let mut v_log = None;
    for (e_v, v) in lookup_giant_steps {
        u_log = lookup_baby_steps.get(&e_v);
        if u_log.is_some() {
            v_log = Some(v);
            //println!("Succcess!");
            break;
        }
    }
    match (u_log, v_log) {
        (Some(u), Some(v)) => Some((*u, v)),
        _=> None,
    }
}


pub fn attack(P: G1, tau_P: G1, tau_d1_P: G1, Q: G2, tau_d2_Q: G2) -> i128 {
    //compute e((tau^d_1)P, (tau^d_2)Q) = tau^d*e(P,Q)
    let e  = Bls12Cheon::pairing(P, Q);
    let tau_d_e = Bls12Cheon::pairing(tau_d1_P, tau_d2_Q);
    let tau_e = Bls12Cheon::pairing(tau_P, Q);
    
    // Some powers of two
    let two = Fr::from(2);
    let two_d = pow_sp(two, D.into(), 32);
    let two_d_inv = two_d.inverse().unwrap();
    let two_d_A_inv = pow_sp(two_d_inv, A.into(), 64);

    let e_d_A = tau_d_e*two_d_A_inv;

    // Write k_0 = A + u + factor*v
    // Run baby-steps big-steps algorithm with respect to u and v
    let n = B-A;
    let factor = FACTOR; // floor(sqrt(n))
    let m = n/factor;


    let (u,v) = baby_steps_giant_steps(e_d_A.0, e.0, two_d, factor, m).unwrap();
    let k_0 = A + u + factor*v;
    let exp = pow_sp(two_d, k_0.into(), 64);
    let two_k_0 = pow_sp(two, k_0.into(), 64);

    // Sanity
    let e_d_k_0 = e*exp;
    assert_eq!(tau_d_e, e_d_k_0);

    let two_inv = two.inverse().unwrap();
    let two_k_0_inv = pow_sp(two_inv, k_0.into(), 64);
    let e_1 = tau_e*two_k_0_inv;

    let q_d: u64 = QD; // (q-1)/D
    let two_q_d = pow_sp(two, q_d.into(), 64);

    // Run the second baby-step big-step

    let factor_1: u64 = FACTORONE; // floor(sqrt(D))+1
    let m_1 = (D as u64)/factor_1;

    let (u_1,v_1) = baby_steps_giant_steps(e_1.0, e.0, two_q_d, factor_1, m_1).unwrap();

    let k_1 = u_1 + factor_1 *v_1;

    let two_dq_k_1 = pow_sp(two_q_d, k_1.into(), 64);
    assert_eq!(e_1, e*two_dq_k_1);

    // Multiply all factors

    let tau = two_k_0*two_dq_k_1;
    //assert_eq!(tau_e, e*(two_k_0*two_dq_k_1));
    //assert_eq!(tau_P, P*tau);
    //println!("Done calculating");

    let tau_128 = bigInt_to_u128(tau.into());
    tau_128.try_into().unwrap()
}

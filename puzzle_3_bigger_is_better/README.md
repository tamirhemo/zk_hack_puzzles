# Bigger is Better
## Introduction
Bob was catching up on the latest in zkSNARK research, and came across the
Vampire paper [1]. In that paper, he found a reference to an inner-product
commitment scheme [2], which allows committing to a vector and later proving
that its inner-product with another (public) vector is equal to a claimed value.
Bob was intrigued by this scheme, and decided to implement it in Rust.

Bob was delighted with the performance of the resulting implementation, and so
decided to deploy it. The scheme requires a universal Powers-of-Tau-type trusted setup,
and so Bob generated a SRS using an MPC ceremony.

Things were going smoothly for a while, but then Bob received an anonymous email that
contained a full break of the scheme! Unfortunately for Bob, the email didn't contain
any details about the break. Can you help Bob figure out the issue, and fix his scheme?

[1]: https://ia.cr/2022/406
[2]: http://www.lsv.fr/Publis/PAPERS/PDF/ILV-imacc11-long.pdf



## Pairing engine
[Arkworks](https://github.com/arkworks-rs/) provides a convenient interface to interact with the data involving elliptic curves and pairings through the [`PairingEngine`](https://docs.rs/ark-ec/latest/ark_ec/trait.PairingEngine.html) trait in the [`ark_ec`](https://docs.rs/ark-ec/latest/ark_ec/index.html#) crate  (note that in the current 0.4 version of arc_ec it is organized under [pairing module](https://docs.rs/ark-ec/0.4.0-alpha.4/ark_ec/pairing/index.html) . The trait looks like this:
```rust
pub trait PairingEngine: Sized + 'static + Copy + Debug + Sync + Send + Eq + PartialEq {
    type Fr: PrimeField + SquareRootField;
    type G1Projective: ProjectiveCurve<BaseField = Self::Fq, ScalarField = Self::Fr, Affine = Self::G1Affine> + From<Self::G1Affine> + Into<Self::G1Affine> + MulAssign<Self::Fr>;
    type G1Affine: AffineCurve<BaseField = Self::Fq, ScalarField = Self::Fr, Projective = Self::G1Projective> + From<Self::G1Projective> + Into<Self::G1Projective> + Into<Self::G1Prepared>;
    type G1Prepared: ToBytes + Default + Clone + Send + Sync + Debug + From<Self::G1Affine>;
    type G2Projective: ProjectiveCurve<BaseField = Self::Fqe, ScalarField = Self::Fr, Affine = Self::G2Affine> + From<Self::G2Affine> + Into<Self::G2Affine> + MulAssign<Self::Fr>;
    type G2Affine: AffineCurve<BaseField = Self::Fqe, ScalarField = Self::Fr, Projective = Self::G2Projective> + From<Self::G2Projective> + Into<Self::G2Projective> + Into<Self::G2Prepared>;
    type G2Prepared: ToBytes + Default + Clone + Send + Sync + Debug + From<Self::G2Affine>;
    type Fq: PrimeField + SquareRootField;
    type Fqe: SquareRootField;
    type Fqk: Field;

   fn miller_loop<'a, I>(i: I) -> Self::Fqk
    where
        I: IntoIterator<Item = &'a (Self::G1Prepared, Self::G2Prepared)>;
    
    fn final_exponentiation(_: &Self::Fqk) -> Option<Self::Fqk>;

    
    fn product_of_pairings<'a, I>(i: I) -> Self::Fqk
    where
        I: IntoIterator<Item = &'a (Self::G1Prepared, Self::G2Prepared)>,
    { ... }
    
    fn pairing<G1, G2>(p: G1, q: G2) -> Self::Fqk
    where
        G1: Into<Self::G1Affine>,
        G2: Into<Self::G2Affine>,
    { ... }
}
```
For a type to implement this trait, we have to provide two groups G_1 and G_2 which will be the pairing groups, a scalar field `Fr`, and the pairing function. 
```rust
fn pairing<G1, G2>(p: G1, q: G2) -> Self::Fqk
```
whose target lies in the field $\mathbb{F_{q^k}}$ where $k$ is the embedding degree. 

This trait makes it convenient to work with this data as we can define all the functions with respect to one trait bound `E::PairingEngine` which gives us access to all the coefficient fields, groups, and the pairing function. 

Another advantage of having such a trait is having some static checks of compatibility, for example, it requires `G1Affine` (similarly for G_2) to have `Fr` as the scalar field, so every time we use a type `E` implementing this trait it is guaranteed that `E::Fr` is the scalar field of `E::G1Affine`. 


## Inner product commitement schemes

As the puzzle description suggests, an inner product commitment scheme allows the prover to commit to a vector **v**
and later prove that inner product of the hidden vector with another public vector **u** is equal to some specific value. 

Bob has implemented the commitment scheme from [2] which is based on polynomial commitments. We will review the algorithm and Bob's implementation of it. The commitment is generically defined for any type `E` implementing `PairingEngine`. The vectors we are commiting to will be over the scalar field `E::Fr` of the two groups.

### Setup (key generation):
The commitment key is generated from the structured reference string (SRS) Bob obtained from the MPC ceremony 
```rust
    let ck = data_structures::CommitmentKey::<Bls12_381>::deserialize_unchecked(SRS).unwrap();
```
It contains the following data: 
```rust
pub struct CommitmentKey<E: PairingEngine> {
    /// The powers [beta^0 * G, beta^1 * G, ..., beta^n * G]
    pub powers_of_beta_g_first: Vec<E::G1Affine>,
    /// The powers [beta^{n + 2} * G,  ..., beta^{2n} * G]
    pub powers_of_beta_g_second: Vec<E::G1Affine>,
    /// The powers [beta^0 * H, beta^1 * H, ..., beta^n * H]
    pub powers_of_beta_h: Vec<E::G2Affine>,
}
```
Namely, the powers beta^i * G for values of i from 0 to 2n **except** n+1, and the powers beta^j H for j between 1 and n. The integer n will be the supported dimension, namely, the commitment key `ck` will enable to commit to n dimensional vectors over `E::Fr`.

### Commitment
The commitment algorithm:
```rust 
    pub fn commit(ck: &CommitmentKey<E>, input: &[E::Fr]) -> Commitment<E> 
```
Takes a commitement key and an input vector and outputs a commitment, which is just an element of G_1, that is, `Commitment<E> ` is defined as:
```
pub struct Commitment<E: PairingEngine>(pub E::G1Affine);
```
The commitment is calculated by taking a vector `input` with components `input_i` and giving the element 
```math
commit(\mathrm{input}) = \sum_{i=1}^{n} input_i \beta^{i}*G$
```
We can compute the sum of these multiplication more efficiently using the [`VariableBaseMSM::multi_scalar_mul`](https://docs.rs/ark-ec/latest/ark_ec/msm/struct.VariableBaseMSM.html#method.multi_scalar_mul) method from the [`ark_ec`](https://docs.rs/ark-ec/latest/ark_ec/index.html#) crate: 
```rust
    pub fn commit(ck: &CommitmentKey<E>, input: &[E::Fr]) -> Commitment<E> {
        let input_ints = input.iter().map(|x| x.into_repr()).collect::<Vec<_>>();
        Commitment(
            VariableBaseMSM::multi_scalar_mul(&ck.powers_of_beta_g_first[1..], &input_ints).into(),
        )
    }

```

### Opening
Given a commitement, the original vector `a` and a public vector `b`, the proof algorithm produces a proof of the evaluation of the scalar product of `a` and `b`:
```rust 
pub fn open(ck: &CommitmentKey<E>, a: &[E::Fr], b: &[E::Fr]) -> Proof<E>
```
where a proof is again just an element of G_1, i.e.,
```rust
pub struct Proof<E: PairingEngine>(pub E::G1Affine);
```
It is produced from a polynomial as follows. Set:
```math
a(X) = \sum_{i=1}^{n} a_i X^i , \quad b^*(X) = \sum_{j=1}^{n} b_j X^{N+1-j}, v = b^Ta = \sum_{i} b_i a_i 
```
so that $v$ is the inner product. We then consider the polynomial:
```
\mu_{ipc}(X) = b^*(X)a(X) - vX^{n+1} 
```
Since $v$ is exactly the inner product, the (n+1) coefficient of $\mu_{ipc}(X)$ is zero. We can then send as a proof a commitment to the coefficient of $\mu_{ipc}$ by:
```math
\mathrm{proof}(ck, a, b) = \sum_{i=1, i\neq n+1}^{2n} \mu_{ipc, i} \cdot \beta^{i}*G. 
```
To implement this, we first compute the inner product:
```rust 
        let dim = a.len();
        let inner_product = a.iter().zip(b.iter()).map(|(&a, b)| a * b).sum::<E::Fr>();
```
We then generate the polynomial $a(X)$:
        let mut a_coeffs = Vec::with_capacity(a.len() + 1);
        a_coeffs.push(E::Fr::zero());
        a_coeffs.extend_from_slice(a);
        let a_poly = DensePolynomial::from_coefficients_vec(a_coeffs);
        assert_eq!(a_poly.degree(), dim);
        assert_eq!(a_poly.coeffs[0], E::Fr::zero());
```
We then generate the polynomial $b^*(X)$ by inverting the order of the elements to account for the shift in powers in the definition of $b^*$:
```rust
        let mut b_rev = b.to_vec();
        b_rev.push(E::Fr::zero());
        b_rev.reverse();
        let b_poly = DensePolynomial::from_coefficients_vec(b_rev);
        assert_eq!(b_poly.degree(), dim);
        assert_eq!(b_poly.coeffs[0], E::Fr::zero());
        assert_eq!(a_poly.degree(), b_poly.degree());
```
We can now compute the product and subtract the (n+1)-coefficent:
```rust
        let mut product = &a_poly * &b_poly;
        assert_eq!(product.coeffs[0], E::Fr::zero());
        assert_eq!(product.coeffs[dim + 1], inner_product);
        assert_eq!(product.degree(), 2 * dim);

        product.coeffs[dim + 1] -= inner_product;

        let product_coeffs = product
            .coeffs
            .iter()
            .map(|x| x.into_repr())
            .collect::<Vec<_>>();
```
Finally, we can compute the proof, note that we have to compute it in two parts because of the missing coecfficient in dimension n+1:
```rust
        // We have to compute the proof piece wise since the (dim + 1)-th coefficient
        // of the product is zero.
        let first_part = VariableBaseMSM::multi_scalar_mul(
            &ck.powers_of_beta_g_first,
            &product_coeffs[..(dim + 1)],
        );
        let second_part = VariableBaseMSM::multi_scalar_mul(
            &ck.powers_of_beta_g_second,
            &product_coeffs[(dim + 2)..],
        );
        let proof = first_part + second_part;
        Proof(proof.into())
    }
```

### Verification
Given the commitment key, a commitment, a public vector `b`, and a proof that the inner product of the commited vector with `b` is equal to the `claimed_inner_product`, we can verify the proof:
```rust
    pub fn verify(
        ck: &CommitmentKey<E>,
        cm: &Commitment<E>,
        b: &[E::Fr],
        claimed_inner_product: E::Fr,
        proof: &Proof<E>,
    ) -> bool 
```
Sending true/false on accept/reject. The verification exploits the fact that the pairing allows us to obtain multiplications which are otherwise interactable in each particular group. Namely, the verifier wants to check the equality
```math
\mu_{ipc}(X) = b^*(X)a(X) - v * X^{n+1}
```
where $v$ is the `claimed_inner_product`. This requires access to the (n+1)-power that is not provided in the commitment key data of either group. We can utilize the pairing to instead check that
```math
e(\mu_{ipc}(\beta)*G, H) = e(a(\beta)*G, b(\beta)*H) - v\cdot e( \beta^{n} * G, \beta*H)
```
which is equivalent to:
```math
\mu_{ipc}(\beta)* e(G,H) = (a(\beta)b^*(\beta) - v\beta^{n+1}) * e(G,H)
```
which has a high probablity of being correct if and only if the proof is valid. Note that the sides of the equation are just given by
```math
\mathrm{Proof}(ck, a, b) = \mu_{ipc}(\beta)*G, \quad \mathrm{commit}(ck, a) = a(\beta)*G
```
so the verifier only needs to calculate the commitment to $b^*$ and the pairings. Let's see how this is implemented in the puzzle code. First, the verifier computes the commitment to $b$:
    {
        let dim = b.len();

        let mut b_rev = b.to_vec();
        b_rev.push(E::Fr::zero());
        b_rev.reverse();
        let b_rev = b_rev.iter().map(|x| x.into_repr()).collect::<Vec<_>>();
        assert_eq!(b_rev.len(), ck.powers_of_beta_h.len());
        let b_comm = VariableBaseMSM::multi_scalar_mul(&ck.powers_of_beta_h, &b_rev).into();
```
And then simply calculates the relevant pairings and compares the term 
```rust
        let e1 = E::pairing(proof.0, ck.powers_of_beta_h[0]);
        let e2 = E::pairing(cm.0, b_comm);
        let e3 = E::pairing(
            ck.powers_of_beta_g_first[dim].mul(claimed_inner_product),
            ck.powers_of_beta_h[1],
        );
        e1 * e3 == e2
    }
```
note that the group action in the target of the pairing is written multiplicatively and also that we multiply by `e3` on the left hand side instead of multiplying by the inverse in the right hand side as in the formula. 

### Security
The security of the protocol relies heavily on the fact that there is no way to extract the element $G^{\beta^{n+1}}$ given all the other powers $G^i$ for all other values of $i$ between 0 and $2n$. This is known as the *n-Diffie-Hellman Exponent problem*,  the security of it is discussed in [3] and in Appendix F of [2]

[3]: https://eprint.iacr.org/2005/018.pdf

## Puzzle Solution 
### Bob's fatal mistake - the puncture
This all looks pretty good, right? so what did Bob do wrong? well, it all comes down to the key extraction:
```rust
    let ck = data_structures::CommitmentKey::<Bls12_381>::deserialize_unchecked(SRS).unwrap();
```
Bob just used the SRS as is from the cerimony and didn't perform any verification that it doesn't contain any backdoors. If we check, it turns out that we have more elements of G_1 than we thought:
```rust
    assert_eq(ck.powers_of_beta_g_first.len(), dim+1)
```
This means we might have access to \beta^{n+1} *G! not looking so good for Bob.

### The attack
Let's see how to exploit this elemenet. The basic idea is that having \beta^{n+1} *G enables us to produce a proof polynomial with a non-zero coefficient in degree n+1, so we can set this degree to anything we want. Namely, if we denote by $v$ the real inner product and let's say we want to convince the verifier that the real inner product is 0, then we will send a commitment to the polynomial
```math
\mu_{\mathrm{fake}}(X) = b^*(X)a(X) = \mu_{ipc}(X) + v\cdot X^{n+1}
```
which we can do by taking a sound proof and shifting the (n+1) degree
```math
\mu_{\mathrm{fake}}(\beta)*G = \mathrm{Proof}(ck, a, b) + v\cdot X^{n+1}
```
Let's see how to implement this. The puzzle asks us to implement the following attack function:

```rust
pub fn attack<E: ark_ec::PairingEngine>(ck: &CommitmentKey<E>, dim: usize) -> attack::Attack<E>
```
whose output is a struct of the form:
```rust
pub struct Attack<E: PairingEngine> {
    /// The vector that will be committed.
    pub a: Vec<E::Fr>,
    /// Commitment to `a`.
    pub commitment: Commitment<E>,
    /// The claimed inner product of `a` and `b := Hash(commitment)`, which differs
    /// from the actual inner product.
    pub claimed_inner_product: E::Fr,
    /// An unsound proof that `a` and `b` have inner product `claimed_inner_product`.
    pub proof: Proof<E>,
}
```
namely, `b` is computed by hashing the commitment, so that we need to supply a proof that works for any `a` and `b`. To demonstrate, we will take a random `a` and generate a commitment to it. We then set `b` to be the hash of `a` using the hash supplied in [`attack.rs`](https://github.com/ZK-Hack/puzzle-bigger-is-better/blob/2626e55083ed5ea7a35c17c0102bf961d1275fd6/src/attack.rs#L45)
```rust
    let mut rng = ark_std::test_rng();
    let a = (0..dim).map(|_| E::Fr::rand(&mut rng)).collect::<Vec<_>>();
    let commitment = ILV::commit(ck, &a);
        let b = attack::hash(commitment, dim);
```
We then generate a valid proof for the actual inner product of `a` and `b` and compute the inner product:
```rust 
    // Get a real proof for the real inner product
    let actual_proof = ILV::open(ck, &a, &b);
    let actual_inner_product = a.iter().zip(b.iter()).map(|(&a, b)| a * b).sum::<E::Fr>();
```
We then shift the prolynomial by that inner product in degree (n+1) and set it to be the proof
```rust
    let attack_shift = g_beta_dim_plus_one.mul(actual_inner_product);
    let new_polyomial = actual_proof.0 + attack_shift.into();
    let proof = Proof(new_polyomial);
```
Finally, we return the attack:
```rust
    attack::Attack {
        a,
        commitment,
        claimed_inner_product: E::Fr::zero(),
        proof: proof,
    }
}
```
and we are done!
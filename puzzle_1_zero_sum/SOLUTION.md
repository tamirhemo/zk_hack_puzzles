# Zero Sum Game
## Introduction
Bob designed a private payment protocol based on summation of polynomial evaluations. A new note is issued with a secret
polynomial $f\in \mathbb{F}[X]$ known to the owner of the note and a public subset $H\subseteq \mathbb{F}$ such that the evaluations of $f$ over $H$ sum to zero. Once the note is spent, the polynomial $f$ is changed to one whose sum over $H$ is different from zero. 

In order to enforce fair bahavior, it should be possible for all participants of the protocol to check the validity of all transactions. This is done by requiring every note to attach the domain $H$, a commitment to the polynomial $f$, and a proof that $\sum_{a\in H} f(a) = 0$. After a note is spent, the statement changes to a new non-zero sum. 

## The univariate sum-check protocol 
The sum-check protocol for univariate polynomials is used to give an efficient proof of the relation $\sum_{a\in H} f(a) = 0$ for a special kind of subsets of $\mathbb{F}$ called *multiplicative subgroups* [^1]. A multiplicative subgroup of $\mathbb{F}$ is a subset $M$ of $\mathbb{F}$ that is closed under multiplication and inversion. That is, if $a\in M$ and $b\in M$ then $ab\in M$ and $a^{-1}\in M$. Note that this implies that $1\in M$. 

[^1]: The protocol works in the same way for the slightly more general multiplicative cosets, but it's not necessary for this review. 

We will review the basic constructions and facts that enable an efficient proof in the univariate sum-check protocol. For full details see section 10.3.1 of [Thaler's book](https://people.cs.georgetown.edu/jthaler/ProofsArgsAndZK.pdf). For any subset $H\subseteq \mathbb{F}$ we denote by $Z_H(X)$ the *vanishing polyniomial* of $H$ defined by
```math
Z_H(X) = \prod_{a\in H} (X-a) 
```
It is a polynomial of degree $n = |H|$, and the lowest degree polynomial which vanishes on all elements of $H$. In fact, the polynomial $Z_H$ divides every polynomial that vanishes on $H$. Namely, if $p\in \mathbb{F}[X]$ vanishes on $H$, then there exists some $q\in \mathbb{F}[X]$ such that $p(X) = Z_H(X)q(X)$. To see this, note that if $f$ vanishes on a point $a\in \mathbb{F}$ then $(X-a)$ divides $f$. Conversely, if such a $q$ exists then for any $a\in H$, we have $p(a) = Z_H(a)q(a) = 0 \cdot q(a) = 0$. Thus, $p$ vanishes on $H$ **if and only if** there exists a polynomial $q$ such that $p(X) = Z_H(X)q(X)$.

This fact give us an efficient way to prove that a polynomial $p$ vanishes on $H$. Namely, it is enough to prove the existance of a $q\in \mathbb{F}[X]$ such that the relation $p(X) = Z_H(X)q(X)$ holds. This can be done without revealing $p$ by supplying commitments to $p$ and $q$, and openning both commitments at the same point $r\in \mathbb{F}$ supplied by a verifier, which enables the verifier to then check $p(r) = Z_H(r)q(r)$ and accept if this equality holds. This is significatly less costly than openning the commitment for $p$ at each one of the points of $H$, especially when the cardinality of $H$ is large. 

The univariate sum-check protocol is based on a similar idea. In the case $H$ is a multiplicative subgroup of $\mathbb{F}$, there exists a simple characterization for the property $\sum_{a\in H}f(a) = 0$. This is based on the following fact. 

**Fact 1**: If $H\subseteq \mathbb{F}$ be a multiplicative subgroup of size $|H| = n$. Then for any polynomial $f\in \mathbb{F}[X]$, the relation $\sum_{a\in H} f(a) = 0$ holds if and only if there exists polynomials $h$, $g$ with degrees bounded by $\mathrm{deg}(h)\leq d-n$ and $\mathrm{deg}(g) \leq n-1$, such that
```math
f(X) = h(X)\cdot \mathbb{Z}_H(X) + X\cdot g(X).
```
See Lemma 10.2. in [Thaler's book](https://people.cs.georgetown.edu/jthaler/ProofsArgsAndZK.pdf) for a proof. The proof relies on the special structure of multiplicative subgroups of finite fields. 

Given Fact 1, we can give a simple proof of a polynomial $f$ having zero sum on $H$. We need to supply commitments to polynomials $g$, $h$, and an opening of the commitments at a point $r\in \mathbb{F}$ given by the verifier. The verifier will then check that $f(r) = h(r)Z_H(r) + rg(r)$.  

## How it all fits in code
In this callenge, we are given a polynomial $f$ and a multiplicative domain $H$ such that $\sum_{a\in H}f(a) \neq 0$. Our goal is to come up with a fitctitous proof that convinces the verifier that $\sum_{a\in H}f(a) = 0$. This is enabled by a vulnerability in the mechanism desinged by Bob to make the sum of the polynomial $f$ secret, which is meant to keep the protocol participants anonymous. Namely, instead of commiting just to the polynomial $f$, Bob's protocol allows the prover to use a secret polynomial $s$, and providing a sum-check proof for $p = f+s$. The main point here is that Bob forgot to limit our choice of $s$ and we can use that to convince our verifier in any sum we want.

To see how to exploit this weekness, let's see how the univariate sum-check protocol is implemented in our particular problem. The challege uses the [arkworks](https://github.com/arkworks-rs/), in particular the [ark_poly](https://docs.rs/ark-poly/latest/ark_poly/) and [ark_poly_commit](https://docs.rs/ark-poly-commit/latest/ark_poly_commit/) crates which contain primitives and implementations for working with polynomials over finite fields and polynomial commitments. 


### The protocol in code
The protocol begins by specifying a (multiplicative) domain using the [`GeneralEvaluationDomain`](https://docs.rs/ark-poly/latest/ark_poly/domain/general/enum.GeneralEvaluationDomain.html) type from the [ark_poly](https://docs.rs/ark-poly/latest/ark_poly/index.html) crate.
```rust
    let domain_size = 16;
    let domain = GeneralEvaluationDomain::new(domain_size).unwrap();
    let max_degree = 30;
```
This challenge uses the [Marlin implementation](https://docs.rs/ark-poly-commit/latest/ark_poly_commit/marlin/marlin_pc/struct.MarlinKZG10.html) of the [KZG polynomial commitment scheme](https://www.iacr.org/archive/asiacrypt2010/6477178/6477178.pdf) which enables us to enforce degree bounds. 

First, the protocol sets up the public keys for the prover and verifier:
```rust
    let mut rng = test_rng();
    let srs = PC::setup(max_degree, None, &mut rng).unwrap();

    let (ck, vk) = PC::trim(&srs, max_degree, 1, Some(&[domain_size - 2])).unwrap();
 ```
 
 We are then given a specific polynomial by the challenge 
 ```rust
    let coeffs = vec![
        F::from(123312u64),
        F::from(124151231u64),
        F::from(1190283019u64),
        F::from(19312315u64),
        F::from(312423151u64),
        F::from(61298741u64),
        F::from(132151231u64),
        F::from(1321512314u64),
        F::from(721315123151u64),
        F::from(783749123u64),
        F::from(2135123151u64),
        F::from(312512314u64),
        F::from(23194890182314u64),
        F::from(321514231512u64),
        F::from(321451231512u64),
        F::from(823897129831u64),
        F::from(908241231u64),
        F::from(9837249823u64),
        F::from(982398741823u64),
        F::from(3891748912u64),
        F::from(21389749812u64),
        F::from(891724876431u64),
        F::from(213145213u64),
        F::from(32897498123u64),
        F::from(3219851289231u64),
        F::from(2184718923u64),
        F::from(31245123131431u64),
        F::from(36712398759812u64),
        F::from(8724876123u64),
        F::from(89783927412u64),
        F::from(8723498123u64),
    ];
    let f = DensePolynomial::from_coefficients_slice(&coeffs);
```

We can even check that the sum is really not zero:
```rust
    let mut real_sum = F::zero();
    for h in domain.elements() {
        real_sum += f.evaluate(&h);
    }
    assert_ne!(real_sum, F::zero());
```

The false statement is made by generating a commitment to the given $f$ and claiming that the sum over `domain` is zero.
```rust
    let sum = F::zero();

    let f = LabeledPolynomial::new("f".into(), f.clone(), None, Some(1));
    let (f_commitment, f_rand) = PC::commit(&ck, &[f.clone()], Some(&mut rng)).unwrap();

    let statement = Statement {
        domain,
        f: f_commitment[0].commitment().clone(),
        sum,
    };

    let proof = prove::<F, PC, FS, StdRng>(&ck, &statement, &f, &f_rand[0], &mut rng).unwrap();
```

The output of the prove method is an instance of the [`Proof`](https://github.com/ZK-Hack/puzzle-zero-sum-game/blob/main/src/data_structures.rs) struct, which is defined as follows:

```rust
    pub struct Proof<F: Field, PC: PolynomialCommitment<F, DensePolynomial<F>>> {
        pub f_opening: F,
        pub s: PC::Commitment,
        pub s_opening: F,
        pub g: PC::Commitment,
        pub g_opening: F,
        pub h: PC::Commitment,
        pub h_opening: F,
        pub pc_proof: PC::BatchProof,
    }
```
The statement is accepted if the verifier accepts the proof. 
```rust
    let res = verify::<F, PC, FS, StdRng>(&vk, &statement, &proof, &mut rng);
    assert_eq!(true, res.is_ok());
```
Our goal is to implement a prover that convinces the verifier of the false statement is true **without** revealing the actual sum given by `real_sum` which can be used to deanonimize us. 


As we said before, this is done by making an appropriate choice of the polyonomial $s$. To see how to do it, let's first look into the verifier's code in more detail.

### The verifier 
The prover and verifier are using the [Fiat-Shamir transform](https://en.wikipedia.org/wiki/Fiat–Shamir_heuristic) in order to enable a non-interactive protocol. Essentially, the random challenges are going to be produced from the prover's own custom input so they cannot be controlled by the prover in any way. The verifier produces a Fiat-Shamir random generator:
```rust
    let mut fs_rng = FS::initialize(&to_bytes![&PROTOCOL_NAME, statement].unwrap());

    fs_rng.absorb(&to_bytes![proof.s, proof.h, proof.g].unwrap());
    let f = LabeledCommitment::new("f".into(), statement.f.clone(), None);
    let s = LabeledCommitment::new("s".into(), proof.s.clone(), None);
    let h = LabeledCommitment::new("h".into(), proof.h.clone(), None);
    let g = LabeledCommitment::new(
        "g".into(),
        proof.g.clone(),
        Some(statement.domain.size() - 2),
    );
```
We can then use the Fiat-Shamir generator to get the openning challenge $xi\in \mathbb{F}$. Arkworks provides a convenient type [`QuerySet`](https://docs.rs/ark-poly-commit/latest/ark_poly_commit/type.QuerySet.html) to encode the oppening challege for several named polynomials. 
```rust
    let xi = F::rand(&mut fs_rng);
    let opening_challenge = F::rand(&mut fs_rng);

    let point_label = String::from("xi");
    let query_set = QuerySet::from([
        ("f".into(), (point_label.clone(), xi)),
        ("h".into(), (point_label.clone(), xi)),
        ("g".into(), (point_label.clone(), xi)),
        ("s".into(), (point_label, xi)),
    ]);
```
We can then check if the given commitments match the claimed openning values using the [`batch_check`](https://docs.rs/ark-poly-commit/0.3.0/ark_poly_commit/trait.PolynomialCommitment.html#method.batch_check) method on a the polynomial commitment. 

```rust
    let res = PC::batch_check(
            vk,
            &[f, s, h, g],
            &query_set,
            &evaluations,
            &proof.pc_proof,
            opening_challenge,
            rng,
        ).unwrap();
        
    assert!(res)
```
Now that we verified the openning values, we can test the required relation $f(xi) + s(xi) = h(x_i)\cdot Z_H(xi) + xi\cdot g(xi)$. 
```rust
    let card_inverse = statement.domain.size_as_field_element().inverse().unwrap();
    let lhs = proof.s_opening + proof.f_opening;
    let rhs = {
        let x_gx = xi * proof.g_opening;
        let zh_eval = statement.domain.evaluate_vanishing_polynomial(xi);

        x_gx + proof.h_opening * zh_eval + statement.sum * card_inverse
    };

    assert_eq!(lhs, rhs) 
```
The verifier accepts when $lhs = rhs$.

### The malicious prover
As we said, the vulnerability of the verifier comes from the fact that we are given complete freedom to choose the polynomial $s$.

**Remark**: the easiest choice is to set $s = -f$, which clearly sums to zero over any set. However, in that case the polynomials $h$ and $g$ will also be zero, which can hint to the verifier that something wrong is happening since we provide opennings and commitments to both $h$ and $g$.

#### Finding the adversarial polynomials
The verifier checks a proof for the sum of the polynomial $f + s$. So, to cheat the verifier, the only condition $s$ needs to satisfy is that $\sum_{a\in \mathrm{domain}} s(a) = -\sum_{a\in \mathrm{domain}}f(a)$. However, we want to make $s$ as random as possible with the sum being the only constraint. We can acheive that by producing a random seed for $s$ using the Fiat-Shamir mechanism (other ways are possible too):
```rust 
    let seed =  b"GEOMETRY-SUMCHECK, double spend attack";

    let mut s_rng = FS::initialize(&to_bytes![&seed, statement].unwrap());
```
We can then use `s_rng` to produce random values for $s$ to have at any given point in the domain. At any point $a$ in the domain we take the value $-f(a)$ and add a random value to it. We keep track of the sum of all the random values and then correct the first element by that sum to make sure out constraint on $s$ remains valid.

```rust
    let mut evaluations = Vec::new();
    let mut sum = F::zero();
    for h in statement.domain.elements() {
        let random = F::rand(&mut s_rng);
        sum = sum + random;
        evaluations.push(-f.evaluate(&h) + random);
    }
    evaluations[0] = evaluations[0] - sum;
```
The polynomial $s$ itself is then determined by interpolating the values set above. Arkworks provides convenient ways to implement this. First, we collect the points and the evaluations into an instance of [`Evaluations`](https://docs.rs/ark-poly/latest/ark_poly/evaluations/univariate/struct.Evaluations.html).
```rust    
    let evals = Evaluations::from_vec_and_domain(evaluations, statement.domain);
```
This instance can then be used to get the interpoleted polynomial using the [`interpolate`](https://docs.rs/ark-poly/latest/ark_poly/evaluations/univariate/struct.Evaluations.html#method.interpolate) method.
```rust
    let s = evals.interpolate(); 
    let p =  f.polynomial().clone() + s.clone();
```
We have defined $s$ via interpolation and also defined $p$ to be the sum of $s$ and $f$. The polynomial $p$ has values that sum to zero on the domain by construction!

All that's left is to produce $g$, $h$ to fit the univariate sum-check proof for $p$. Once again, the [ark_poly](https://docs.rs/ark-poly/latest/ark_poly/) crate provides conveninet ways to do this. Using the [divide_by_vanishing_poly](https://docs.rs/ark-poly/latest/ark_poly/polynomial/univariate/struct.DensePolynomial.html#method.divide_by_vanishing_poly) we get polynomials $h, r$ such that $p(X) = h(X)Z_H(X) + r(X)$. We then find $g$ by dividing $r(X)$ by $X$.  
```rust
    let (h, r) = p.divide_by_vanishing_poly(statement.domain).unwrap();
    let x =  DensePolynomial::from_coefficients_vec(vec![F::zero(), F::one()]);
    let g = r.div(&x);
```
We now have all the ingridients to make our fictitious proof. 
#### Preparing the proof

Now that we have the polynomials, all we need to do is to produce commitments, opennings, and package them into the Proof data structure. First, we will make labeled polynomials and then use them to make labeled commitments. This makes it easy for batch openning and quarying. 

```rust
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
```
To produce the opennings of the commitments, we follow the Fiat-Shamir protocol to make a random challenge, as the verifier does. 
```rust
    let mut fs_rng = FS::initialize(&to_bytes![&PROTOCOL_NAME, statement].unwrap());

    fs_rng.absorb(&to_bytes![
        s_c.commitment().clone(), 
        h_c.commitment().clone(), 
        g_c.commitment().clone()].unwrap());
```
We can now use the generator `fs_rng` to produce the opening challenge and quary set, as the verifier does. 
```rust
    let xi = F::rand(&mut fs_rng);
    let opening_challenge = F::rand(&mut fs_rng);
    
    let point_label = String::from("xi");
    let query_set = QuerySet::from([
        ("f".into(), (point_label.clone(), xi)),
        ("h".into(), (point_label.clone(), xi)),
        ("g".into(), (point_label.clone(), xi)),
        ("s".into(), (point_label, xi)),
    ]);
```
We can now collect the opennings of our commitments by evaluating our polynomials on the quary set and produce a proof for these opennings using the [`bath_open`](https://docs.rs/ark-poly-commit/latest/ark_poly_commit/trait.PolynomialCommitment.html#method.batch_open) method.
```rust
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
```
Finally, we can return an instance of Proof containing all the data we produced
```rust
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
```
and we are done! 
# Power Corrupts
## Introduction
### Problem Description
Bob has invented a new pairing-friendly elliptic curve, which he wanted to use with [Groth16](https://eprint.iacr.org/2016/260.pdf).
For that purpose, Bob has performed a trusted setup, which resulted in an SRS containting
a secret $\tau$ raised to high powers multiplied by a specific generator in both source groups. The public parameters of the protocoll include two groups $\mathbb{G}_1$, $\mathbb{G}_2$ of a given order $q$, a generator $P$ of $\mathbb{G}_1$, a generator $Q$ of $\mathbb{G}_2$ and a pairing
```math
e \colon \mathbb{G}_1 \times \mathbb{G}_2 \rightarrow \mathbb{G}_T
```
The parameters of the curve and the setup are available [here](https://gist.github.com/kobigurk/352036cee6cb8e44ddf0e231ee9c3f9b).

Alice wants to recover $\tau$ and she noticed a few interesting details about the curve and the setup. Specifically, she noticed that the sum $d$ of the highest power $d_1$ of $\tau$ in $\mathbb{G}_1$ portion of the SRS, meaning the SRS contains an element of the form $\tau^{d_1} G_1$ where $G_1$ is a generator of $\mathbb{G}_1$, and the highest power $d_2$ of $\tau$ in $\mathbb{G}_2$ divides $q-1$. 

Additionally, she managed to perform a social engineering attack on Bob and extract the 
following information: if you express $\tau$ as $\tau = 2^{k_0 + k_1((q-1/d))} \mod r$, 
where $r$ is the order of the scalar field, $k_0$ is 51 bits and its fifteen most 
significant bits are 10111101110 (15854 in decimal). That is A < k0 < B where 
A = 1089478584172543 and B = 1089547303649280.

Alice then remembered the Cheon attack...

## The Cheon Attack
Many cryptographic protocols rely on the difficulty of the discrete logarithm problem. Namely, given a finite group $G$ of prime order $q$ and elements $g, h \in G$, it is generally difficult to find $\alpha\in \mathbb{F}_q^{*}$ such that
```math
g^{\alpha} = h
```
This is known as the *discrete logarithm problem* (DLP for short). The best known algorithms that work for any group of order $q$ require about $\mathcal{O}(\sqrt{q})$ multiplication steps. The actual cost then depends on the cost of computing multiplications and comparing elements in $G$. One method that achieves this is the baby-step giant-step algorithm reviewed below. 

The power $\alpha$ is generally kept as a secret and then it is considered safe for the owner of the secret to send publicly elements like $g$ and $g^{\alpha}$, relying on the fact that finding $\alpha$ itself is inttractable.

**Remark:** For some groups, there are faster methods. For example, for the additive group $\mathbb{F}_q$ the discrete logarithm problem takes the form $\alpha \cdot g = h$ which has the trivial solution $\alpha = g^{-1}h$ since $g$ is non-zero. A non-trivial case is the multiplicative group $\mathbb{F}_q$ where there are solutions to the discrete logarithm problem with $\mathcal{O}(\exp(c\sqrt[3]{\log(q)\log(\log(q))^2}))$ steps, which is asympotically less than $\mathcal{O}(q^\epsilon)$ for every $\epsilon > 0$.

Many modern protocols, such as polynomial commitment schemes used in various constructions of ZK SNARKS, require the publication of not just $g$ and $g^\alpha$ but also the publicaton of powers $g^{\alpha^2}, g^{\alpha^3} \dots, g^{\alpha^d}$ for some integer $d$. It was noticed by Jung Hee Cheon in the [paper](http://www.math.snu.ac.kr/~jhcheon/publications/2010/StrongDH_JoC_Final2.pdf) that some choices of $d$ make it possible to find $\alpha$ considerably faster, with only $\sqrt{\frac{q-1}{d}} + \sqrt{d}$ multiplication steps needed. 

First, we describe the baby-steps giant-step agorithm and then explain how it is used in the Cheon attack.

### The Baby-Step Giant-Step Algorithm (BSGS)
The baby-step giant-step algorithm is was invented by [Daniel Shanks](https://en.wikipedia.org/wiki/Daniel_Shanks). The algorithm works as follows:

Denote $m = \lceil\sqrt{q}\rceil$. Then we can write $\alpha$ as $\alpha = u + mv$ for some $u$ and $v$ between $1$ and $m$. Using this notation, we get the equation $g^{u+mv} = h$ which is equivalent to $g^{mv} = hg^{-u}$.

We create a hash map containing the key-value pairs $(hg^{-u}, u)$ for $1\leq u\leq m$. We then loop over values of $v$ from $1$ to $m$ and query the map with the key $g^{mv}$. 

This only requires $\mathcal{O}(\sqrt{q})$ multiplications and storing $\mathcal{O}(\sqrt{q})$ elements.

### The attack
Here is how we can use the information that $d$ divides $q-1$. The essential observation is that while $\alpha$ lives in the multiplicative group $\mathbb{F}_q$ which is of order $q-1$, the element $\alpha^{d}$ lives in a subgroup of order $\frac{q-1}{d}$. This is simply because
```math
(\alpha^{d})^{\frac{q-1}{d}} = \alpha^{q-1} = 1 \ \mathrm{mod} \ q.
```
Now we can give the details of the attack, later we will also give the code usd for the puzzle but for now we will only describe the algorithm. For simplicity, we assume that $2$ is a is a generator of the multiplicative group $\mathbb{F}_q^{*}$ (this is the case of this puzzle)[^1]. In this case the element $\alpha$ can be written uniquely in the form $\alpha = 2^{k}$ for some $0\leq k\leq (q-1)$. The Cheon attack starts by writing $k = k_0 + k_1 \frac{q-1}{d}$. Using this paramerization, we have that
```math
\alpha^{d} = 2^{kd} = 2^{d\cdot k_0 + (q-1)} = 2^{d\cdot k_0}\cdot 2^{q-1} = 2^{d\cdot k_0}\  \mathrm{mod}\ q
```
 which gives us the equation $g^{\alpha^d} = g^{2^{d\cdot k_0}}$. The first observation is that finding $k_0$ can be done with $\sqrt{\frac{q-1}{d}}$ steps. Namely, denote $m = \left\lceil \sqrt{(q-1)/d}\right\rceil$ and denote $\eta_0 = 2^{d}$, $g_d = g^{2^d}$. We can use the baby-step giant-step algorithm and write $k_0 = u + mv$ with $1\leq u,v\leq m$. This just means that we construct a hash table with pairs $(g_d^{\eta_0^{-u}}, u)$  and then comparing the keys of the table with the values $g^{\eta_0^{mv}}$ for $v$ between $1$ and $m$.  

Thus, we can find $k_0$ with $\mathcal{O}(\sqrt{(q-1)/d})$ multiplication steps. 

[^1]: In other words, 2 is a primitive $(q-1)$-root of unity mod $q$.

Now that we have $k_0$, all we need is to find the value $\alpha\cdot 2^{-k_0}$. If we denote by $\eta_1 = 2^{(q-1)/d}$, we can run the baby-step giant-step algorithm with around $\sqrt{d}$ steps. Namely, if we denote
```math
g_1 = g^{\alpha 2^{-k_0}}, \eta_1 = 2^{(q-1)/d}
```
we can then write $k_1 = u_1 + m_1 v_1$ with $m_1 = \lceil \sqrt{d}\rceil$. Then, we store the key-value pairs $(g_1^{\eta^{-u}}, u)$ and compare them against the values $g^{\eta_1^{m_1v}}$ for $1\leq v\leq m_1 $

This means we can find $k_1$ with $\mathcal{O}(\sqrt{d})$ multiplication steps.

## Elliptic Curves and Pairings 
This section contains some background on elliptic curves and pairings. The material of this section is not strictly needed for understanding the solution of the puzzle and can be skipped. 

Elliptic curves are an important source of finite groups for which the discrete logarithm problem is assumed to be hard. For a mathematical introduction to elliptic curves, the classic reference is [Silverman's book](https://link.springer.com/book/10.1007/978-0-387-09494-6). For an introduction aimed at cryptography see [Pairings for Beginners](https://static1.squarespace.com/static/5fdbb09f31d71c1227082339/t/5ff394720493bd28278889c6/1609798774687/PairingsForBeginners.pdf) by Craig Costello.

### An introduction to Elliptic Curves
We fix a prime $p$ which is not $2$ or $3$. An elliptic curve $E$ defined over the field $\mathbb{F}_p$ is given by the equation
```math
y^2 = x^3 + Ax + B
```
where $A, B$ are elements of $\mathbb{F}_p$. What do we mean by this? 

For every degree $l>0$, we have a unique field of size $\mathbb{F}_{p^l}$, called the extension field of $\mathbb{F}_p$ of degree $l$. These fields also contain each other so that we have an infinite sequence of inclusions:
```math
 \mathbb{F}_p \subseteq \mathbb{F}_{p^2}\subseteq \dots \mathbb{F}_{p^l} \subseteq  \mathbb{F}_{p^{l+1}} \subseteq \dots
```
For every such field we consider the set of *rational points* over it:
```math
E(\mathbb{F}_{p^l}) = \{(x,y)\in \mathbb{F}_{p^l}\times \mathbb{F}_{p^l}|\ y^2 = x^3 + Ax + B\} \cup O
```
it is the set of pairs satisfying the elliptic curve equations, plus an extra point $O$, which by definition belongs to $E(\mathbb{F}_p)$. The point $\mathbb{O}$ is sometimes caled "the point at infinity", which we think of as corresponding to the value of both $x$ and $y$ being zero. 

The inclusion of the fields means that we also have corresponding inclusions of the sets of rational points of the elliptic curve:
```math
 E(\mathbb{F}_p) \subseteq E(\mathbb{F}_{p^2})\subseteq \dots E(\mathbb{F}_{p^l}) \subseteq  E(\mathbb{F}_{p^{l+1}}) \subseteq \dots
```
We sometimes identify the curve $E$ itself with the union of all these rational points, namely, 
```math
E = \bigcup_{l>0} E(\mathbb{F}_{p^l})
```
In practice, we can only implement a finite amount of information so all of the computations we will do will take place in a **big enough** field $\mathbb{F}_{p^l}$. We will see how to choose it shortly.

In this puzzle, the elliptic curve is defined by the equation
```math
E_{\mathrm{BLS}}: \ y^2 = x^3 + 4
```
We will use $E_{\mathrm{BLS}}$ to refer to this curve. It is a member of the Barreto-Lynn-Scott (BLS) family of curves.

A point $P$ not equal to $O$ is said to be defined over $\mathbb{F}_{p^l}$ if it is the smallest value such that $P$ can be written as a pair $(x,y)$ with $x, y$ in $\mathbb{F}_{p^l}$. By definition, the point $O$ is defined over $\mathbb{F}_p$.

**Remark:** The origin of the terminology "curve" comes from algebraic geometry, and is inteded to emphasize that there is only one "continuous" degree of freedom. Namely, for any given $x$ there are two possible values of $y$ such that $(x,y)$ belongs to $E$, since $y$ appears in the second power. 

### The Group Structure
There is a way to give $E$ the structure of a commutative group, namely, there is a group operation
```math
\oplus : E \times E \rightarrow E
```
that is commutative, associative, and the point $O$ acts as the identity. Namely, for any point $P$ of $E$
```math
P\oplus O = P
``` 
Moreover, this group operation is such that for each $l$, the subset of elements defined over $\mathbb{F}_{p^l}$ is preserved under it, that is,
```math
P, Q \in E(\mathbb{F}_{p^l}) \implies P\oplus Q \in E(\mathbb{F}_{p^l})
```
This means that the group operation on $E$ restricts to a group operation
```math
\oplus : E(\mathbb{F}_{p^l}) \times E(\mathbb{F}_{p^l}) \rightarrow E(\mathbb{F}_{p^l})
```
The operation itself is defined via geometric means. We will not go into it in this write-up since it is more concerned with the formal properties of elliptic curve groups and pairings and stating these dos not rely on the actual definition of the group operation. 

### Subgroups of Prime Order
Now let $q$ be a prime different from $p$. We denote
```math
E[q] = \{P\in E | \ qP = O\}
```
the subgroup of points of order that divides $q$. It is not clear a-priori that such points exist. However, if $q$ is a prime different from $p$, then it is known that there are exactly $q^2$ points of order $q$. We can consider the group $E[q]$ as a vector space over $\mathbb{F}_q$, with addition given by the elliptic curve addition and multiplication given by scalar multiplication (which is just succesive additions).

For our curve $E_{\mathrm{BLS}}$, the group $\mathbb{G}_1$ is a subgroup of $E(\mathbb{F}_p)$, namely, the generator $P$ and thus all elements have coordinates $(x,y)$ that are elements of $\mathbb{F}_q$ and $P$ generates all the elements of order $q$ which are also defined over $\mathbb{F}_p$.

The second subgroup $\mathbb{G}_2$ of order $q$ we will consider will be embedded in
```math
\mathbb{G}_2 \subseteq E_{\mathrm{BLS}}(\mathbb{F}_{p^{12}})
```
and we are given a fixed generator $Q$ of that group. Representing it in practice means having to give a pair of elements of $\mathbb{F}_{p^{12}}$, which can be demanding in terms of memory. We will see later how to represent $Q$ more efficiently.

### The Weil Pairing
The third element of the problem is the pairing. The pairing consists of another group $\mathbb{G}_T$ of order $q$ and a map
```math
e: \mathbb{G}_1 \times \mathbb{G}_2 \rightarrow \mathbb{G}_T
```
such that the following propeties are satisfied:
1. $e(P+P',Q) = e(P,Q)e(P',Q)$
2. $e(P,Q + Q') = e(P,Q)e(P,Q')$

Note that we write the multiplication operation on $\mathbb{G}_T$ multilicatively for reasons that we will explain shortly. These properties imply that for all elements $\alpha$, $\beta$ in $\mathbb{F}_q$ we have
```math
e(\alpha P, Q) =  e(P,Q)^{\alpha}, \quad e(P, \beta Q) = e(P, Q)^{\beta}
```
From a cryptographic prespective, the usefullness of a pairing comes partly from enabling us to multiply secret values. Namely, assume we have secret numbers $\alpha$, $\beta$ in $\mathbb{F}_q$ and we are only given the elements $\alpha P$ and $\beta Q$. By pairing these elements we get:
```math
e(\alpha P, \beta Q) = e(P, \beta Q)^{\alpha} = (e(P,Q)^{\beta})^{\alpha} = e(P,Q)^{\alpha \beta}
```
which represents the multiplication of $\alpha$ and $\beta$ as an element in the group $\mathbb{G}_T$.

The origin of the pairing in our groups comes is induced from the *Weil pairing* from the theory of elliptic curves which is named after the mathematician [André Weil](https://en.wikipedia.org/wiki/André_Weil). The geometric origins of this pairing are beyond the scope of this note, but we will try to give some prespective on it. 

For every elliptic curve $E$, there is a pairing
```math
e_q: E[q]\times E[q] \rightarrow \mathbb{F}^{*}_{p^{k}}
```
where $k$ is called the *embedding degree*. It is the smallest integer such that $\mathbb{F}_{p^{k}}$ contains a primitive $q$-th root of unity. Thus, we define $\mathbb{G}_T$ to be the multiplicative subgroup of elements of $\mathbb{F}_{p^{k}}$ of order $q$. This is the image of the pairing.

The Weil pairing satisfies some interesting properties:
1. $e(S, T + T') = e(S,T)e(S,T')$ for all $S,T, T'\in E[q]$
2. $e(S + S', T) = e(S,T)e(S',T)$ for all $S,S', T\in E[q]$
3. $e(S,T) = e(T,S)^{-1}$ for all $S,T\in E[q]$

In particular, the last property implies that $e(T,T)=1$ for all $T\in E[q]$. We can use these properties to gain some insights on the Weil pairing. 

In the case of our puzzle, $k=12$. Moreover, the points $P$ and $Q$ form a basis to the elements $E[q]$ consisered as a vector space over $\mathbb{F}_q$. That is, every two elements $S$, $T$ in $E_{\mathrm{BLS}}[q]$ can be written uniquely in the form
```math
S = a P + b Q, \ T = cP + dQ
```
We can use properties of the Weil pairing to express the pairing of $S$ and $T$ as follows:
```math
e(S,T) = e(aP+bQ, cP+dQ) = e(P, cP+dQ)^{a} e(Q, cP+dQ)^{b} = e(P,Q)^{ad}\cdot e(Q,P)^{bc} = e(P,Q)^{ad-bc}
```
That is, we got that 
```math
e(aP+bQ, cP+dQ) = e(P,Q)^{\mathrm{det}\left(\begin{array}{cc} 
a & b\\
c & d
\end{array}\right)}
```
This gives a connection between the Weil pairing and the determinant, which is not coincidental. 

Finally, the pairing in the puzzle is obtained simply by the restriction of the Weil pairing in each of the terms
```math
e: \mathbb{G}_1 \times \mathbb{G}_2 \rightarrow \mathbb{G}_T \subseteq \mathbb{F}^{*}_{p^{12}}
```
namely, restricting to the subgroups
```math
\mathbb{G}_1 \subseteq E[\mathbb{F}_p] \subseteq E[\mathbb{F}_{p^{12}}], \ \mathbb{G}_2 \subseteq E[\mathbb{F}_{p^{12}}]
```

#### Pairing Friendly Curves 
The presence of a pairing introduces a volunrebility in the discrete logarithm problem of an elliptic curve. To see this, note that we can use the pairing to transfer the DLP over $E[q]$ to the DLP over $\mathbb{F}_{p^{k}}$. This is actually what we will do later in the Cheon attack. Thus, if $k$ is too small, this could be a problem. 

Curves such as $E_{\mathrm{BLS}}$ are favorable both in terms of embedding degree and in terms of the prime order subgroup they contain. Such curves are often called *pairing friendly*.

#### Twisted curves and short representations
Recall that
```math
\mathbb{G}_1 \subseteq E(\mathbb{F}_p), \quad \mathbb{G}_2\subseteq E(\mathbb{F}_{p^{12}})
```
In particular, in order to implement the second group we need to represent points of it as pairs of elements $(x,y)$ of $\mathbb{F}_{p^{12}}$, which is a huge field. Luckily, we can avoid this using some isights from algebraic geometry. This realization that was given in the [work](https://www.iacr.org/archive/asiacrypt2001/22480516.pdf) of Boneh, Lynn, and Shacham who applied it to get short signatures (namely, elements of $\mathbb{G}_2$) in a pairing bases signature scheme. 

The key observation is that the reason we need to go all the way to $\mathbb{F}_{p^{12}}$ is because we insist on representing points in a certain coordinate system, namely, we are thinking of our curve as determined by the equation $y^2 = x^3 + Ax + B$. 

What changes can we make to this equation without changing the curve? not much if we want to preserve all the structure. However, if we only want to preserve the group $E(\mathbb{F}_{p^{12}})$, then we can make a change of coordinates while keeping the group law the same. 

To make this explicit, we will focus only on our curve $E_{\mathrm{BLS}}$ given by $y^2 = x^3 + 4$. We fix an element $i$ such that
```math
i\in \mathbb{F}_{p^2}, \quad i^2 = (p-1) = -1 \  \mathrm{mod }\ p 
```
and we fix an element $u$ such that
```math
u\in \mathbb{F}_{p^{12}}, \quad u^6 = (1+i)^{-1}
```
We define the coordinate change sending a pair $(x,y)$ of elements in $\mathbb{F}_{p^{12}}$ to the pair $(x',y')$ given by
```math
(x',y') = (x/u^2, y/u^3) 
```
What sort of equation do $x', y'$ satisfy? well, if we look at the formulas, we get
```math
(y')^2 = (yu^-3)^2 = y^2 u^{-6} = (x^3 + 4)u^{-6} = ((x'u^2)^3 + 4)u^{-6} = (x')^2 + 4u^6 = (x')^2 + 4(i+1).
```
where in the last equality we used the equation satisfied by $u$. So, the elements $(x',y')$ are actually points of the curve $E'$ defined over the field $\mathbb{F}_{p^2}$ given by
```math
E' \colon y^2 = x^3 + 4(1+i)
```
The map taking $(x,y)$ to $(x',y')$ is defining a map 
```math
\Psi : E(\mathbb{F}_{p^{12}}) \rightarrow E'(\mathbb{F}_{p^{12}})
```
which is a bijection that respects the group law, namely, $\Psi(x+y) = \Psi(x) + \Psi(y)$. Thus, we can identify these two groups using $\Psi$. 

The main feature of this for us is that it turns out (not reviewed precisely here) that using $\Psi$ it is possible to give an indentification
```math
\mathbb{G}_2 \rightarrow \Psi(\mathbb{G}_2) \subseteq E'(\mathbb{F}_{p^2}).
```
This means that we can use a representation of $Q$ and all its powers as pairs $(x', y')$ of elements in $\mathbb{F}_{p^2}$, which is a significant saving!

What is happening here and how could you have guessed this formulas?

You may have seen changes of coordinates in math classes, for example, any invertible $n\times n$ matrix $M$ can be considered   as a cooordinate change. Another example is the transition between Cartesian and Polar coordinates. However, these changes of coordinates usually map a certain set into itself and not to a different set, so what is happening here?

What's different now is that in calculus classes nobody cared about the algebraic structure. Namely, if we have a set defined by algebraic equations, it can have the same set of elements as just a collection of elements in the algbraic closure, but not in all finite field. In our example of the elliptic curve, the two curves $E$ and $E'$ can be identified as curves defined over $\mathbb{F}_{q^{12}}$, but not when considered over a smaller subfield. It is in fact possible to explicitely calculate all such pairs of $E$ and $E'$ for any elliptic curve (see Proposition X.5.4. in Silverman's book). 

## Solving the Puzzle
The public parameters of the puzzle consist of groups $\mathbb{G}_1$ with a generator $P$, a group $\mathbb{G_2}$ with a generator $Q$, and the elements $\tau P, \tau^{d_1}P, \tau^{d_1}Q$. Our goal is to implement the function
```rust
fn attack(P: G1, tau_P: G1, tau_d1_P: G1, Q: G2, tau_d2_Q: G2) -> i128
```
which returns the secret $\tau$. 

The verifier code is simple, simply check that `tau*P = tau_P`:
```rust
pub fn verify(P: G1, tau_P: G1, tau_128: i128) -> bool {
    let tau = Fr::from(tau_128);
    return tau_P == P.mul(Fr::from(tau));
}
```
### The attack
Alice noticed that $d = d_1 + d_2$ divides $q-1$. In order to apply Cheon's attack we need the $\tau^d$-power of an element. However, we are only given $\tau^{d_1}P$ and $\tau^{d_2}Q$. This is where we can utilize the pairing. Denote $R = e(P, Q)$, which is a generator of $\mathbb{G}_T$. Using the billinearity properties of the pairing, we can compute $\tau R$, $\tau^{d} R$ via:
```math
e(\tau^{d_1}P, \tau^{d_2}Q) = \tau^{d_1}e(P, \tau^{d_2}Q) = \tau^{d_1}\tau^{d_2}e(P, Q)= \tau^{d} e(P,Q) = \tau^{d} R
```
Now that we have $R$, $\tau R$, and $\tau^{d} R$, we can use a Cheon-style attack using the fact that $d$ divies $q-1$. Recall that the Cheon attack has two parts, first we write $\tau = 2^{k_0 + k_1(\frac{q-1}{d})} $ and use the baby-step giant-step algorithm twice to find $k_0$ and then finding $k_1$.

#### Finding $k_0$ - using the extra information
The standard Cheon attack described before requires $\mathcal{O}(\sqrt{(q-1)/d})$ multiplication steps and memory. This might work given enough compute and time, but if you are using a laptop that's few years old that might not be the best thing to do as you can quickly run out of memory. In fact, there are algorithms like [Pollard's rho](https://en.wikipedia.org/wiki/Pollard%27s_rho_algorithm_for_logarithms) which have versions requiring less memory, but that still leavs an inconvenietly long running time. 

Alice gave us more information, namely, we know that $k_0$ lies in a range between given constants $A$ and $B$. But how can be modify the Cheon attack with this information?

The answer comes from realizing that the baby-step giant-step procedure is not related to the order of the group, but it's a general procedure that can be used every time we partition our domain into giant steps. In our case, this simply means that we can write $k_0 = A + u + mv$ with $m = \lceil \sqrt{B-A}\rceil$ and with $1\leq u,v\leq m$. Denoting $\eta_0 = 2^{d}$, we have the euqation
```math
\tau^d = 2^{d\cdot k_0} = \eta_0^{k_0} = \eta_0^{A} \cdot \eta_0^{u + mv} 
```
If we denote $R_d = 2^{\eta_0^{-A}}R =2^{-A}(\tau R)$, we can run the BSGS algorithm on the equation
```math
\eta_0^{-u}R_d = \eta_0^{mv} R
```
This only requires $\mathrm{O}(\sqrt{B-A})$ multiplication steps, which is much smaller than $\mathcal{O}(\sqrt{(q-1)/d})$.

#### Finding $k_1$ 
This step is essentially the same as the one in the original Cheon's attack running the BSGS algorithm with the equation
```math
\eta_1^{-u}R_1 = \eta_1^{m_1v_1} R
```
where we defined $R_1 = 2^{-k_0}\cdot tau_R $ and $\eta_1 = 2^{q-1/d}$. This step requires $\mathcal{O}(d)$ elements. 

### The attacker in code
Let's see how to implement this and actually solve the puzzle. First, we set up a general baby-steps giant steps function for iterating finding $(u,v)$ satisfying a relation $g_1^{\eta^{-u}} = g^{\eta^{mv}}$. We utilize the provided method [`pow_sp2(p, exp, n)`](https://github.com/ZK-Hack/puzzle-power-corrupts/blob/1e65f77052e41f8ca3a85bbe06020d20862a1713/src/utils.rs#L26) which returns a hash map of pairs $(p^{exp^k}, k)$ for $k = 1,\dots,n$. As this function was quite fast, we just computed tables for both baby steps and giant steps. 
```rust
fn baby_steps_giant_steps<F: Field>(
    g_d: F,
    g: F,
    eta_1: Fr,
    factor: u64,
    m: u64,
) -> Option<(u64, u64)> {
    let eta_1_inv = eta_1.inverse().unwrap();
    let eta_1_factor = pow_sp(eta_1, factor.into(), 64);

    // Construct lookup tables for baby steps and giant steps powers
    let lookup_baby_steps = pow_sp2(g_d, eta_1_inv, m);
    let lookup_giant_steps = pow_sp2(g, eta_1_factor, m);

    // Try to find values for u, v
    let mut u_log = None;
    let mut v_log = None;
    for (e_v, v) in lookup_giant_steps {
        u_log = lookup_baby_steps.get(&e_v);
        if u_log.is_some() {
            v_log = Some(v);
            break;
        }
    }
    // Return the values found
    match (u_log, v_log) {
        (Some(u), Some(v)) => Some((*u, v)),
        _ => None,
    }
}
```
We can use this function to write our attacker. First, let's hard-code some numerical values we will need, these can also be computed in Rust. 
```rust
const A: u64 = 1089478584172543;
const B: u64 = 1089547303649280;
const D: u32 = 702047372;
const QD: u64 = 1587011986761171; //(q-1)/D
const FACTOR: u64 = 262144; // floor(sqrt(n))
const FACTORONE: u64 = 26497; // floor(sqrt(D))+1
```
We will refer to these constants in the code below. The first step is to save and calulate $R, \tau^{d}R$, and the powers $2^{d}$. We do this using the pairing and the provided [`pow_sp`](https://github.com/ZK-Hack/puzzle-power-corrupts/blob/1e65f77052e41f8ca3a85bbe06020d20862a1713/src/utils.rs#L10) method:
```rust
pub fn attack(P: G1, tau_P: G1, tau_d1_P: G1, Q: G2, tau_d2_Q: G2) -> i128 {
    //compute e((tau^d_1)P, (tau^d_2)Q) = tau^d*e(P,Q)
    let R = Bls12Cheon::pairing(P, Q);
    let tau_d_R = Bls12Cheon::pairing(tau_d1_P, tau_d2_Q);
    let tau_R = Bls12Cheon::pairing(tau_P, Q);

    // Some powers of two
    let two = Fr::from(2);
    let two_d = pow_sp(two, D.into(), 32);
    let two_d_inv = two_d.inverse().unwrap();
    let two_d_A_inv = pow_sp(two_d_inv, A.into(), 64);
    
    ...
```
Next, we find $k_0$ by using the [`baby_steps_giant_steps`](#baby_steps_giant_steps) function we just defined for finding $(u,v)$ satisfying the relation $\eta_0^{-u}R_d = \eta_0^{mv} R$ with $\eta_0 = 2^{d}$:. 

However, note that [`baby_steps_giant_steps`](#baby_steps_giant_steps) accepts elements of a field. The element $R$ has the type [`PairingOutput`](https://docs.rs/ark-ec/0.4.0-alpha.3/ark_ec/pairing/struct.PairingOutput.html) which does not satify the [`Field`](https://docs.rs/ark-ff/latest/ark_ff/fields/trait.Field.html) trait. This is because the image of the pairing is the group $\mathbb{G}_T$, which is embedded as a subset of a field of size $p^{12}$, but is not equal to it, so the type representing the image only "remembers" the group law, which is what we want from a correct typing. 

Concretely, as the pairing image is a subgroup of $\mathbb{F}^{*}_{p^{12}}$ the struct [`PairingOutput`](https://docs.rs/ark-ec/0.4.0-alpha.3/ark_ec/pairing/struct.PairingOutput.html) is a wrapper around an element [`TargetField`](https://docs.rs/ark-ec/0.4.0-alpha.3/ark_ec/pairing/trait.Pairing.html#associatedtype.TargetField). That is, we can get the underlying field element from `R` by `R.0`.  
```rust
    let tau_d_A_R = tau_d_R * two_d_A_inv;

    // Write k_0 = A + u + factor*v
    // Run baby-steps big-steps algorithm with respect to u and v
    let n = B - A;
    let factor = FACTOR; // floor(sqrt(n))
    let m = n / factor;

    let (u, v) = baby_steps_giant_steps(tau_d_A_R.0, R.0, two_d, factor, m).unwrap();
    let k_0 = A + u + factor * v;
    let exp = pow_sp(two_d, k_0.into(), 64);
    let two_k_0 = pow_sp(two, k_0.into(), 64);

    // Sanity
    let e_d_k_0 = R * exp;
    assert_eq!(tau_d_R, e_d_k_0);
    
```
Using the value of $k_0$ just found, we use [`baby_steps_giant_steps`](#baby_steps_giant_steps) again to find $k_1$:

```rust

    let two_inv = two.inverse().unwrap();
    let two_k_0_inv = pow_sp(two_inv, k_0.into(), 64);
    let R_1 = tau_R * two_k_0_inv;

    let q_d: u64 = QD; // (q-1)/D
    let two_q_d = pow_sp(two, q_d.into(), 64);

    // Run the second baby-step big-step

    let factor_1: u64 = FACTORONE; // floor(sqrt(D))+1
    let m_1 = (D as u64) / factor_1;

    let (u_1, v_1) = baby_steps_giant_steps(R_1.0, R.0, two_q_d, factor_1, m_1).unwrap();

    let k_1 = u_1 + factor_1 * v_1;

    let two_dq_k_1 = pow_sp(two_q_d, k_1.into(), 64);
    assert_eq!(R_1, R * two_dq_k_1);

```
Finally, we  multiply $2^{k_0}$ with $2^{k_1 \cdot \frac{q-1}{d}}$ to find $\tau$ and convert it to the desired `i128` format,
```rust
    let tau = two_k_0 * two_dq_k_1;
    let tau_128 = bigInt_to_u128(tau.into());
    tau_128.try_into().unwrap()
}
```
and we are done!
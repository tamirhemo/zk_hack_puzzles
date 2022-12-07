# ZK Hack III Puzzle Solutions

This repository contains solutions and write-ups for puzzles given as part of the [ZK Hack III online event](https://zkhack.dev/zkhackIII/). Currently there are solutions for the first two puzzles. The solution to the third puzzle will be uploaded after the submission period is done. 

Each folder contains a README file with an description of the puzzle and an explanation of the solution and some of the necessary background needed to understand it. 

## Puzzle 1 - Zero Sum Game
This puzzle is about a faulty implementation of a system based on the cryptographic sum-check protocol. For an explanation of the puzzle solution, including a background of the underlying cryptography, see [`puzzle 1 README`](./puzzle_1_zero_sum/README.md). 

To run the solution code for this puzzle: 
```
$ cargo run --bin sumcheck-puzzle --release
```
Succesful execusion signifies a good solution, and will output the puzzle description 
```


    ______ _   __  _   _            _
    |___  /| | / / | | | |          | |
       / / | |/ /  | |_| | __ _  ___| | __
      / /  |    \  |  _  |/ _` |/ __| |/ /
    ./ /___| |\  \ | | | | (_| | (__|   <
    \_____/\_| \_/ \_| |_/\__,_|\___|_|\_\
    
Bob has designed a new private payments protocol design, where every note comes with a secret 
polynomial f whose sum over a specific set is zero. This is enforced using a sumcheck protocol.
Once a note is spent, f is modified to a different polynomial whose sum isn't zero. One day, 
after an interesting conversation with her friends, Alice got an idea for an attack that can 
potentially allow her to double spend notes.

Alice successfully double spent a note. Can you figure out how she did it?

Be very careful, if the verifier somehow learns the sum of the modified f, 
they can deanonymize you.

In the rest of protocol that is not described here, the masking polynomial used by 
the prover is opened twice. Therefore, the masking polynomial cannot be a 
constant polynomial.

To see examples of sumcheck, you can review the protocol described in 
https://github.com/arkworks-rs/marlin/blob/master/diagram/diagram.pdf.
```


## Puzzle 2 - Power Corrupts
This puzzle is based on the [Cheon attack](http://www.math.snu.ac.kr/~jhcheon/publications/2010/StrongDH_JoC_Final2.pdf) exploiting public information in certain set-ups for some modern cryptographic protocols. For an explanation of the puzzle and its solution, including a background of the underlying cryptography, see [`puzzle 2 README`](./puzzle_2_power_corrupts/README.md). 

```
$ cargo run --bin blstest --release
```

Succesful execusion signifies a good solution and might take a few minutes to run. It will output the puzzle description:

```
    ______ _   __  _   _            _
    |___  /| | / / | | | |          | |
       / / | |/ /  | |_| | __ _  ___| | __
      / /  |    \  |  _  |/ _` |/ __| |/ /
    ./ /___| |\  \ | | | | (_| | (__|   <
    \_____/\_| \_/ \_| |_/\__,_|\___|_|\_\
    

Bob has invented a new pairing-friendly elliptic curve, which he wanted to use with Groth16.
For that purpose, Bob has performed a trusted setup, which resulted in an SRS containting
a secret $\tau$ raised to high powers multiplied by a specific generator in both source groups. 
The exact parameters of the curve and part of the output of the setup are described in the 
document linked below.

Alice wants to recover $\tau$ and she noticed a few interesting details about the curve and
the setup. Specifically, she noticed that the sum $d$ of the highest power $d_1$ of $\tau$ in 
$\mathbb{G}_1$ portion of the SRS, meaning the SRS contains an element of the form 
$\tau^{d_1} G_1$ where $G_1$ is a generator of $\mathbb{G}_1$, and the highest power $d_2$ 
of $\tau$ in $\mathbb{G}_2$  divides $q-1$, where $q$ is the order of the groups. 

Additionally, she managed to perform a social engineering attack on Bob and extract the 
following information: if you express $\tau$ as $\tau = 2^{k_0 + k_1((q-1/d))} \mod r$, 
where $r$ is the order of the scalar field, $k_0$ is 51 bits and its fifteen most 
significant bits are 10111101110 (15854 in decimal). That is A < k0 < B where 
A = 1089478584172543 and B = 1089547303649280.

Alice then remembered the Cheon attack...

NOTE: for exponentiating $F_r$ elements, use the `pow_sp` and `pow_sp2` functions in
`utils.rs`.

The parameters of the curve and the setup are available at 
https://gist.github.com/kobigurk/352036cee6cb8e44ddf0e231ee9c3f9b
```


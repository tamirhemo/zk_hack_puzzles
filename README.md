# ZK Hack Puzzle Solutions

This repository contains solutions and write-ups for puzzles given as part of the [ZK Hack III online event](https://zkhack.dev/zkhackIII/).

## Puzzle 1 - Zero Sum Game
This puzzle is about a faulty implementation of a system based on the cryptographic sum-check protocol. For an explanation of the puzzle solution 
with the background see [`puzzle 1 README`] (`./system/puzzle_1_zero_sum/README.md). 

To run the solution code for this puzzle: 
```
$ cargo run --bin sumcheck-puzzle --release
```

This will output the puzzle description 
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
This puzzle is based on the Cheon attack. 

```
$ cargo run --bin blstest --release
```

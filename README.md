Sokoban solver
==============

[![pipeline status](https://gitlab.com/martin-t/sokoban-solver/badges/master/pipeline.svg)](https://gitlab.com/martin-t/sokoban-solver/commits/master)
[![dependency status](https://deps.rs/repo/gitlab/martin-t/sokoban-solver/status.svg)](https://deps.rs/repo/gitlab/martin-t/sokoban-solver)

A toy sokoban solver, one of my first Rust projects. It can't do much, I am more playing with the language than trying to write a good solver. It can currently solve only level 1 of the original Sokoban levels because they tend to be large and require goalroom optimizations. It, however, chews through even difficult small levels like [microban](https://gitlab.com/martin-t/sokoban-solver/-/tree/master/levels/microban1) faster than any human ever could.

Some parts are intentionally more general than they need to be so that I can properly test Rust's generics:

- There are 2 level formats (the standard XSB/SOK plus a custom one) for both input and output
- It can solve normal Sokoban levels or "remover" levels (they have exactly one goal spot which eats boxes pushed onto it)
- It can look for both move and push optimal solutions

The original goal was to help me with level 100 of the game [Supaplex](https://en.wikipedia.org/wiki/Supaplex) which is inspired by Sokoban level 43. This is much easier with a remover because there is no need for goalroom optimizations. The version with a remover can be solved in a few seconds, with goals it takes significantly longer and takes much more memory. Similarly, more of the original Sokoban levels can be solved when the goals are replaced with a remover.

State space graphs
------------------

Sokoban-solver can generate graphs to visualize the searched state space:

[![media/state-space-microban-79.dot.png](media/state-space-microban-79.dot.png)](media/state-space-microban-79.dot.png)
*Pack Microban, level 79*

---

[![media/state-space-696-1.dot.svg](media/state-space-696-1.dot.svg)](media/state-space-696-1.dot.svg)
*Pack 696, level 1, older visualization format*

Method
------

Currently uses A* with distances to the nearest goal (or remover) as heuristic. The only deadlock detection is a result of this - boxes on dead end cells have no way to reach any goals.

It can optimize for the lowest number of pushes, moves or both (giving priority to one or the other).

Installation
------------

Requires nightly (will be installed automatically thanks to the `rust-toolchain` file).

Development
-----------

Optionally use `git config core.hooksPath git-hooks` to check the code before committing.

Maintenance status
------------------

Sokoban-solver is passively maintained.

I don't intend to add features or improve the algorithm. Sokoban is a surprisingly deep problem and although it's fun to solve by hand from time to time and I'd love to test my ideas about solving it programatically, there's no way I'd be able to even match the currently existing solvers without a very large time investment.

I might fix bugs but for the most part I am only using this project to get a feeling for how much effort it is to maintain a (small) Rust project long term - I intend to fix clippy lints, update deps, etc.

License
-------

Everything except `levels/` is licensed under AGPLv3 or later.

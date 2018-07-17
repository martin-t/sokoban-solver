#!/usr/bin/env bash

# workaround until https://github.com/japaric/criterion.rs/issues/176 is fixed

firefox \
-new-tab target/criterion/Moves/levels_boxxle1_1_txt/report/index.html \
-new-tab target/criterion/Pushes/levels_boxxle1_1_txt/report/index.html \
-new-tab target/criterion/Pushes/levels_boxxle1_5_txt/report/index.html \
-new-tab target/criterion/Pushes/levels_boxxle1_18_txt/report/index.html \
-new-tab target/criterion/Pushes/levels_boxxle1_108_txt/report/index.html \

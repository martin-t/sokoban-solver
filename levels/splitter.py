#!/usr/bin/env python3

import os
from os import path
import re
import sys

file_name = sys.argv[1]
dir_name = sys.argv[2]

with open(file_name, 'r') as f:
	s = f.read()
	p = ';.*\n\n'
	levels = re.split(p, s)

	os.mkdir(dir_name)
	for i in range(len(levels)):
		if levels[i] == '' or levels[i].isspace():
			continue

		of = open(path.join(dir_name, str(i + 1) + ".txt"), 'w')
		of.write(levels[i])
		of.close()

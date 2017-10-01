f = open("Boxxle1.txt", 'r')
s = f.read()
p = ';.*\n\n'
re.split(p,s)
levels = re.split(p,s)
for i in range(len(levels)):
	of = open("boxxle1/{}".format(i+1), 'w')
	of.write(levels[i])
	of.close()
# leaves one bad file at the end

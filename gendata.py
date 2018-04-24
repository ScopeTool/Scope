#!/usr/bin/env python

import sys
from time import sleep
from random import randint
from math import sin, floor

def log(chan, *args):
	rslt = "~."+str(chan)+'@'
	for i in args:
		rslt+=str(i)+','
	print rslt

data = """
~.Bar@-0.5,-0.5, 0.2
~.Bar@0.0,0.5, 1.5
~.Bar@0.5,-0.25, 1.0
"""
for i in data.split('\n'):
	print i
	# sleep(randint(0,2))

res = 10.0
for i in xrange(0, int(300*2*3.14159*res)):
	y = sin(i/res)
	y = y*min(floor(abs(y)+.2), 1)
	# print ":#D:"+str(i)+","+str(y)
	log("d", i, y)
	log("e", i,  sin(((i+0)/res))*10)
	sys.stdout.flush()
	sleep(0.08)

# r = .9
# divs = 64/4 + 3
# step = r/divs
# for x in xrange(divs): 
# 	log("A", 1*r, 1*r)
# 	log("A", 1*r, -1*r)
# 	log("A", -1*r, -1*r)
# 	log("A", -1*r, 1*r)
# 	r -= step


# Requires that all channels are bound together 
# from itertools import combinations, islice
# step = 2/26.0
# pos = -1+ step /2
# cp = pos
# cpy = pos
# alphabet = list("abcdefghijklmnopqrstuvwxyz")
# for i,c in enumerate(alphabet + range(0,150) + ["".join(x) for x in islice(combinations(alphabet, 3), 500)]):
# 	log(c, cp, cpy, 30)
# 	sys.stdout.flush()
# 	cp += step
# 	if cp > 1:
# 		cpy += step
# 		cp = pos
# 	# sleep(0.1)
